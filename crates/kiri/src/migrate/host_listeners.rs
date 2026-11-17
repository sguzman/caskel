//! Read-only observation of ports bound in the CLI's own Linux network
//! namespace. This is a point-in-time /proc snapshot, not a socket probe,
//! an ownership lookup, or evidence that a future cutover is safe.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Listener {
    pub protocol: &'static str,
    pub address: String,
    pub port: u16,
}

/// /proc/net stores IPv4 bytes in native little-endian order; IPv6
/// addresses are four little-endian u32 words.
fn decode_address(hex: &str, ipv6: bool) -> Option<String> {
    if !ipv6 {
        if hex.len() != 8 { return None; }
        let raw = u32::from_str_radix(hex, 16).ok()?;
        return Some(Ipv4Addr::from(raw.to_le_bytes()).to_string());
    }
    if hex.len() != 32 { return None; }
    let mut bytes = [0u8; 16];
    for i in 0..4 {
        let chunk = u32::from_str_radix(&hex[i * 8..i * 8 + 8], 16).ok()?;
        bytes[i * 4..i * 4 + 4].copy_from_slice(&chunk.to_le_bytes());
    }
    Some(Ipv6Addr::from(bytes).to_string())
}

/// TCP: only LISTEN (0A). UDP: any bound socket, including a connected
/// datagram socket; it can still prevent another owner from binding.
fn parse_row(line: &str, protocol: &'static str, ipv6: bool) -> Option<Listener> {
    let mut fields = line.split_whitespace();
    fields.next()?;
    let local = fields.next()?;
    fields.next()?;
    let state = fields.next()?;
    if protocol == "tcp" && state != "0A" { return None; }
    let (address, port) = local.split_once(':')?;
    let port = u16::from_str_radix(port, 16).ok()?;
    if port == 0 { return None; }
    Some(Listener {
        protocol,
        address: decode_address(address, ipv6)?,
        port,
    })
}

/// Never bind/connect/stop services. Fail closed if a required /proc view
/// cannot be read, rather than presenting incomplete data as a clean host.
pub(super) fn snapshot() -> io::Result<Vec<Listener>> {
    let paths = [
        ("/proc/net/tcp", "tcp", false),
        ("/proc/net/tcp6", "tcp", true),
        ("/proc/net/udp", "udp", false),
        ("/proc/net/udp6", "udp", true),
    ];
    let mut seen = BTreeSet::new();
    for (path, protocol, ipv6) in paths {
        let content = fs::read_to_string(path).map_err(|e| {
            io::Error::new(e.kind(), format!("cannot read local network namespace {path}: {e}"))
        })?;
        for line in content.lines().skip(1) {
            if let Some(Listener { protocol, address, port }) = parse_row(line, protocol, ipv6) {
                seen.insert((protocol, port, address));
            }
        }
    }
    Ok(seen.into_iter().map(|(protocol, port, address)| Listener {
        protocol, address, port,
    }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, UdpSocket};

    #[test]
    fn parses_ipv4_ipv6_and_ignores_nonlisteners() {
        let tcp4 = "0: 0100007F:494F 00000000:0000 0A 00000000:00000000 00:00000000 00000000 0 0 0";
        let entry = parse_row(tcp4, "tcp", false).unwrap();
        assert_eq!((entry.address.as_str(), entry.port), ("127.0.0.1", 18767));
        let tcp6 = "0: 00000000000000000000000001000000:0050 00000000000000000000000000000000:0000 0A";
        assert_eq!(parse_row(tcp6, "tcp", true).unwrap().address, "::1");
        assert!(parse_row(&tcp4.replace(" 0A ", " 01 "), "tcp", false).is_none());
        assert!(parse_row(&tcp4.replace(":494F", ":0000"), "tcp", false).is_none());
        assert_eq!(parse_row(&tcp4.replace(" 0A ", " 07 "), "udp", false).unwrap().port, 18767);
        assert!(parse_row("malformed", "tcp", false).is_none());
    }

    #[test]
    fn reads_real_local_tcp_and_udp_sockets_without_modifying_them() {
        let tcp = TcpListener::bind("127.0.0.1:0").unwrap();
        let udp = UdpSocket::bind("127.0.0.1:0").unwrap();
        let seen = snapshot().unwrap();
        assert!(seen.iter().any(|s| s.protocol == "tcp" && s.address == "127.0.0.1"
            && s.port == tcp.local_addr().unwrap().port()));
        assert!(seen.iter().any(|s| s.protocol == "udp" && s.address == "127.0.0.1"
            && s.port == udp.local_addr().unwrap().port()));
    }
}
