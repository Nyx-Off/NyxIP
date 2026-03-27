use std::net::Ipv4Addr;
use std::time::Instant;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Ping a host using the Windows ICMP API via IcmpSendEcho.
/// Falls back to system ping command if the API call fails.
pub async fn ping(ip: Ipv4Addr, timeout_ms: u32) -> Option<f64> {
    if let Some(rtt) = ping_icmp_api(ip, timeout_ms) {
        return Some(rtt);
    }
    ping_command(ip, timeout_ms).await
}

/// Native Windows ICMP ping using IcmpSendEcho from iphlpapi.
fn ping_icmp_api(ip: Ipv4Addr, timeout_ms: u32) -> Option<f64> {
    unsafe {
        use windows::Win32::NetworkManagement::IpHelper::{
            IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho,
        };

        let handle = IcmpCreateFile().ok()?;
        let ip_addr = u32::from_ne_bytes(ip.octets());

        let send_data = [0u8; 32];
        let send_ptr = send_data.as_ptr() as *const core::ffi::c_void;
        let send_size = send_data.len() as u16;

        // Reply buffer: ICMP_ECHO_REPLY (28 bytes) + data + 8 extra
        let reply_size: u32 = 28 + send_data.len() as u32 + 8;
        let mut reply_buf = vec![0u8; reply_size as usize];
        let reply_ptr = reply_buf.as_mut_ptr() as *mut core::ffi::c_void;

        let result = IcmpSendEcho(
            handle,
            ip_addr,
            send_ptr,
            send_size,
            None,
            reply_ptr,
            reply_size,
            timeout_ms,
        );

        let _ = IcmpCloseHandle(handle);

        if result > 0 {
            // ICMP_ECHO_REPLY layout:
            //   offset 0: Address (u32)
            //   offset 4: Status  (u32)
            //   offset 8: RoundTripTime (u32)
            let status = u32::from_ne_bytes([
                reply_buf[4], reply_buf[5], reply_buf[6], reply_buf[7],
            ]);
            if status == 0 {
                let rtt = u32::from_ne_bytes([
                    reply_buf[8], reply_buf[9], reply_buf[10], reply_buf[11],
                ]);
                Some(rtt as f64)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Fallback ping using the system `ping.exe` command.
async fn ping_command(ip: Ipv4Addr, timeout_ms: u32) -> Option<f64> {
    let start = Instant::now();

    let mut cmd = tokio::process::Command::new("ping");
    cmd.args(["-n", "1", "-w", &timeout_ms.to_string(), &ip.to_string()]);

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output().await.ok()?;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    if !stdout.contains("TTL=") && !stdout.contains("ttl=") {
        return None;
    }

    for prefix in &["temps=", "time=", "temps<", "time<"] {
        if let Some(pos) = stdout.find(prefix) {
            let rest = &stdout[pos + prefix.len()..];
            if let Some(end) = rest.find("ms") {
                let rtt_str = rest[..end].trim();
                if let Ok(rtt) = rtt_str.parse::<f64>() {
                    return Some(rtt);
                }
            }
        }
    }

    Some(elapsed)
}
