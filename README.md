# qr-drop

[![Crates.io](https://img.shields.io/crates/v/qr-drop.svg)](https://crates.io/crates/qr-drop)
[![Docs.rs](https://docs.rs/qr-drop/badge.svg)](https://docs.rs/qr-drop)
[![CI](https://github.com/realalden27/qr-drop/actions/workflows/ci.yml/badge.svg)](https://github.com/realalden27/qr-drop/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

**Instant PC ↔ Phone file, folder, and text sharing via terminal QR code in pure Rust.**

No accounts, no internet uploads, no third-party apps required. Just point your phone camera at your terminal and transfer files directly over your local Wi-Fi.

---

## Features

- ⚡ **Zero-Configuration**: No app install needed on your phone—uses the built-in iOS Camera or Android Lens/Chrome.
- 🖥️ **Ultra-Compact Half-Block QR**: Uses 2-in-1 Unicode half-blocks (`▀`, `▄`, `█`, ` `) so the entire QR code fits cleanly in standard terminal windows without vertical scrolling or distorted aspect ratios.
- 📁 **Files & Folder Archiving**: Send a single file or stream an entire folder zipped on-the-fly.
- 📥 **Bidirectional Transfers**:
  - `qr-drop <file>`: Send a file or folder from your computer to your phone.
  - `qr-drop receive`: Receive photos, documents, and notes from your phone onto your computer.
  - `qr-drop text <message>`: Instantly copy text, URLs, or clipboard snippets onto your phone.
- 🔒 **Local & Ephemeral**: Direct peer-to-peer Wi-Fi transfer. Shuts down automatically after a single transfer (or pass `-k` / `--keep-alive` to keep running).
- 🛡️ **100% Panic-Free & Pure Rust**: Zero unsafe code (`forbid(unsafe_code)`), strictly linted against panics, indexing bounds violations, and unwraps.

---

## Installation

### Via Cargo
```bash
cargo install qr-drop
```

### Pre-built Binaries
Download the latest pre-compiled binary for Windows, macOS, or Linux from the [GitHub Releases](https://github.com/realalden27/qr-drop/releases) page.

---

## Quick Start

### 1. Send a File to Your Phone
```bash
qr-drop report.pdf
```
or explicitly:
```bash
qr-drop send report.pdf
```
A terminal QR code will appear. Scan it with your phone's camera, tap the notification, and your file downloads instantly.

### 2. Send an Entire Folder
```bash
qr-drop ./vacation_photos
```
`qr-drop` compresses the folder on-the-fly into a zip archive and streams it directly to your phone.

### 3. Receive Files and Photos from Your Phone
```bash
qr-drop receive
```
Scan the QR code to open an upload page on your phone. You can pick photos, take new photos directly with your camera, select documents, or type a text note. Received files are saved in the `received/` folder (or custom folder: `qr-drop receive ./downloads`).

### 4. Share a Text Snippet or Link
```bash
qr-drop text "https://github.com/realalden27/qr-drop"
```
Or pipe from stdin:
```bash
git log -1 | qr-drop text -
```
The phone displays a clean mobile card with a **"Copy to Clipboard"** button.

---

## Command-Line Options

```
Usage: qr-drop [OPTIONS] [PATH] [COMMAND]

Commands:
  send     Share a file or folder with your phone [default]
  receive  Receive files or notes from your phone
  text     Share a text snippet, URL, or note
  help     Print this message or the help of the given subcommand(s)

Arguments:
  [PATH]  Shortcut to send a file or folder without typing 'send'

Options:
  -p, --port <PORT>        Specific port to listen on (default: 0 for random free port)
  -H, --ip <IP>            Specific IP address to advertise in the QR code (default: auto-detected LAN IP)
  -k, --keep-alive         Keep server running after first transfer (disables single-use auto-shutdown)
      --invert             Invert QR code colors for light-themed terminal backgrounds
      --margin <MARGIN>    Quiet zone margin around the QR code in terminal characters [default: 2]
  -h, --help               Print help
  -V, --version            Print version
```

---

## How It Works

1. **Zero-Packet LAN IP Resolution**: `qr-drop` queries the operating system routing table via a connectionless UDP socket bind (`8.8.8.8:80`) to discover your preferred outbound LAN IP without transmitting any packets over the internet.
2. **2-in-1 Half-Block Unicode Rendering**: Terminal characters are roughly twice as tall as they are wide (~1:2 aspect ratio). `qr-drop` packs two vertical QR matrix pixels into single Unicode half-block characters (`▀`, `▄`, `█`, ` `), preserving a square aspect ratio and halving the required line count.
3. **Embedded Ephemeral HTTP Server**: An asynchronous [Axum](https://github.com/tokio-rs/axum) server is mounted with randomized endpoint tokens to prevent local subnet scraping. All CSS and JavaScript are inlined without any external CDN dependencies.
4. **Instant Graceful Shutdown**: Unless `--keep-alive` is specified, the server terminates after serving the file or receiving the upload, freeing the port immediately.

---

## Security

- **Path Traversal Protection**: Uploaded file names are strictly sanitized to prevent directory traversal attacks (`../` or absolute path escapes).
- **Ephemeral Tokenized URLs**: Transfer URLs are protected with random 8-character tokens (`/d/x7k9p2m1`) so other devices on the same Wi-Fi network cannot guess the download/upload links.
- **Local Network Only**: All communication happens strictly within your local Wi-Fi / subnet; no telemetry or external cloud servers are involved.

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
