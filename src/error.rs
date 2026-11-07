use std::fmt;

/// Application result alias.
pub type Result<T> = std::result::Result<T, QrDropError>;

/// Errors returned across the `qr-drop` application.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum QrDropError {
    /// Standard I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// QR code generation error.
    #[error("QR generation error: {0}")]
    Qr(#[from] qrcode::types::QrError),

    /// Network or IP address detection error.
    #[error("Network resolution error: {0}")]
    Network(String),

    /// Server initialization or runtime error.
    #[error("Server error: {0}")]
    Server(String),

    /// File or target resource not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Invalid or unsafe file path error.
    #[error("Path validation error: {0}")]
    InvalidPath(String),

    /// Zip archive creation error.
    #[error("Zip compression error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

impl QrDropError {
    /// Constructs a network resolution error.
    #[must_use]
    pub fn network(msg: impl fmt::Display) -> Self {
        Self::Network(msg.to_string())
    }

    /// Constructs a server error.
    #[must_use]
    pub fn server(msg: impl fmt::Display) -> Self {
        Self::Server(msg.to_string())
    }

    /// Constructs a not found error.
    #[must_use]
    pub fn not_found(msg: impl fmt::Display) -> Self {
        Self::NotFound(msg.to_string())
    }

    /// Constructs an invalid path error.
    #[must_use]
    pub fn invalid_path(msg: impl fmt::Display) -> Self {
        Self::InvalidPath(msg.to_string())
    }
}
