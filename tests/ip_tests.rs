#![allow(clippy::unwrap_used, clippy::expect_used)]

use qr_drop::ip::{detect_local_ip, resolve_ip};
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn test_detect_local_ip() {
    // In any standard network environment with internet/LAN, this resolves the outbound adapter IP.
    // If running in an isolated environment, it may return an error, which resolve_ip handles.
    let res = detect_local_ip();
    if let Ok(ip) = res {
        assert!(!ip.is_unspecified(), "Local IP must not be 0.0.0.0");
    }
}

#[test]
fn test_resolve_ip_with_override() {
    let custom = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 42));
    let resolved = resolve_ip(Some(custom)).expect("Override IP must always resolve");
    assert_eq!(resolved, custom);
}

#[test]
fn test_resolve_ip_fallback() {
    let resolved = resolve_ip(None).expect("IP resolution should succeed with fallback");
    assert!(!resolved.is_unspecified());
}
