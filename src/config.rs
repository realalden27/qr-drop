use std::io::Read as _;
use std::net::IpAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::error::{QrDropError, Result};
use crate::qr::QrOptions;
use crate::server::{Payload, ServerConfig};

/// Instant PC <-> Phone sharing via terminal QR code in pure Rust.
#[derive(Parser, Debug)]
#[command(
    name = "qr-drop",
    version,
    about = "Instant PC <-> Phone sharing via terminal QR code in pure Rust.",
    long_about = "A zero-configuration, cross-platform CLI for peer-to-peer file, folder, and text sharing between PC and mobile devices over your local network using terminal QR codes."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Shortcut to send a file or folder without typing 'send'
    #[arg(value_name = "PATH")]
    pub direct_path: Option<PathBuf>,

    /// Specific port to listen on (0 for random free port)
    #[arg(short = 'p', long, default_value = "0", global = true)]
    pub port: u16,

    /// Specific IP address to advertise in the QR code (defaults to auto-detected LAN IP)
    #[arg(short = 'H', long, global = true)]
    pub ip: Option<IpAddr>,

    /// Keep server running after first transfer (disables single-use auto-shutdown)
    #[arg(short = 'k', long, global = true)]
    pub keep_alive: bool,

    /// Invert QR code colors for light-themed terminal backgrounds
    #[arg(long, global = true)]
    pub invert: bool,

    /// Quiet zone margin around the QR code in terminal characters
    #[arg(long, default_value = "2", global = true)]
    pub margin: usize,
}

/// Available CLI subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Share a file or folder with your phone
    Send {
        /// Path to the file or directory to send
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
    /// Receive files or notes from your phone
    Receive {
        /// Destination folder to save received files (default: "received" folder)
        #[arg(value_name = "OUTPUT_DIR", default_value = "received")]
        output: PathBuf,

        /// Automatically open destination folder in file manager (default: true)
        #[arg(short = 'o', long)]
        open: bool,

        /// Do not automatically open the destination folder after receiving files
        #[arg(long)]
        no_open: bool,

        /// Do not show a desktop notification after receiving files
        #[arg(long)]
        no_notify: bool,
    },
    /// Share a text snippet, URL, or note
    Text {
        /// Text content to share (use '-' or omit to read from stdin)
        #[arg(value_name = "CONTENT")]
        content: Option<String>,
    },
}

impl Cli {
    /// Resolves the CLI input into a `(Payload, ServerConfig)` tuple ready for execution.
    pub fn resolve(self) -> Result<(Payload, ServerConfig)> {
        let server_config = ServerConfig {
            ip: self.ip,
            port: self.port,
            keep_alive: self.keep_alive,
            qr_options: QrOptions {
                invert: self.invert,
                margin: self.margin,
            },
        };

        let payload = match (self.command, self.direct_path) {
            (Some(Commands::Send { path }), _) | (None, Some(path)) => {
                let canonical = match path.canonicalize() {
                    Ok(p) => p,
                    Err(e) => {
                        return Err(QrDropError::invalid_path(format!(
                            "Failed to resolve path '{}': {e}",
                            path.display()
                        )));
                    },
                };

                let is_dir = canonical.is_dir();
                if !is_dir && !canonical.is_file() {
                    return Err(QrDropError::invalid_path(format!(
                        "Path '{}' is neither a file nor a directory",
                        canonical.display()
                    )));
                }

                Payload::SendFile {
                    path: canonical,
                    is_dir,
                }
            },
            (
                Some(Commands::Receive {
                    output,
                    open: _,
                    no_open,
                    no_notify,
                }),
                _,
            ) => {
                let out = if output.is_relative() {
                    match std::env::current_dir() {
                        Ok(cd) => cd.join(output),
                        Err(_) => output,
                    }
                } else {
                    output
                };
                Payload::Receive {
                    output_dir: out,
                    open: !no_open,
                    notify: !no_notify,
                }
            },
            (Some(Commands::Text { content }), _) => {
                let text = match content {
                    Some(s) if s != "-" => s,
                    _ => {
                        let mut buffer = String::new();
                        std::io::stdin()
                            .read_to_string(&mut buffer)
                            .map_err(QrDropError::Io)?;
                        buffer
                    },
                };

                if text.trim().is_empty() {
                    return Err(QrDropError::invalid_path("Text snippet cannot be empty"));
                }

                Payload::Text { content: text }
            },
            (None, None) => {
                return Err(QrDropError::invalid_path(
                    "No file, text, or subcommand specified. Run `qr-drop --help` for usage details.",
                ));
            },
        };

        Ok((payload, server_config))
    }
}
