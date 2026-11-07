use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use axum::Router;
use axum::routing::get;
use rand::Rng as _;
use rand::distributions::Alphanumeric;
use tokio::sync::mpsc::{Sender, channel};

use crate::error::{QrDropError, Result};
use crate::ip::resolve_ip;
use crate::qr::{QrOptions, render_terminal_qr};
use crate::routes;

/// The payload type being served by `qr-drop`.
#[derive(Debug, Clone)]
pub enum Payload {
    /// Serving a file or entire directory for download.
    SendFile {
        /// Target path on local disk.
        path: PathBuf,
        /// Whether the path represents a directory.
        is_dir: bool,
    },
    /// Hosting an upload form to receive files or notes.
    Receive {
        /// Output directory to store received files.
        output_dir: PathBuf,
        /// Whether to automatically open the destination folder in file manager upon receiving files.
        open: bool,
        /// Whether to display a desktop notification upon receiving files.
        notify: bool,
    },
    /// Displaying a text snippet for easy copying on mobile.
    Text {
        /// The text snippet content.
        content: String,
    },
}

/// Shared application state for Axum request handlers.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Ephemeral random URL token.
    pub token: String,
    /// Active sharing payload.
    pub payload: Payload,
    /// Channel sender to trigger single-use server shutdown.
    pub shutdown_tx: Option<Sender<()>>,
}

/// Server configuration options.
#[derive(Debug, Clone, Default)]
pub struct ServerConfig {
    /// Optional IP address override.
    pub ip: Option<IpAddr>,
    /// Port to listen on (0 for automatic ephemeral port).
    pub port: u16,
    /// If true, server keeps running after transfer until Ctrl+C.
    pub keep_alive: bool,
    /// QR code rendering parameters.
    pub qr_options: QrOptions,
}

/// Generates a short random alphanumeric token for the URL endpoint.
#[must_use]
pub fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}

/// Creates the configured Axum router for the given application state.
pub fn create_router(state: AppState) -> Router {
    let token = state.token.clone();
    match &state.payload {
        Payload::SendFile { .. } => Router::new()
            .route(&format!("/d/{token}"), get(routes::download::page_handler))
            .route(
                &format!("/d/{token}/raw"),
                get(routes::download::raw_handler),
            ),
        Payload::Receive { .. } => Router::new().route(
            &format!("/u/{token}"),
            get(routes::upload::page_handler).post(routes::upload::upload_handler),
        ),
        Payload::Text { .. } => {
            Router::new().route(&format!("/t/{token}"), get(routes::text::page_handler))
        },
    }
    .layer(tower_http::cors::CorsLayer::permissive())
    .with_state(state)
}

/// Starts the `qr-drop` embedded Axum server, displays the terminal QR code, and awaits transfer completion or Ctrl+C.
pub async fn start_server(payload: Payload, config: ServerConfig) -> Result<()> {
    let token = generate_token();
    let (shutdown_tx, mut shutdown_rx) = channel::<()>(1);

    let state = AppState {
        token: token.clone(),
        payload: payload.clone(),
        shutdown_tx: if config.keep_alive {
            None
        } else {
            Some(shutdown_tx)
        },
    };

    let (route_prefix, summary_line) = match &payload {
        Payload::SendFile { path, is_dir } => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "file".to_string());
            let size = routes::download::calculate_size(path);
            let kind = if *is_dir { "Folder (Zip)" } else { "File" };
            (
                "d",
                format!(
                    "  Sharing {kind}: {name} ({})",
                    routes::download::format_filesize(size)
                ),
            )
        },
        Payload::Receive { output_dir, .. } => (
            "u",
            format!("  Ready to receive into: {}", output_dir.display()),
        ),
        Payload::Text { content } => {
            let preview = if content.len() > 40 {
                let mut s = content.chars().take(37).collect::<String>();
                s.push_str("...");
                s
            } else {
                content.clone()
            };
            ("t", format!("  Sharing text: \"{preview}\""))
        },
    };

    let router = create_router(state);

    let bind_addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .map_err(|e| QrDropError::server(format!("Failed to bind to {bind_addr}: {e}")))?;

    let actual_port = listener
        .local_addr()
        .map_err(|e| QrDropError::server(format!("Failed to retrieve assigned port: {e}")))?
        .port();

    let lan_ip = resolve_ip(config.ip)?;
    let target_url = format!("http://{lan_ip}:{actual_port}/{route_prefix}/{token}");

    let qr_str = render_terminal_qr(&target_url, config.qr_options)?;

    println!();
    println!("{qr_str}");
    println!("  Scan QR code above, or open in mobile browser:");
    println!("     {target_url}");
    println!();
    println!("{summary_line}");

    if config.keep_alive {
        println!("  Server running in persistent mode. Press Ctrl+C to stop.");
    } else {
        println!("  Server will automatically shut down after transfer completes.");
        println!("     (Use --keep-alive / -k to keep the server running)");
    }
    println!();

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    println!("\n  Interrupted (Ctrl+C). Server stopped.");
                }
                _ = shutdown_rx.recv() => {
                    println!("\n  Transfer complete. Shutting down server.");
                }
            }
        })
        .await
        .map_err(|e| QrDropError::server(format!("Server execution error: {e}")))?;

    Ok(())
}
