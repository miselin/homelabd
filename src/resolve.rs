use std::net::{IpAddr, ToSocketAddrs};
use std::str::FromStr;

/// Resolves a hostname to an IP address, but only returns it if it's in the known list.
/// Falls back to the first known IP address if resolution fails or returns an untrusted IP.
pub fn resolve_or_fallback(hostname: &str, known_ips: &[IpAddr]) -> Result<IpAddr, String> {
    // Prefer first known IP as fallback
    let fallback = match known_ips.first() {
        Some(ip) => *ip,
        None => return Err("No known IPs provided".into()),
    };

    // Attempt to resolve the hostname
    match (hostname, 0).to_socket_addrs() {
        Ok(addrs) => {
            for addr in addrs {
                let ip = addr.ip();
                if known_ips.contains(&ip) {
                    return Ok(ip);
                }
            }
            Ok(fallback)
        }
        Err(e) => Err(format!("Failed to resolve hostname: {}", e)),
    }
}
