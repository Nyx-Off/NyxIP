use std::net::Ipv4Addr;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ScanResult {
    pub ip: Ipv4Addr,
    pub hostname: String,
    pub mac: String,
    pub vendor: String,
    pub latency_ms: Option<f64>,
    pub open_ports: Vec<u16>,
    pub status: HostStatus,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[allow(dead_code)]
pub enum HostStatus {
    Alive,
    Dead,
    Unknown,
}

impl std::fmt::Display for HostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HostStatus::Alive => write!(f, "Alive"),
            HostStatus::Dead => write!(f, "Dead"),
            HostStatus::Unknown => write!(f, "Unknown"),
        }
    }
}
