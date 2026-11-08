# Changelog

All notable changes to `qr-drop` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5] - 2026-11-08

### Fixed
- **Upload Multipart Body Limit**: Disabled Axum's default 2 MiB request body limit (`DefaultBodyLimit::disable()`) on the receive route. Uploading files or photos larger than 2 MiB previously failed with `Chunk read error parsing 'multipart/form-data' request`. Large file uploads now stream directly to disk without hitting artificial payload limits.
- **Incomplete Upload File Cleanup**: Clean up partially written files from disk if a transfer fails or disconnects mid-stream.

### Added
- **Upload Summary Display**: Web landing form now reflects the server's transfer response summary upon completion.
- **Large File E2E Test**: Added regression test verifying large (25 MiB+) streaming file transfers.

### Changed
- **Modern Web Interface Redesign**: Overhauled all three web templates (`download.html`, `text.html`, `upload.html`) into a clean, professional monochromatic UI. Added automatic file extension badges, interactive upload queue with individual file removal, live uploaded-bytes progress counter, instant URL link detection for shared snippets, and mobile touch polish with zero external CDN dependencies.

## [0.1.4] - 2026-11-07

### Fixed
- **Release CI Runner Queue**: Migrated macOS runner for `x86_64-apple-darwin` from deprecated `macos-13` to `macos-latest` (Apple Silicon with Universal cross-compilation), eliminating queue hang in GitHub Actions release matrix.

### Added
- **Dual crates.io and GitHub Update Resolution**: `qr-drop update` now simultaneously inspects the crates.io API and GitHub Releases API.
- **Cargo-Aware Self-Update**: Detects if `qr-drop` was installed via Cargo (`.cargo/bin`) and automatically executes `cargo install qr-drop` to perform the update.

## [0.1.3] - 2026-11-07

### Added
- **Auto-Upgrade on Newer Binary Run**: When running a newer `qr-drop` binary outside the Programs directory, it automatically detects the version difference, updates the installed Programs folder binary, and prints an upgrade notice.
- **In-App Self-Updater (`qr-drop update`)**: Built-in `update` subcommand that queries the GitHub Releases API for the latest release, downloads the platform-specific pre-compiled archive, and performs an in-place self-replacement.

### Changed
- Replaced web template icons with clean vector SVGs and ensured 100% emoji-free terminal output.

## [0.1.2] - 2026-11-07

### Changed
- Automatic install logic now checks if the target binary already exists in user programs, running silently on subsequent executions.
- Removed all emojis from console output and documentation.

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
