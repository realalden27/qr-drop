use std::io::Cursor;
use std::path::Path;

use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use axum::response::{Html, IntoResponse, Response};
use tokio_util::io::ReaderStream;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::server::{AppState, Payload};

const DOWNLOAD_TEMPLATE: &str = include_str!("../templates/download.html");

/// Formats a byte count into a readable string (e.g. "14.2 MB").
#[must_use]
pub fn format_filesize(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

/// Computes the total byte size of a file or directory recursively.
pub fn calculate_size(path: &Path) -> u64 {
    if path.is_file() {
        return std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    let mut total: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                total = total.saturating_add(entry.metadata().map(|m| m.len()).unwrap_or(0));
            } else if p.is_dir() {
                total = total.saturating_add(calculate_size(&p));
            }
        }
    }
    total
}

/// Packs an entire directory into an in-memory zip archive.
fn create_zip_archive(dir_path: &Path) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    fn add_dir_contents(
        zip: &mut ZipWriter<Cursor<&mut Vec<u8>>>,
        base_dir: &Path,
        current_dir: &Path,
        options: SimpleFileOptions,
    ) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(base_dir).unwrap_or(&path);
            let name = relative_path.to_string_lossy().replace('\\', "/");

            if path.is_dir() {
                zip.add_directory(&name, options)
                    .map_err(|e| std::io::Error::other(e.to_string()))?;
                add_dir_contents(zip, base_dir, &path, options)?;
            } else if path.is_file() {
                zip.start_file(&name, options)
                    .map_err(|e| std::io::Error::other(e.to_string()))?;
                let mut f = std::fs::File::open(&path)?;
                std::io::copy(&mut f, zip)?;
            }
        }
        Ok(())
    }

    add_dir_contents(&mut zip, dir_path, dir_path, options)?;
    zip.finish()
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(buffer)
}

/// Handler for the landing download page.
pub async fn page_handler(State(state): State<AppState>) -> Response {
    let (filename, filesize, is_dir) = match &state.payload {
        Payload::SendFile { path, is_dir } => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "file".to_string());
            let display_name = if *is_dir { format!("{name}.zip") } else { name };
            let size = calculate_size(path);
            (display_name, format_filesize(size), *is_dir)
        },
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    let download_url = format!("/d/{}/raw", state.token);
    let icon_desc = if is_dir {
        "Folder Archive"
    } else {
        "File Download"
    };

    let html = DOWNLOAD_TEMPLATE
        .replace("{{FILENAME}}", &filename)
        .replace("{{FILESIZE}}", &format!("{filesize} ({icon_desc})"))
        .replace("{{DOWNLOAD_URL}}", &download_url);

    Html(html).into_response()
}

/// Handler for streaming the raw file or zip archive.
pub async fn raw_handler(State(state): State<AppState>) -> Response {
    let (path, is_dir) = match &state.payload {
        Payload::SendFile { path, is_dir } => (path.clone(), *is_dir),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    if is_dir {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "archive".to_string());
        let zip_filename = format!("{name}.zip");

        match create_zip_archive(&path) {
            Ok(bytes) => {
                let len = bytes.len();
                let notify = state.shutdown_tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    if let Some(tx) = notify {
                        let _ = tx.send(()).await;
                    }
                });

                Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, "application/zip")
                    .header(
                        CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{zip_filename}\""),
                    )
                    .header(CONTENT_LENGTH, len.to_string())
                    .body(Body::from(bytes))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            },
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create zip: {e}"),
            )
                .into_response(),
        }
    } else {
        match tokio::fs::File::open(&path).await {
            Ok(file) => {
                let metadata = match file.metadata().await {
                    Ok(m) => m,
                    Err(e) => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                    },
                };

                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "download".to_string());

                let mime = mime_guess::from_path(&path)
                    .first_or_octet_stream()
                    .to_string();

                let stream = ReaderStream::new(file);
                let body = Body::from_stream(stream);

                let notify = state.shutdown_tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    if let Some(tx) = notify {
                        let _ = tx.send(()).await;
                    }
                });

                Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, mime)
                    .header(
                        CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{filename}\""),
                    )
                    .header(CONTENT_LENGTH, metadata.len().to_string())
                    .body(body)
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            },
            Err(e) => (StatusCode::NOT_FOUND, format!("File not found: {e}")).into_response(),
        }
    }
}
