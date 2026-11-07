# Changelog

All notable changes to `qr-drop` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-07

### Added
- **Bidirectional Peer-to-Peer Local Network Sharing**:
  - `qr-drop <path>` / `qr-drop send <path>`: Instant file or directory sending.
  - Automatic on-the-fly streaming zip archive creation for entire directory trees.
  - `qr-drop receive [output_dir]`: Mobile web UI for uploading files, camera photos, and text notes directly onto the host PC.
  - `qr-drop text <message>`: Ephemeral text snippet sharing with mobile clipboard one-tap copy button.
- **Ultra-Compact Unicode Terminal QR Codes**:
  - 2-in-1 vertical half-block (`▀`, `▄`, `█`, ` `) module rendering for perfect square aspect ratio in terminals.
  - Quiet zone padding control (`--margin`) and dark/light terminal inversion (`--invert`).
- **Zero-Packet LAN IP Discovery**:
  - Outbound adapter IP resolution using kernel UDP socket routing table query (`8.8.8.8:80`).
- **Embedded Ephemeral HTTP Server**:
  - Fully inlined, self-contained modern dark-mode responsive HTML5 templates (no external CDNs or internet access required).
  - Secure randomized endpoint tokens to protect against local subnet probing.
  - Single-use auto-shutdown upon transfer completion, with optional `--keep-alive` (`-k`) flag.
- **Robustness & Strict Lints**:
  - 100% panic-free, unwrap-free, and indexing-safe codebase with `forbid(unsafe_code)`.
  - Comprehensive end-to-end integration and unit tests covering download, zip streaming, upload, notes, IP detection, and QR code rendering.
