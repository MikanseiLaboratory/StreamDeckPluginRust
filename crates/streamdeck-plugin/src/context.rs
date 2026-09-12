use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::protocol::{CommandSender, Coordinates, Target, TriggerDescription};
use crate::Result;

/// Identity of a live action instance.
#[derive(Debug, Clone)]
pub struct ActionIdentity {
    /// Instance context.
    pub context: String,
    /// Action UUID.
    pub action_id: String,
    /// Device identifier.
    pub device_id: String,
    /// Slot coordinates.
    pub coordinates: Option<Coordinates>,
}

/// Per-handler context: settings, shared state, and command helpers.
pub struct ActionContext<'a, Settings, State> {
    identity: ActionIdentity,
    settings: Mutex<Settings>,
    state: &'a Arc<State>,
    sender: CommandSender,
}

impl<'a, Settings, State> ActionContext<'a, Settings, State> {
    pub(crate) fn new(
        identity: ActionIdentity,
        settings: Settings,
        state: &'a Arc<State>,
        sender: CommandSender,
    ) -> Self {
        Self {
            identity,
            settings: Mutex::new(settings),
            state,
            sender,
        }
    }

    /// Instance identity.
    pub fn identity(&self) -> &ActionIdentity {
        &self.identity
    }

    /// Shared plugin state.
    pub fn state(&self) -> &State {
        self.state
    }

    /// Command sender for uncommon operations.
    pub fn sender(&self) -> &CommandSender {
        &self.sender
    }

    pub(crate) fn into_settings(self) -> Settings {
        self.settings
            .into_inner()
            .unwrap_or_else(|error| error.into_inner())
    }
}

impl<Settings: Clone, State> ActionContext<'_, Settings, State> {
    /// Current typed settings.
    pub fn settings(&self) -> Settings {
        self.settings
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}

impl<Settings: Serialize, State> ActionContext<'_, Settings, State> {
    /// Mutate settings and persist them through Stream Deck.
    pub fn update_settings(&self, mutate: impl FnOnce(&mut Settings)) -> Result<()> {
        let value = {
            let mut settings = self
                .settings
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            mutate(&mut settings);
            serde_json::to_value(&*settings)?
        };
        self.sender.set_settings(&self.identity.context, &value)?;
        Ok(())
    }

    /// Persist the current settings snapshot.
    pub fn set_settings_now(&self) -> Result<()> {
        let value = serde_json::to_value(
            &*self
                .settings
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        )?;
        self.sender.set_settings(&self.identity.context, &value)?;
        Ok(())
    }
}

impl<Settings, State> ActionContext<'_, Settings, State> {
    /// `setTitle`
    pub fn set_title(&self, title: impl AsRef<str>) -> Result<()> {
        Ok(self.sender.set_title(
            &self.identity.context,
            Some(title.as_ref()),
            Target::HardwareAndSoftware,
            None,
        )?)
    }

    /// `setImage` from a string (data URL or path the host understands).
    pub fn set_image(&self, image: impl AsRef<str>) -> Result<()> {
        Ok(self.sender.set_image(
            &self.identity.context,
            Some(image.as_ref()),
            Target::HardwareAndSoftware,
            None,
        )?)
    }

    /// `setImage` from PNG bytes.
    pub fn set_image_bytes(&self, png_bytes: &[u8]) -> Result<()> {
        Ok(self.sender.set_image_bytes(
            &self.identity.context,
            png_bytes,
            Target::HardwareAndSoftware,
            None,
        )?)
    }

    /// `setImage` from a file.
    pub fn set_image_from_file(&self, path: impl AsRef<Path>) -> Result<()> {
        Ok(self.sender.set_image_from_file(
            &self.identity.context,
            path,
            Target::HardwareAndSoftware,
            None,
        )?)
    }

    /// `setState`
    pub fn set_state(&self, state: i32) -> Result<()> {
        Ok(self.sender.set_state(&self.identity.context, state)?)
    }

    /// `showOk`
    pub fn show_ok(&self) -> Result<()> {
        Ok(self.sender.show_ok(&self.identity.context)?)
    }

    /// `showAlert`
    pub fn show_alert(&self) -> Result<()> {
        Ok(self.sender.show_alert(&self.identity.context)?)
    }

    /// `sendToPropertyInspector`
    pub fn send_to_property_inspector<T: Serialize>(&self, payload: &T) -> Result<()> {
        Ok(self
            .sender
            .send_to_property_inspector(&self.identity.context, &serde_json::to_value(payload)?)?)
    }

    /// `setFeedback`
    pub fn set_feedback<T: Serialize>(&self, payload: &T) -> Result<()> {
        Ok(self
            .sender
            .set_feedback(&self.identity.context, &serde_json::to_value(payload)?)?)
    }

    /// `setFeedbackLayout`
    pub fn set_feedback_layout(&self, layout: impl AsRef<str>) -> Result<()> {
        Ok(self
            .sender
            .set_feedback_layout(&self.identity.context, layout.as_ref())?)
    }

    /// `setTriggerDescription`
    pub fn set_trigger_description(&self, description: TriggerDescription) -> Result<()> {
        Ok(self
            .sender
            .set_trigger_description(&self.identity.context, &description)?)
    }

    /// `getResources`
    pub fn get_resources(&self) -> Result<()> {
        Ok(self.sender.get_resources(&self.identity.context, None)?)
    }

    /// `setResources`
    pub fn set_resources(&self, resources: &BTreeMap<String, String>) -> Result<()> {
        Ok(self
            .sender
            .set_resources(&self.identity.context, resources)?)
    }

    /// `openUrl`
    pub fn open_url(&self, url: impl AsRef<str>) -> Result<()> {
        Ok(self.sender.open_url(url.as_ref())?)
    }

    /// `logMessage`
    pub fn log_message(&self, message: impl AsRef<str>) -> Result<()> {
        Ok(self.sender.log_message(message.as_ref())?)
    }
}
