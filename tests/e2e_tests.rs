#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::Read as _;
use std::net::SocketAddr;
use std::path::PathBuf;

use qr_drop::server::{AppState, Payload, create_router};
use reqwest::multipart::{Form, Part};

fn create_temp_dir(suffix: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("qr_drop_test_{}_{}", suffix, rand::random::<u32>()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[tokio::test]
async fn test_download_single_file_e2e() {
    let temp_dir = create_temp_dir("single_file");
    let test_file = temp_dir.join("sample.txt");
    std::fs::write(&test_file, b"Hello from qr-drop single file transfer!").unwrap();

    let token = "testtok1".to_string();
    let state = AppState {
        token: token.clone(),
        payload: Payload::SendFile {
            path: test_file.clone(),
            is_dir: false,
        },
        shutdown_tx: None,
    };

    let router = create_router(state);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = reqwest::Client::new();

    // 1. Test landing page
    let page_resp = client
        .get(format!("http://127.0.0.1:{port}/d/{token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(page_resp.status(), reqwest::StatusCode::OK);
    let html = page_resp.text().await.unwrap();
    assert!(html.contains("sample.txt"));

    // 2. Test raw download
    let raw_resp = client
        .get(format!("http://127.0.0.1:{port}/d/{token}/raw"))
        .send()
        .await
        .unwrap();
    assert_eq!(raw_resp.status(), reqwest::StatusCode::OK);
    let content = raw_resp.bytes().await.unwrap();
    assert_eq!(&content[..], b"Hello from qr-drop single file transfer!");

    server_handle.abort();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_download_folder_zip_e2e() {
    let temp_dir = create_temp_dir("folder_zip");
    let sub_dir = temp_dir.join("subdir");
    std::fs::create_dir_all(&sub_dir).unwrap();

    let file_a = temp_dir.join("file_a.txt");
    let file_b = sub_dir.join("file_b.txt");
    std::fs::write(&file_a, b"Alpha content").unwrap();
    std::fs::write(&file_b, b"Beta content").unwrap();

    let token = "testtok2".to_string();
    let state = AppState {
        token: token.clone(),
        payload: Payload::SendFile {
            path: temp_dir.clone(),
            is_dir: true,
        },
        shutdown_tx: None,
    };

    let router = create_router(state);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/d/{token}/raw"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(content_type, "application/zip");

    let bytes = resp.bytes().await.unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();

    let mut found_a = false;
    let mut found_b = false;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        let name = file.name().replace('\\', "/");
        if name.ends_with("file_a.txt") {
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            assert_eq!(s, "Alpha content");
            found_a = true;
        } else if name.ends_with("file_b.txt") {
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            assert_eq!(s, "Beta content");
            found_b = true;
        }
    }

    assert!(found_a, "file_a.txt must be present in zip");
    assert!(found_b, "file_b.txt must be present in zip");

    server_handle.abort();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_upload_files_and_note_e2e() {
    let out_dir = create_temp_dir("upload_dest");

    let token = "testtok3".to_string();
    let state = AppState {
        token: token.clone(),
        payload: Payload::Receive {
            output_dir: out_dir.clone(),
            open: false,
            notify: false,
        },
        shutdown_tx: None,
    };

    let router = create_router(state);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = reqwest::Client::new();

    // 1. Verify GET /u/{token} displays upload form
    let get_resp = client
        .get(format!("http://127.0.0.1:{port}/u/{token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(get_resp.status(), reqwest::StatusCode::OK);

    // 2. Perform multipart POST with file and text note
    let form = Form::new()
        .part(
            "files",
            Part::bytes(b"DATA_OF_UPLOADED_IMAGE".to_vec()).file_name("photo.png"),
        )
        .part("text", Part::text("Meet me at coffee shop!"));

    let post_resp = client
        .post(format!("http://127.0.0.1:{port}/u/{token}"))
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(post_resp.status(), reqwest::StatusCode::OK);

    // 3. Verify file saved on disk
    let uploaded_file = out_dir.join("photo.png");
    assert!(
        uploaded_file.exists(),
        "Uploaded photo.png must exist on disk"
    );
    let saved_bytes = std::fs::read(&uploaded_file).unwrap();
    assert_eq!(saved_bytes, b"DATA_OF_UPLOADED_IMAGE");

    // 4. Verify text note saved on disk
    let entries = std::fs::read_dir(&out_dir).unwrap();
    let mut found_note = false;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("note_") && name.ends_with(".txt") {
            let note_content = std::fs::read_to_string(entry.path()).unwrap();
            assert_eq!(note_content, "Meet me at coffee shop!");
            found_note = true;
        }
    }
    assert!(found_note, "Text note file must exist on disk");

    server_handle.abort();
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[tokio::test]
async fn test_text_snippet_e2e() {
    let token = "testtok4".to_string();
    let snippet_text = "Secret Token: <token_1234>&abc";
    let state = AppState {
        token: token.clone(),
        payload: Payload::Text {
            content: snippet_text.to_string(),
        },
        shutdown_tx: None,
    };

    let router = create_router(state);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/t/{token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let html = resp.text().await.unwrap();
    // Verify HTML escaping
    assert!(html.contains("&lt;token_1234&gt;&amp;abc"));

    server_handle.abort();
}

#[test]
fn test_receive_cli_defaults_to_received_dir() {
    use clap::Parser as _;
    let cli = qr_drop::config::Cli::parse_from(["qr-drop", "receive"]);
    let (payload, _) = cli.resolve().unwrap();
    match payload {
        Payload::Receive {
            output_dir,
            open,
            notify,
        } => {
            assert!(open, "Default receive must have open=true");
            assert!(notify, "Default receive must have notify=true");
            assert!(
                output_dir.ends_with("received"),
                "Default receive output dir must end with 'received', got: {}",
                output_dir.display()
            );
        },
        _ => panic!("Expected Payload::Receive"),
    }
}
