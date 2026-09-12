use serde::Serialize;
use serde_json::Value;

/// Outgoing WebSocket command names defined by the Stream Deck plugin protocol.
pub struct CommandNames;

impl CommandNames {
    /// `getGlobalSettings`
    pub const GET_GLOBAL_SETTINGS: &'static str = "getGlobalSettings";
    /// `getResources`
    pub const GET_RESOURCES: &'static str = "getResources";
    /// `getSecrets`
    pub const GET_SECRETS: &'static str = "getSecrets";
    /// `getSettings`
    pub const GET_SETTINGS: &'static str = "getSettings";
    /// `logMessage`
    pub const LOG_MESSAGE: &'static str = "logMessage";
    /// `openUrl`
    pub const OPEN_URL: &'static str = "openUrl";
    /// `sendToPropertyInspector`
    pub const SEND_TO_PROPERTY_INSPECTOR: &'static str = "sendToPropertyInspector";
    /// `setFeedback`
    pub const SET_FEEDBACK: &'static str = "setFeedback";
    /// `setFeedbackLayout`
    pub const SET_FEEDBACK_LAYOUT: &'static str = "setFeedbackLayout";
    /// `setGlobalSettings`
    pub const SET_GLOBAL_SETTINGS: &'static str = "setGlobalSettings";
    /// `setImage`
    pub const SET_IMAGE: &'static str = "setImage";
    /// `setResources`
    pub const SET_RESOURCES: &'static str = "setResources";
    /// `setSettings`
    pub const SET_SETTINGS: &'static str = "setSettings";
    /// `setState`
    pub const SET_STATE: &'static str = "setState";
    /// `setTitle`
    pub const SET_TITLE: &'static str = "setTitle";
    /// `setTriggerDescription`
    pub const SET_TRIGGER_DESCRIPTION: &'static str = "setTriggerDescription";
    /// `showAlert`
    pub const SHOW_ALERT: &'static str = "showAlert";
    /// `showOk`
    pub const SHOW_OK: &'static str = "showOk";
    /// `switchToProfile`
    pub const SWITCH_TO_PROFILE: &'static str = "switchToProfile";
}

/// Encoder accessibility labels sent with `setTriggerDescription`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerDescription {
    /// Rotate hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotate: Option<String>,
    /// Push hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<String>,
    /// Touch hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touch: Option<String>,
    /// Long-touch hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_touch: Option<String>,
}

/// Generic outbound envelope. Null fields are omitted.
#[derive(Debug, Clone, Serialize)]
pub struct OutgoingCommand {
    event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<Value>,
}

impl OutgoingCommand {
    /// Start a command with the given event name.
    pub fn new(event: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            context: None,
            device: None,
            id: None,
            payload: None,
        }
    }

    /// Set `context`.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Set `device`.
    pub fn with_device(mut self, device: impl Into<String>) -> Self {
        self.device = Some(device.into());
        self
    }

    /// Set optional `id`.
    pub fn with_id(mut self, id: Option<impl Into<String>>) -> Self {
        self.id = id.map(Into::into);
        self
    }

    /// Set `payload`.
    pub fn with_payload(mut self, payload: Value) -> Self {
        self.payload = Some(payload);
        self
    }
}

#[cfg(feature = "tokio-transport")]
mod sender;

#[cfg(feature = "tokio-transport")]
pub use sender::CommandSender;
