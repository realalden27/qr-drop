use std::path::{Path, PathBuf};
use std::time::SystemTime;

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use tokio::io::AsyncWriteExt as _;

use crate::server::{AppState, Payload};

const UPLOAD_TEMPLATE: &str = include_str!("../templates/upload.html");

/// Sanitizes a user-provided filename to prevent directory traversal and special character exploits.
#[must_use]
pub fn sanitize_filename(name: &str) -> String {
    let raw_name = Path::new(name)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "upload.bin".to_string());

    let sanitized: String = raw_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() || sanitized.chars().all(|c| c == '.' || c == ' ') {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("upload_{ts}.bin")
    } else {
        sanitized
    }
}

/// Computes an unused destination path to avoid silently overwriting existing files.
#[must_use]
pub fn resolve_unique_path(dir: &Path, filename: &str) -> PathBuf {
    let initial_path = dir.join(filename);
    if !initial_path.exists() {
        return initial_path;
    }

    let p = Path::new(filename);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = p.extension().and_then(|s| s.to_str());

    for counter in 1..=9999 {
        let candidate_name = match ext {
            Some(extension) => format!("{stem}_{counter}.{extension}"),
            None => format!("{stem}_{counter}"),
        };
        let candidate = dir.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    dir.join(format!("{filename}_{ts}"))
}

/// Handler for the landing upload form page.
pub async fn page_handler(State(state): State<AppState>) -> Response {
    if !matches!(&state.payload, Payload::Receive { .. }) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let upload_url = format!("/u/{}", state.token);
    let html = UPLOAD_TEMPLATE.replace("{{UPLOAD_URL}}", &upload_url);
    Html(html).into_response()
}

/// Handler for incoming multipart form uploads (files and text).
pub async fn upload_handler(State(state): State<AppState>, mut multipart: Multipart) -> Response {
    let (output_dir, should_open, should_notify) = match &state.payload {
        Payload::Receive {
            output_dir,
            open,
            notify,
        } => (output_dir.clone(), *open, *notify),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    if !output_dir.exists() {
        match tokio::fs::create_dir_all(&output_dir).await {
            Ok(()) => {},
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create destination folder: {e}"),
                )
                    .into_response();
            },
        }
    }

    let mut saved_files_count: usize = 0;
    let mut total_bytes: u64 = 0;
    let mut saved_text_count: usize = 0;

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Multipart parse error: {e}"),
                )
                    .into_response();
            },
        };

        let field_name = field.name().map(|s| s.to_string());

        if field_name.as_deref() == Some("files") {
            let original_name = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "upload.bin".to_string());

            let safe_name = sanitize_filename(&original_name);
            let target_path = resolve_unique_path(&output_dir, &safe_name);

            let mut destination = match tokio::fs::File::create(&target_path).await {
                Ok(file) => file,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to create file {}: {e}", target_path.display()),
                    )
                        .into_response();
                },
            };

            let mut stream = field;
            let mut file_bytes: u64 = 0;

            loop {
                match stream.chunk().await {
                    Ok(Some(chunk)) => {
                        if let Err(e) = destination.write_all(&chunk).await {
                            drop(destination);
                            let _ = tokio::fs::remove_file(&target_path).await;
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed writing chunk to disk: {e}"),
                            )
                                .into_response();
                        }
                        file_bytes = file_bytes.saturating_add(chunk.len() as u64);
                    },
                    Ok(None) => break,
                    Err(e) => {
                        drop(destination);
                        let _ = tokio::fs::remove_file(&target_path).await;
                        return (StatusCode::BAD_REQUEST, format!("Chunk read error: {e}"))
                            .into_response();
                    },
                }
            }

            if let Err(e) = destination.flush().await {
                drop(destination);
                let _ = tokio::fs::remove_file(&target_path).await;
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed flushing file to disk: {e}"),
                )
                    .into_response();
            }

            println!(
                "  Received: {} ({})",
                target_path.display(),
                crate::routes::download::format_filesize(file_bytes)
            );

            saved_files_count = saved_files_count.saturating_add(1);
            total_bytes = total_bytes.saturating_add(file_bytes);
        } else if field_name.as_deref() == Some("text") {
            match field.text().await {
                Ok(text_content) => {
                    let trimmed = text_content.trim();
                    if !trimmed.is_empty() {
                        let ts = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let text_file_name = format!("note_{ts}.txt");
                        let text_path = resolve_unique_path(&output_dir, &text_file_name);

                        if let Err(e) = tokio::fs::write(&text_path, trimmed).await {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed saving text note: {e}"),
                            )
                                .into_response();
                        }

                        println!(
                            "  Received Text Note:\n{trimmed}\n  -> Saved to: {}",
                            text_path.display()
                        );
                        saved_text_count = saved_text_count.saturating_add(1);
                    }
                },
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        format!("Failed reading text field: {e}"),
                    )
                        .into_response();
                },
            }
        }
    }

    if saved_files_count == 0 && saved_text_count == 0 {
        return (StatusCode::BAD_REQUEST, "No files or text received.").into_response();
    }

    if should_notify {
        let summary = if saved_files_count > 0 {
            format!("qr-drop: Received {saved_files_count} file(s)")
        } else {
            "qr-drop: Received text note".to_string()
        };
        let body = format!("Saved in: {}", output_dir.display());
        crate::notification::send_notification(&summary, &body);
    }

    if should_open {
        crate::notification::open_folder(&output_dir);
    }

    let notify = state.shutdown_tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Some(tx) = notify {
            let _ = tx.send(()).await;
        }
    });

    (
        StatusCode::OK,
        format!(
            "Success! Saved {saved_files_count} file(s) ({}) and {saved_text_count} note(s).",
            crate::routes::download::format_filesize(total_bytes)
        ),
    )
        .into_response()
}
