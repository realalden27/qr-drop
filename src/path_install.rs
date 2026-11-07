use std::path::{Path, PathBuf};

/// Automatically installs the running executable to the operating system's standard user programs directory
/// and ensures that directory is in the user's PATH environment variable.
pub fn ensure_in_path() {
    // Skip if explicitly disabled or running under unit tests
    if std::env::var("QR_DROP_NO_PATH_INSTALL").is_ok() || cfg!(test) {
        return;
    }

    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };

    let dir = match current_exe.parent() {
        Some(d) => d,
        None => return,
    };

    // Do not auto-install when running in temporary directories
    let temp_dir = std::env::temp_dir();
    if dir.starts_with(&temp_dir) {
        return;
    }

    // Do not auto-install when running local development builds (target/debug or target/release)
    let dir_str = dir.to_string_lossy();
    if dir_str.ends_with("target\\debug")
        || dir_str.ends_with("target/debug")
        || dir_str.ends_with("target\\release")
        || dir_str.ends_with("target/release")
    {
        return;
    }

    #[cfg(target_os = "windows")]
    install_windows(&current_exe);

    #[cfg(target_family = "unix")]
    install_unix(&current_exe);
}

#[cfg(target_os = "windows")]
fn install_windows(current_exe: &Path) {
    let local_app_data = match std::env::var_os("LOCALAPPDATA") {
        Some(val) => PathBuf::from(val),
        None => return,
    };

    let target_dir = local_app_data.join("Programs").join("qr-drop");
    let target_exe = target_dir.join("qr-drop.exe");

    // Already running from the installed Programs location, or already installed
    if target_exe.exists() || current_exe == target_exe {
        return;
    }

    // If installed via cargo (.cargo\bin), it is already in a proper tools directory
    let current_str = current_exe.to_string_lossy();
    if current_str.contains(".cargo\\bin") || current_str.contains(".cargo/bin") {
        return;
    }

    if std::fs::create_dir_all(&target_dir).is_err() {
        return;
    }

    if std::fs::copy(current_exe, &target_exe).is_err() {
        return;
    }

    let target_dir_str = target_dir.to_string_lossy();
    let script = format!(
        "$old = [Environment]::GetEnvironmentVariable('Path', 'User'); \
         if ($old -notlike '*{target_dir_str}*') {{ \
             $new = if ($old) {{ \"$old;{target_dir_str}\" }} else {{ '{target_dir_str}' }}; \
             [Environment]::SetEnvironmentVariable('Path', $new, 'User'); \
         }}"
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output();

    if output.is_ok_and(|out| out.status.success()) {
        println!(
            "  Installed to '{}' and added to your User PATH. You can now run 'qr-drop' from any terminal.",
            target_dir.display()
        );
    }
}

#[cfg(target_family = "unix")]
fn install_unix(current_exe: &Path) {
    let home = match std::env::var_os("HOME") {
        Some(val) => PathBuf::from(val),
        None => return,
    };

    let local_bin = home.join(".local").join("bin");
    let target_exe = local_bin.join("qr-drop");

    // Already running from ~/.local/bin or already installed there
    if target_exe.exists() || current_exe == target_exe {
        return;
    }
    let current_str = current_exe.to_string_lossy();
    if current_str.contains(".cargo/bin") {
        return;
    }

    if std::fs::create_dir_all(&local_bin).is_err() {
        return;
    }

    if std::fs::copy(current_exe, &target_exe).is_err() {
        return;
    }

    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if let Ok(metadata) = std::fs::metadata(&target_exe) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&target_exe, perms);
        }
    }

    println!(
        "  Installed to '{}' and ready to run from terminal.",
        target_exe.display()
    );
}
