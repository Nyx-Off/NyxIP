use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

/// Common TCP ports to probe during a scan.
pub const COMMON_PORTS: &[u16] = &[
    21, 22, 23, 25, 53, 80, 110, 135, 139, 143, 443, 445, 993, 995,
    1433, 1521, 3306, 3389, 5432, 5900, 8080, 8443,
];

/// Scan the given ports on a host and return those that are open.
pub async fn scan_ports(ip: Ipv4Addr, ports: &[u16], timeout_ms: u64) -> Vec<u16> {
    let mut handles = Vec::with_capacity(ports.len());

    for &port in ports {
        let addr = SocketAddr::new(ip.into(), port);
        let handle = tokio::spawn(async move {
            match timeout(Duration::from_millis(timeout_ms), TcpStream::connect(addr)).await {
                Ok(Ok(_)) => Some(port),
                _ => None,
            }
        });
        handles.push(handle);
    }

    let mut open = Vec::new();
    for handle in handles {
        if let Ok(Some(port)) = handle.await {
            open.push(port);
        }
    }

    open.sort();
    open
}
