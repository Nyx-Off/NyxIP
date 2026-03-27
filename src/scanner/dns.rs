use std::net::Ipv4Addr;
use dns_lookup::lookup_addr;

/// Perform a reverse DNS lookup for the given IP address.
/// Returns None if the lookup fails or returns the raw IP string.
pub fn reverse_lookup(ip: Ipv4Addr) -> Option<String> {
    let ip_addr = std::net::IpAddr::V4(ip);
    lookup_addr(&ip_addr)
        .ok()
        .filter(|hostname| hostname != &ip.to_string())
}
