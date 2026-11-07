use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use crate::error::{QrDropError, Result};

/// Detects the local IPv4 address used for outbound LAN routing.
///
/// Uses an ephemeral UDP socket connected to a public target (`8.8.8.8:80`).
/// Because UDP is connectionless, no packets are actually transmitted across
/// the wire; the operating system kernel simply inspects its local routing
/// table to bind the socket to the preferred outbound network adapter.
pub fn detect_local_ip() -> Result<IpAddr> {
    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0)))
        .map_err(|e| QrDropError::network(format!("Failed to bind local UDP socket: {e}")))?;

    socket
        .connect("8.8.8.8:80")
        .map_err(|e| QrDropError::network(format!("Failed to route to outbound gateway: {e}")))?;

    let local_addr = socket
        .local_addr()
        .map_err(|e| QrDropError::network(format!("Failed to retrieve socket address: {e}")))?;

    let ip = local_addr.ip();
    if ip.is_unspecified() {
        return Err(QrDropError::network(
            "Resolved unspecified local IP (0.0.0.0)",
        ));
    }

    Ok(ip)
}

/// Resolves the IP address to use, preferring an explicit override if provided.
pub fn resolve_ip(override_ip: Option<IpAddr>) -> Result<IpAddr> {
    match override_ip {
        Some(ip) => Ok(ip),
        None => detect_local_ip().or(Ok(IpAddr::V4(Ipv4Addr::LOCALHOST))),
    }
}
