pub mod types;
pub mod ping;
pub mod arp;
pub mod dns;
pub mod ports;

use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::sync::mpsc;
use types::{HostStatus, ScanResult};

/// Network scanner that probes hosts for liveness, MAC, DNS, and open ports.
pub struct Scanner {
    pub timeout_ms: u32,
    pub scan_ports: bool,
}

impl Scanner {
    pub fn new(timeout_ms: u32, scan_ports: bool) -> Self {
        Self {
            timeout_ms,
            scan_ports,
        }
    }

    /// Scan a single host and return a complete ScanResult.
    pub async fn scan_host(&self, ip: Ipv4Addr, oui_db: &crate::oui::OuiDatabase) -> ScanResult {
        let latency = ping::ping(ip, self.timeout_ms).await;
        let status = if latency.is_some() {
            HostStatus::Alive
        } else {
            HostStatus::Dead
        };

        let mut hostname = String::new();
        let mut mac = String::new();
        let mut vendor = String::new();
        let mut open_ports = Vec::new();

        if status == HostStatus::Alive {
            // Reverse DNS
            hostname = dns::reverse_lookup(ip).unwrap_or_default();

            // ARP → MAC → OUI vendor
            if let Some(m) = arp::get_mac(ip) {
                vendor = oui_db.lookup(&m).unwrap_or_default();
                mac = m;
            }

            // Port scan (optional)
            if self.scan_ports {
                open_ports = ports::scan_ports(ip, ports::COMMON_PORTS, 500).await;
            }
        }

        ScanResult {
            ip,
            hostname,
            mac,
            vendor,
            latency_ms: latency,
            open_ports,
            status,
        }
    }

    /// Scan a range of IPs concurrently.
    ///
    /// Results are streamed through `tx`. Progress updates (done, total) go
    /// through `progress_tx`. The scan can be cancelled via the `cancel` Notify.
    pub async fn scan_range(
        &self,
        ips: Vec<Ipv4Addr>,
        tx: mpsc::UnboundedSender<ScanResult>,
        progress_tx: mpsc::UnboundedSender<(usize, usize)>,
        cancel: Arc<tokio::sync::Notify>,
        oui_db: Arc<crate::oui::OuiDatabase>,
    ) {
        let total = ips.len();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(128));
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let mut handles = Vec::with_capacity(total);

        for ip in ips {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let tx = tx.clone();
            let progress_tx = progress_tx.clone();
            let cancel = cancel.clone();
            let counter = counter.clone();
            let oui_db = oui_db.clone();
            let timeout_ms = self.timeout_ms;
            let scan_ports = self.scan_ports;

            let handle = tokio::spawn(async move {
                let scanner = Scanner::new(timeout_ms, scan_ports);

                tokio::select! {
                    result = scanner.scan_host(ip, &oui_db) => {
                        let _ = tx.send(result);
                    }
                    _ = cancel.notified() => {}
                }

                let done = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                let _ = progress_tx.send((done, total));
                drop(permit);
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }
    }
}
