//! Stream Deck plugin WebSocket protocol types and transport.
//!
//! This crate is the wire layer. Plugin authors who want actions, settings, and
//! dispatch should use [`streamdeck-plugin`](https://crates.io/crates/streamdeck-plugin).

#![deny(missing_docs)]

/// Outgoing command names and the cloneable sender.
pub mod commands;
/// Inbound event names and registration metadata.
pub mod events;
/// Action-scoped payload parsers.
pub mod payload;
/// Launch-argument parsing.
pub mod registration;

mod error;

pub use commands::{CommandNames, OutgoingCommand, TriggerDescription};

#[cfg(feature = "tokio-transport")]
pub use commands::CommandSender;
pub use error::Error;
pub use events::{
    ApplicationInfo, ColorInfo, Controller, Coordinates, DeviceInfo, DeviceSize, DeviceType,
    EventNames, IncomingMessage, PluginInfo, RegistrationInfo, Target,
};
pub use payload::{
    parse_action_payload, parse_dial_rotate, parse_title_parameters, parse_touch_tap,
    ActionPayload, DialRotatePayload, TitleParameters, TitleParametersPayload, TouchTapPayload,
};
pub use registration::RegistrationArguments;

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// JSON helpers that match Stream Deck's camelCase wire format.
pub mod json {
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    /// Deserialize an inbound Stream Deck message.
    pub fn from_str<T: DeserializeOwned>(json: &str) -> Result<T, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize an outbound command, omitting `null` fields.
    pub fn to_string<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
        serde_json::to_string(value)
    }
}

/// Dual-loop Stream Deck connection.
#[cfg(feature = "tokio-transport")]
pub mod connection;
/// WebSocket transport trait and Tokio implementation.
#[cfg(feature = "tokio-transport")]
pub mod transport;

#[cfg(feature = "tokio-transport")]
pub use connection::StreamDeckConnection;
#[cfg(feature = "tokio-transport")]
pub use transport::{TokioWebSocketTransport, Transport};

#[cfg(all(test, feature = "tokio-transport"))]
mod connection_tests;
#[cfg(test)]
mod tests;
