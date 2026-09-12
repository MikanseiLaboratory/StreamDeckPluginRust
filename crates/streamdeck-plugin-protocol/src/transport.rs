use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use crate::error::TransportError;
use crate::Result;

/// Byte-oriented Stream Deck transport. Tests inject an in-memory implementation.
#[async_trait]
pub trait Transport: Send {
    /// Connect to `url`.
    async fn connect(&mut self, url: &str) -> Result<()>;
    /// Send a UTF-8 text frame.
    async fn send(&mut self, json: &str) -> Result<()>;
    /// Receive the next text frame. `None` means the socket closed.
    async fn receive(&mut self) -> Result<Option<String>>;
}

/// Tokio WebSocket transport talking to `ws://127.0.0.1:{port}`.
pub struct TokioWebSocketTransport {
    stream: Option<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

impl TokioWebSocketTransport {
    /// Create a disconnected transport.
    pub fn new() -> Self {
        Self { stream: None }
    }
}

impl Default for TokioWebSocketTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for TokioWebSocketTransport {
    async fn connect(&mut self, url: &str) -> Result<()> {
        let (stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .map_err(TransportError::new)?;
        self.stream = Some(stream);
        Ok(())
    }

    async fn send(&mut self, json: &str) -> Result<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| TransportError::new("transport is not connected"))?;
        stream
            .send(Message::Text(json.to_string().into()))
            .await
            .map_err(TransportError::new)?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<String>> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| TransportError::new("transport is not connected"))?;
        loop {
            match stream.next().await {
                None => return Ok(None),
                Some(Err(error)) => return Err(TransportError::new(error).into()),
                Some(Ok(Message::Text(text))) => return Ok(Some(text.to_string())),
                Some(Ok(Message::Close(_))) => return Ok(None),
                Some(Ok(_)) => continue,
            }
        }
    }
}
