use std::net::Ipv4Addr;

/// Get the local IP address of this machine
pub fn get_local_ip() -> Option<Ipv4Addr> {
    // Connect to a public DNS to determine local IP
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:53").ok()?;
    let addr = socket.local_addr().ok()?;
    match addr.ip() {
        std::net::IpAddr::V4(ip) => Some(ip),
        _ => None,
    }
}

/// Suggest a default scan range based on local IP (assumes /24)
pub fn suggest_range() -> String {
    if let Some(ip) = get_local_ip() {
        let octets = ip.octets();
        format!("{}.{}.{}.1-254", octets[0], octets[1], octets[2])
    } else {
        "192.168.1.1-254".to_string()
    }
}
