use serde_json::json;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::commands::CommandSender;
use crate::events::{IncomingMessage, RegistrationInfo};
use crate::registration::RegistrationArguments;
use crate::transport::{TokioWebSocketTransport, Transport};
use crate::{Error, Result};

/// Dual-loop Stream Deck connection: one reader, one writer, one task.
pub struct StreamDeckConnection<T: Transport> {
    arguments: RegistrationArguments,
    transport: T,
    outbound_tx: mpsc::UnboundedSender<String>,
    outbound_rx: Option<mpsc::UnboundedReceiver<String>>,
}

impl StreamDeckConnection<TokioWebSocketTransport> {
    /// Connect with the default Tokio WebSocket transport.
    pub fn new(arguments: RegistrationArguments) -> Self {
        Self::with_transport(arguments, TokioWebSocketTransport::new())
    }
}

impl<T: Transport> StreamDeckConnection<T> {
    /// Inject a transport. Used by tests.
    pub fn with_transport(arguments: RegistrationArguments, transport: T) -> Self {
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel();
        Self {
            arguments,
            transport,
            outbound_tx,
            outbound_rx: Some(outbound_rx),
        }
    }

    /// Plugin UUID from launch arguments.
    pub fn plugin_uuid(&self) -> &str {
        &self.arguments.plugin_uuid
    }

    /// Parsed `-info` payload.
    pub fn info(&self) -> &RegistrationInfo {
        &self.arguments.info
    }

    /// Cloneable command sender. Safe to share across actions and services.
    pub fn sender(&self) -> CommandSender {
        CommandSender::new(self.outbound_tx.clone(), self.arguments.plugin_uuid.clone())
    }

    /// Register, then read events until the socket or token ends.
    pub async fn run<F, Fut>(mut self, mut on_message: F, token: CancellationToken) -> Result<()>
    where
        F: FnMut(IncomingMessage) -> Fut + Send,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let url = format!("ws://127.0.0.1:{}", self.arguments.port);
        self.transport.connect(&url).await?;

        let register = json!({
            "event": self.arguments.register_event,
            "uuid": self.arguments.plugin_uuid,
        });
        self.transport.send(&register.to_string()).await?;

        let mut outbound_rx = self.outbound_rx.take().expect("run called twice");
        loop {
            tokio::select! {
                _ = token.cancelled() => break,
                incoming = self.transport.receive() => {
                    match incoming? {
                        None => break,
                        Some(json) => match crate::json::from_str::<IncomingMessage>(&json) {
                            Ok(message) if !message.event.trim().is_empty() => on_message(message).await,
                            Ok(_) => {}
                            Err(error) => eprintln!("Failed to deserialize Stream Deck message: {error}"),
                        },
                    }
                }
                next = outbound_rx.recv() => {
                    if let Some(json) = next {
                        self.transport.send(&json).await?;
                    }
                }
            }
        }
        Ok::<(), Error>(())
    }
}
