use std::path::Path;

/// Opens the specified folder in the operating system's default file manager.
pub fn open_folder(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

/// Sends a desktop notification alerting that files or notes were received.
pub fn send_notification(summary: &str, body: &str) {
    let _ = notify_rust::Notification::new()
        .appname("qr-drop")
        .summary(summary)
        .body(body)
        .show();
}
