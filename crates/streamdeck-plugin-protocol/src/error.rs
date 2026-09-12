use thiserror::Error;

/// Protocol and transport failures.
#[derive(Debug, Error)]
pub enum Error {
    /// Launch arguments from Stream Deck were missing or invalid.
    #[error("{0}")]
    Registration(String),
    /// JSON serialization or deserialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// The outbound command channel was closed.
    #[error("command channel is closed")]
    ChannelClosed,
    /// The WebSocket transport failed.
    #[error(transparent)]
    Transport(#[from] TransportError),
    /// A filesystem operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Transport-specific error wrapper so the core crate can compile without Tokio.
#[derive(Debug, Error)]
#[error("{0}")]
pub struct TransportError(pub String);

impl TransportError {
    /// Create a transport error from a displayable value.
    pub fn new(error: impl std::fmt::Display) -> Self {
        Self(error.to_string())
    }
}
