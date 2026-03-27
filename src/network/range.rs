use std::net::Ipv4Addr;

/// Parse an IP range string into a list of IPs
/// Supports:
/// - CIDR: "192.168.1.0/24"
/// - Dash range: "192.168.1.1-254" (last octet range)
/// - Dash full: "192.168.1.1-192.168.1.254"
/// - Single IP: "192.168.1.1"
pub fn parse_range(input: &str) -> Result<Vec<Ipv4Addr>, String> {
    let input = input.trim();

    if input.contains('/') {
        parse_cidr(input)
    } else if input.contains('-') {
        parse_dash(input)
    } else {
        input.parse::<Ipv4Addr>()
            .map(|ip| vec![ip])
            .map_err(|e| format!("Invalid IP: {}", e))
    }
}

fn parse_cidr(input: &str) -> Result<Vec<Ipv4Addr>, String> {
    let parts: Vec<&str> = input.split('/').collect();
    if parts.len() != 2 {
        return Err("Invalid CIDR notation".to_string());
    }

    let base_ip: Ipv4Addr = parts[0].parse().map_err(|e| format!("Invalid IP: {}", e))?;
    let prefix: u32 = parts[1].parse().map_err(|e| format!("Invalid prefix: {}", e))?;

    if prefix > 32 {
        return Err("Prefix must be 0-32".to_string());
    }

    let ip_u32 = u32::from(base_ip);
    let mask = if prefix == 0 { 0 } else { !0u32 << (32 - prefix) };
    let network = ip_u32 & mask;
    let broadcast = network | !mask;

    let mut ips = Vec::new();
    // Skip network and broadcast for /24 and smaller
    let start = if prefix >= 24 { network + 1 } else { network };
    let end = if prefix >= 24 { broadcast } else { broadcast + 1 };

    for addr in start..end {
        ips.push(Ipv4Addr::from(addr));
    }

    Ok(ips)
}

fn parse_dash(input: &str) -> Result<Vec<Ipv4Addr>, String> {
    let parts: Vec<&str> = input.split('-').collect();
    if parts.len() != 2 {
        return Err("Invalid range format".to_string());
    }

    let start_ip: Ipv4Addr = parts[0].parse().map_err(|e| format!("Invalid start IP: {}", e))?;

    // Check if the second part is just a number (last octet) or full IP
    if let Ok(end_octet) = parts[1].trim().parse::<u8>() {
        let octets = start_ip.octets();
        let start_last = octets[3];
        let mut ips = Vec::new();
        for i in start_last..=end_octet {
            ips.push(Ipv4Addr::new(octets[0], octets[1], octets[2], i));
        }
        Ok(ips)
    } else if let Ok(end_ip) = parts[1].trim().parse::<Ipv4Addr>() {
        let start = u32::from(start_ip);
        let end = u32::from(end_ip);
        if end < start {
            return Err("End IP must be >= start IP".to_string());
        }
        Ok((start..=end).map(Ipv4Addr::from).collect())
    } else {
        Err("Invalid range end".to_string())
    }
}
