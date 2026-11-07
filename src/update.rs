use std::io::Cursor;

use crate::error::{QrDropError, Result};

#[derive(serde::Deserialize, Debug)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(serde::Deserialize, Debug)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(serde::Deserialize, Debug)]
struct CratesResponse {
    #[serde(rename = "crate")]
    krate: Option<CrateMeta>,
}

#[derive(serde::Deserialize, Debug)]
struct CrateMeta {
    max_version: Option<String>,
}

fn is_cargo_install(exe: &std::path::Path) -> bool {
    exe.components()
        .any(|c| c.as_os_str().to_string_lossy().contains(".cargo"))
}

/// Parses a semver version string (e.g. "0.1.2", "v0.1.2", or "qr-drop 0.1.2") into (major, minor, patch).
#[must_use]
pub fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let clean = s.trim();
    let clean = clean.strip_prefix("qr-drop").unwrap_or(clean).trim();
    let clean = clean.trim_start_matches('v').trim();

    let parts: Vec<&str> = clean.split('.').collect();
    let major = parts.first()?.parse::<u64>().ok()?;
    let minor = parts.get(1)?.parse::<u64>().ok()?;
    let patch_str = parts.get(2)?;
    let patch = patch_str.split('-').next()?.parse::<u64>().ok()?;
    Some((major, minor, patch))
}

/// Identifies the asset filename target identifier for the current OS and architecture.
#[must_use]
pub fn get_target_triple() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "x86_64-pc-windows-msvc"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "aarch64-apple-darwin"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "x86_64-apple-darwin"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "x86_64-unknown-linux-gnu"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "aarch64-unknown-linux-gnu"
    }
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64")
    )))]
    {
        "unknown"
    }
}

/// Checks crates.io and GitHub Releases and updates the running binary if a newer release exists.
pub async fn check_and_update() -> Result<()> {
    let current_version_str = env!("CARGO_PKG_VERSION");
    let current_version = parse_version(current_version_str).unwrap_or((0, 0, 0));

    println!("Checking for updates on crates.io and GitHub...");

    let client = reqwest::Client::builder()
        .user_agent(format!(
            "qr-drop/{current_version_str} (https://github.com/realalden27/qr-drop)"
        ))
        .build()
        .map_err(|e| QrDropError::network(format!("Failed to build HTTP client: {e}")))?;

    // 1. Query crates.io API
    let crates_url = "https://crates.io/api/v1/crates/qr-drop";
    let crates_version = match client.get(crates_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<CratesResponse>().await {
                data.krate.and_then(|k| k.max_version)
            } else {
                None
            }
        },
        _ => None,
    };

    // 2. Query GitHub Releases API
    let github_url = "https://api.github.com/repos/realalden27/qr-drop/releases/latest";
    let release: Option<GitHubRelease> = match client.get(github_url).send().await {
        Ok(resp) if resp.status().is_success() => resp.json::<GitHubRelease>().await.ok(),
        _ => None,
    };

    let latest_github_version_str = release
        .as_ref()
        .map(|r| r.tag_name.trim().trim_start_matches('v').to_string());

    let github_parsed = latest_github_version_str
        .as_deref()
        .and_then(parse_version)
        .unwrap_or((0, 0, 0));
    let crates_parsed = crates_version
        .as_deref()
        .and_then(parse_version)
        .unwrap_or((0, 0, 0));

    let max_available = std::cmp::max(github_parsed, crates_parsed);

    if max_available <= current_version {
        println!("qr-drop is already up to date (v{current_version_str}).");
        return Ok(());
    }

    let target_version_str = if crates_parsed >= github_parsed {
        crates_version.as_deref().unwrap_or(current_version_str)
    } else {
        latest_github_version_str
            .as_deref()
            .unwrap_or(current_version_str)
    };

    println!("Found newer version v{target_version_str} (current: v{current_version_str}).");

    // If currently running from a cargo installation (.cargo/bin), update via cargo install
    if std::env::current_exe().is_ok_and(|p| is_cargo_install(&p)) {
        println!("Updating via cargo install qr-drop...");
        let status = std::process::Command::new("cargo")
            .args(["install", "qr-drop"])
            .status();
        if matches!(status, Ok(s) if s.success()) {
            println!("Successfully updated qr-drop via cargo to v{target_version_str}!");
            return Ok(());
        }
        println!("cargo install failed or cargo not found; attempting GitHub binary download...");
    }

    let release = match release {
        Some(r) => r,
        None => {
            return Err(QrDropError::network(
                "New version detected on crates.io, but GitHub Releases could not be reached. Run 'cargo install qr-drop' to update.".to_string(),
            ));
        },
    };

    let target = get_target_triple();
    let matching_asset = release.assets.iter().find(|a| a.name.contains(target));

    let asset = match matching_asset {
        Some(a) => a,
        None => {
            return Err(QrDropError::network(format!(
                "No release binary found for target '{target}' in release '{}'. Please install manually from {}",
                release.tag_name, "https://github.com/realalden27/qr-drop/releases"
            )));
        },
    };

    println!(
        "Found newer version {} (current: v{}). Downloading {}...",
        release.tag_name, current_version_str, asset.name
    );

    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(|e| QrDropError::network(format!("Download failed: {e}")))?
        .bytes()
        .await
        .map_err(|e| QrDropError::network(format!("Failed reading asset bytes: {e}")))?;

    let temp_dir = std::env::temp_dir();
    let temp_exe = temp_dir.join(format!(
        "qr_drop_update_{}.tmp",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));

    if asset.name.ends_with(".zip") {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
            .map_err(|e| QrDropError::server(format!("Failed to read zip archive: {e}")))?;
        let mut found = false;
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| QrDropError::server(format!("Zip read error: {e}")))?;
            let name = file.name().replace('\\', "/");
            if name.ends_with("qr-drop.exe") || name.ends_with("qr-drop") {
                let mut outfile = std::fs::File::create(&temp_exe)?;
                std::io::copy(&mut file, &mut outfile)?;
                found = true;
                break;
            }
        }
        if !found {
            return Err(QrDropError::server(
                "Executable not found inside downloaded archive",
            ));
        }
    } else if asset.name.ends_with(".tar.gz") {
        let tar = flate2::read::GzDecoder::new(Cursor::new(bytes));
        let mut archive = tar::Archive::new(tar);
        let mut found = false;
        let entries = archive
            .entries()
            .map_err(|e| QrDropError::server(format!("Tar read error: {e}")))?;
        for entry in entries {
            let mut file =
                entry.map_err(|e| QrDropError::server(format!("Tar entry error: {e}")))?;
            let path = file
                .path()
                .map_err(|e| QrDropError::server(format!("Tar path error: {e}")))?
                .to_string_lossy()
                .into_owned();
            if path.ends_with("qr-drop") {
                file.unpack(&temp_exe)
                    .map_err(|e| QrDropError::server(format!("Unpack error: {e}")))?;
                found = true;
                break;
            }
        }
        if !found {
            return Err(QrDropError::server(
                "Executable not found inside downloaded archive",
            ));
        }
    } else {
        std::fs::write(&temp_exe, &bytes)?;
    }

    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if let Ok(metadata) = std::fs::metadata(&temp_exe) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&temp_exe, perms);
        }
    }

    self_replace::self_replace(&temp_exe)
        .map_err(|e| QrDropError::server(format!("Failed to self-replace executable: {e}")))?;

    let _ = std::fs::remove_file(&temp_exe);

    println!("Successfully updated qr-drop to {}!", release.tag_name);

    Ok(())
}
