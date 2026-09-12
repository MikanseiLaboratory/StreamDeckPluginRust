use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{json, Value};

use super::{CommandNames, OutgoingCommand, TriggerDescription};
use crate::events::Target;
use crate::{Error, Result};

/// Cloneable outbound command channel used by actions and shared services.
#[derive(Clone)]
pub struct CommandSender {
    tx: tokio::sync::mpsc::UnboundedSender<String>,
    plugin_uuid: String,
}

impl CommandSender {
    /// Create a sender that writes JSON into `tx`.
    pub fn new(
        tx: tokio::sync::mpsc::UnboundedSender<String>,
        plugin_uuid: impl Into<String>,
    ) -> Self {
        Self {
            tx,
            plugin_uuid: plugin_uuid.into(),
        }
    }

    /// Plugin UUID used as `context` for global commands.
    pub fn plugin_uuid(&self) -> &str {
        &self.plugin_uuid
    }

    /// Enqueue a pre-built command.
    pub fn send_command(&self, command: &OutgoingCommand) -> Result<()> {
        let json = crate::json::to_string(command)?;
        self.tx.send(json).map_err(|_| Error::ChannelClosed)
    }

    /// Enqueue raw JSON.
    pub fn send_raw(&self, json: String) -> Result<()> {
        self.tx.send(json).map_err(|_| Error::ChannelClosed)
    }

    /// `setTitle`
    pub fn set_title(
        &self,
        context: &str,
        title: Option<&str>,
        target: Target,
        state: Option<i32>,
    ) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::SET_TITLE)
                .with_context(context)
                .with_payload(json!({ "title": title, "target": target, "state": state })),
        )
    }

    /// `setImage` from a data URL or path string.
    pub fn set_image(
        &self,
        context: &str,
        image: Option<&str>,
        target: Target,
        state: Option<i32>,
    ) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::SET_IMAGE)
                .with_context(context)
                .with_payload(json!({ "image": image, "target": target, "state": state })),
        )
    }

    /// `setImage` from PNG bytes.
    pub fn set_image_bytes(
        &self,
        context: &str,
        png_bytes: &[u8],
        target: Target,
        state: Option<i32>,
    ) -> Result<()> {
        let data_url = format!("data:image/png;base64,{}", base64_encode(png_bytes));
        self.set_image(context, Some(&data_url), target, state)
    }

    /// `setImage` from a local file. MIME is inferred from the extension.
    pub fn set_image_from_file(
        &self,
        context: &str,
        path: impl AsRef<Path>,
        target: Target,
        state: Option<i32>,
    ) -> Result<()> {
        let path = path.as_ref();
        let bytes = std::fs::read(path)?;
        let mime = match path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref()
        {
            Some("jpg" | "jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            _ => "image/png",
        };
        let data_url = format!("data:{mime};base64,{}", base64_encode(&bytes));
        self.set_image(context, Some(&data_url), target, state)
    }

    /// `setState`
    pub fn set_state(&self, context: &str, state: i32) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::SET_STATE)
                .with_context(context)
                .with_payload(json!({ "state": state })),
        )
    }

    /// `setSettings` — payload is the settings object itself.
    pub fn set_settings(&self, context: &str, settings: &Value) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::SET_SETTINGS)
                .with_context(context)
                .with_payload(settings.clone()),
        )
    }

    /// `getSettings`
    pub fn get_settings(&self, context: &str, id: Option<String>) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::GET_SETTINGS)
                .with_context(context)
                .with_id(id),
        )
    }

    /// `setGlobalSettings`
    pub fn set_global_settings(&self, settings: &Value) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::SET_GLOBAL_SETTINGS)
                .with_context(&self.plugin_uuid)
                .with_payload(settings.clone()),
        )
    }

    /// `getGlobalSettings`
    pub fn get_global_settings(&self, id: Option<String>) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::GET_GLOBAL_SETTINGS)
                .with_context(&self.plugin_uuid)
                .with_id(id),
        )
    }

    /// `showOk`
    pub fn show_ok(&self, context: &str) -> Result<()> {
        self.send_command(&OutgoingCommand::new(CommandNames::SHOW_OK).with_context(context))
    }

    /// `showAlert`
    pub fn show_alert(&self, context: &str) -> Result<()> {
        self.send_command(&OutgoingCommand::new(CommandNames::SHOW_ALERT).with_context(context))
    }

    /// `sendToPropertyInspector`
    pub fn send_to_property_inspector(&self, context: &str, payload: &Value) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::SEND_TO_PROPERTY_INSPECTOR)
                .with_context(context)
                .with_payload(payload.clone()),
        )
    }

    /// `setFeedback`
    pub fn set_feedback(&self, context: &str, payload: &Value) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::SET_FEEDBACK)
                .with_context(context)
                .with_payload(payload.clone()),
        )
    }

    /// `setFeedbackLayout`
    pub fn set_feedback_layout(&self, context: &str, layout: &str) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::SET_FEEDBACK_LAYOUT)
                .with_context(context)
                .with_payload(json!({ "layout": layout })),
        )
    }

    /// `setTriggerDescription`
    pub fn set_trigger_description(
        &self,
        context: &str,
        description: &TriggerDescription,
    ) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::SET_TRIGGER_DESCRIPTION)
                .with_context(context)
                .with_payload(serde_json::to_value(description)?),
        )
    }

    /// `openUrl`
    pub fn open_url(&self, url: &str) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::OPEN_URL).with_payload(json!({ "url": url })),
        )
    }

    /// `logMessage`
    pub fn log_message(&self, message: &str) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::LOG_MESSAGE)
                .with_payload(json!({ "message": message })),
        )
    }

    /// `switchToProfile`
    pub fn switch_to_profile(
        &self,
        device: &str,
        profile: Option<&str>,
        page: Option<i32>,
    ) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::SWITCH_TO_PROFILE)
                .with_context(&self.plugin_uuid)
                .with_device(device)
                .with_payload(json!({ "profile": profile, "page": page })),
        )
    }

    /// `getSecrets`
    pub fn get_secrets(&self) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::GET_SECRETS).with_context(&self.plugin_uuid),
        )
    }

    /// `getResources`
    pub fn get_resources(&self, context: &str, id: Option<String>) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::GET_RESOURCES)
                .with_context(context)
                .with_id(id),
        )
    }

    /// `setResources` — payload is the resource map itself.
    pub fn set_resources(&self, context: &str, resources: &BTreeMap<String, String>) -> Result<()> {
        self.send_command(
            &OutgoingCommand::new(CommandNames::SET_RESOURCES)
                .with_context(context)
                .with_payload(serde_json::to_value(resources)?),
        )
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 63) as usize] as char);
        out.push(TABLE[((triple >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
