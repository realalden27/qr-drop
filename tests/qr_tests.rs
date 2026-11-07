#![allow(clippy::unwrap_used, clippy::expect_used)]

use qr_drop::qr::{QrOptions, render_terminal_qr};

#[test]
fn test_render_qr_basic() {
    let url = "http://192.168.1.50:8080/d/test123";
    let qr = render_terminal_qr(url, QrOptions::default()).expect("QR code rendering must succeed");

    assert!(!qr.is_empty(), "QR code string must not be empty");
    assert!(qr.contains('\n'), "QR code must have multiple lines");

    // Must contain Unicode block characters
    let has_blocks = qr
        .chars()
        .any(|c| c == '▀' || c == '▄' || c == '█' || c == ' ');
    assert!(has_blocks, "QR code must contain Unicode block characters");
}

#[test]
fn test_render_qr_invert() {
    let url = "http://localhost:3000/u/token";
    let options = QrOptions {
        invert: true,
        margin: 1,
    };
    let qr = render_terminal_qr(url, options).expect("Inverted QR code rendering must succeed");
    assert!(!qr.is_empty());
}

#[test]
fn test_render_qr_zero_margin() {
    let url = "test";
    let options = QrOptions {
        invert: false,
        margin: 0,
    };
    let qr = render_terminal_qr(url, options).expect("Zero margin QR code rendering must succeed");
    assert!(!qr.is_empty());
}
