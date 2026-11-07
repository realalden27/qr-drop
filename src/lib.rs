//! # qr-drop
//!
//! Instant PC <-> Phone file, folder, and text sharing via terminal QR code in pure Rust.

pub mod config;
pub mod error;
pub mod ip;
pub mod qr;
pub mod routes;
pub mod server;

pub use config::{Cli, Commands};
pub use error::{QrDropError, Result};
pub use ip::{detect_local_ip, resolve_ip};
pub use qr::{QrOptions, render_terminal_qr};
pub use server::{AppState, Payload, ServerConfig, create_router, start_server};
