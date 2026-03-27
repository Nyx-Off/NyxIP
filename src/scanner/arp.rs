use std::net::Ipv4Addr;

/// Resolve the MAC address of a host using the Windows SendARP API.
/// Returns the MAC as a formatted string (e.g. "AA:BB:CC:DD:EE:FF"), or None.
pub fn get_mac(ip: Ipv4Addr) -> Option<String> {
    unsafe {
        use windows::Win32::NetworkManagement::IpHelper::SendARP;

        let dest = u32::from_ne_bytes(ip.octets());
        let mut mac_buf = [0u8; 8]; // 6 bytes needed, 8 for alignment
        let mut mac_len: u32 = 6;

        let ret = SendARP(
            dest,
            0, // source IP — 0 lets the system choose
            &mut mac_buf as *mut _ as *mut core::ffi::c_void,
            &mut mac_len,
        );

        if ret == 0 && mac_len == 6 {
            let mac = format!(
                "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                mac_buf[0], mac_buf[1], mac_buf[2],
                mac_buf[3], mac_buf[4], mac_buf[5],
            );
            // Ignore the broadcast / empty MAC
            if mac == "00:00:00:00:00:00" || mac == "FF:FF:FF:FF:FF:FF" {
                None
            } else {
                Some(mac)
            }
        } else {
            None
        }
    }
}
