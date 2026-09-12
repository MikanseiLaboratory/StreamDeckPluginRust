//! High-DX Stream Deck plugin host.
//!
//! Depend on this crate only. Protocol types and `#[streamdeck_action]` are re-exported.

#![deny(missing_docs)]

extern crate self as streamdeck_plugin;

pub use async_trait::async_trait;
pub use inventory;
pub use serde_json;
pub use streamdeck_plugin_protocol as protocol;
pub use streamdeck_plugin_protocol::{
    parse_action_payload, parse_dial_rotate, parse_title_parameters, parse_touch_tap,
    ActionPayload, CommandNames, CommandSender, Controller, Coordinates, DeviceInfo, DeviceType,
    DialRotatePayload, EventNames, IncomingMessage, OutgoingCommand, RegistrationArguments,
    RegistrationInfo, Target, TitleParameters, TitleParametersPayload, TouchTapPayload,
    TriggerDescription,
};

#[cfg(feature = "macros")]
pub use streamdeck_plugin_macros::streamdeck_action;

#[cfg(feature = "typegen")]
pub use ts_rs::TS;

mod action;
mod builder;
mod context;
mod dispatch;
mod lifecycle;
mod logging;
mod manifest;
mod registry;

pub use action::{Action, ActionEvent, ActionInstance, EncoderAction, KeypadAction, TypedInstance};
pub use builder::{Plugin, PluginBuilder};
pub use context::{ActionContext, ActionIdentity};
pub use lifecycle::{PluginLifecycle, PluginService};
pub use logging::init_tracing;
pub use registry::{ActionRegistration, Registry};

#[cfg(feature = "typegen")]
mod typegen;
#[cfg(feature = "typegen")]
pub use typegen::{export_typescript, TypescriptExport};

/// Crate result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Framework-level errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Protocol or transport failure.
    #[error(transparent)]
    Protocol(#[from] streamdeck_plugin_protocol::Error),
    /// JSON failure.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Filesystem failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Manifest / registration mismatch.
    #[error("{0}")]
    Manifest(String),
    /// Shared state downcast failed.
    #[error("{0}")]
    State(String),
}

inventory::collect!(ActionRegistration);

#[cfg(feature = "typegen")]
inventory::collect!(TypescriptExport);

#[cfg(test)]
mod tests;
