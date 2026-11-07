# Changelog

All notable changes to `qr-drop` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-11-07

### Added
- **Automatic Self-Path Registration on First Run**:
  - Automatically detects whether the running executable's directory is registered in the user's `PATH`.
  - On Windows, automatically adds the folder to the user's permanent `PATH` environment variable on the very first run.
  - On Unix / macOS, automatically symlinks into `~/.local/bin/qr-drop` if available.
  - Includes guard rails to ignore temporary directories (`std::env::temp_dir()`) and unit test executions.
- **Automated Multi-Platform Release CI**:
  - GitHub Actions matrix workflow to build and release pre-compiled stripped binaries for Windows (`.zip`), macOS Apple Silicon & Intel (`.tar.gz`), and Linux (`.tar.gz`).

## [0.1.0] - 2026-11-07

### Added
- **Bidirectional Peer-to-Peer Local Network Sharing**:
  - `qr-drop <path>` / `qr-drop send <path>`: Instant file or directory sending.
  - Automatic on-the-fly streaming zip archive creation for entire directory trees.
  - `qr-drop receive [output_dir]`: Mobile web UI for uploading files, camera photos, and text notes directly onto the host PC. Defaults to the gitignored `received/` directory.
  - Native OS desktop notification on transfer completion and automatic file manager opening focused on the received destination (with `--no-open` and `--no-notify` toggles).
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
