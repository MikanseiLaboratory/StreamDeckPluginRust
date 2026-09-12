use std::any::Any;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use futures::FutureExt;
use serde_json::Value;

use crate::action::{ActionEvent, ActionInstance};
use crate::lifecycle::PluginLifecycle;
use crate::protocol::{CommandSender, EventNames, IncomingMessage};
use crate::registry::Registry;
use crate::Result;

/// Creates and routes events to action instances.
pub struct Dispatcher {
    registry: Registry,
    state: Arc<dyn Any + Send + Sync>,
    sender: CommandSender,
    instances: HashMap<String, Box<dyn ActionInstance>>,
    lifecycle: Vec<Box<dyn PluginLifecycle>>,
}

impl Dispatcher {
    /// Create a dispatcher.
    pub fn new(
        registry: Registry,
        state: Arc<dyn Any + Send + Sync>,
        sender: CommandSender,
        lifecycle: Vec<Box<dyn PluginLifecycle>>,
    ) -> Self {
        Self {
            registry,
            state,
            sender,
            instances: HashMap::new(),
            lifecycle,
        }
    }

    /// Sequential dispatch. Handler panics are isolated.
    pub async fn dispatch(&mut self, message: IncomingMessage) {
        if let Err(error) = self.dispatch_inner(message).await {
            tracing::error!(error = %error, "dispatch failed");
        }
    }

    async fn dispatch_inner(&mut self, message: IncomingMessage) -> Result<()> {
        match message.event.as_str() {
            EventNames::WILL_APPEAR => return self.appear(&message).await,
            EventNames::WILL_DISAPPEAR => return self.disappear(&message).await,
            EventNames::APPLICATION_DID_LAUNCH => {
                let application = nested_str(&message, "application");
                for hook in &mut self.lifecycle {
                    hook.on_application_did_launch(&application).await?;
                }
                return Ok(());
            }
            EventNames::APPLICATION_DID_TERMINATE => {
                let application = nested_str(&message, "application");
                for hook in &mut self.lifecycle {
                    hook.on_application_did_terminate(&application).await?;
                }
                return Ok(());
            }
            EventNames::DEVICE_DID_CONNECT => {
                if let (Some(device), Some(info)) =
                    (message.device.as_deref(), message.device_info.as_ref())
                {
                    for hook in &mut self.lifecycle {
                        hook.on_device_did_connect(device, info).await?;
                    }
                }
                return Ok(());
            }
            EventNames::DEVICE_DID_DISCONNECT => {
                if let Some(device) = message.device.as_deref() {
                    for hook in &mut self.lifecycle {
                        hook.on_device_did_disconnect(device).await?;
                    }
                }
                return Ok(());
            }
            EventNames::DEVICE_DID_CHANGE => {
                if let (Some(device), Some(info)) =
                    (message.device.as_deref(), message.device_info.as_ref())
                {
                    for hook in &mut self.lifecycle {
                        hook.on_device_did_change(device, info).await?;
                    }
                }
                return Ok(());
            }
            EventNames::SYSTEM_DID_WAKE_UP => {
                for hook in &mut self.lifecycle {
                    hook.on_system_did_wake_up().await?;
                }
                return Ok(());
            }
            EventNames::DID_RECEIVE_DEEP_LINK => {
                let url = nested_str(&message, "url");
                for hook in &mut self.lifecycle {
                    hook.on_did_receive_deep_link(&url).await?;
                }
                return Ok(());
            }
            EventNames::DID_RECEIVE_GLOBAL_SETTINGS => {
                let settings = nested_value(&message, "settings");
                for hook in &mut self.lifecycle {
                    hook.on_did_receive_global_settings(&settings).await?;
                }
                return Ok(());
            }
            EventNames::DID_RECEIVE_SECRETS => {
                let secrets = nested_value(&message, "secrets");
                for hook in &mut self.lifecycle {
                    hook.on_did_receive_secrets(&secrets).await?;
                }
                return Ok(());
            }
            _ => {}
        }

        let Some(context) = message.context.clone() else {
            tracing::debug!(event = %message.event, "no action instance for event");
            return Ok(());
        };
        let Some(instance) = self.instances.get_mut(&context) else {
            tracing::debug!(event = %message.event, context, "no action instance for event");
            return Ok(());
        };
        let Some(event) = ActionEvent::from_message(&message) else {
            return Ok(());
        };
        isolate(instance.handle(event, self.sender.clone())).await
    }

    async fn appear(&mut self, message: &IncomingMessage) -> Result<()> {
        let (Some(context), Some(action_uuid)) = (message.context.clone(), message.action.clone())
        else {
            return Ok(());
        };
        let Some(registration) = self.registry.get(&action_uuid) else {
            tracing::warn!(action = %action_uuid, "no action registered for UUID");
            return Ok(());
        };
        let event = ActionEvent::from_message(message)
            .unwrap_or(ActionEvent::WillAppear(Default::default()));
        let mut instance = (registration.factory)(Arc::clone(&self.state));
        let payload = match &event {
            ActionEvent::WillAppear(payload) => payload,
            _ => &Default::default(),
        };
        instance.attach(
            context.clone(),
            action_uuid,
            message.device.clone().unwrap_or_default(),
            payload.coordinates,
        );
        instance.apply_settings(&payload.settings);
        isolate(instance.handle(event, self.sender.clone())).await?;
        self.instances.insert(context, instance);
        Ok(())
    }

    async fn disappear(&mut self, message: &IncomingMessage) -> Result<()> {
        let Some(context) = message.context.as_deref() else {
            return Ok(());
        };
        let Some(mut instance) = self.instances.remove(context) else {
            return Ok(());
        };
        if let Some(event) = ActionEvent::from_message(message) {
            isolate(instance.handle(event, self.sender.clone())).await?;
        }
        Ok(())
    }

    /// Test helper.
    #[cfg(test)]
    pub fn instance(&mut self, context: &str) -> Option<&mut Box<dyn ActionInstance>> {
        self.instances.get_mut(context)
    }
}

fn nested_str(message: &IncomingMessage, key: &str) -> String {
    message
        .payload
        .as_ref()
        .and_then(|value| value.get(key))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn nested_value(message: &IncomingMessage, key: &str) -> Value {
    message
        .payload
        .as_ref()
        .and_then(|value| value.get(key))
        .cloned()
        .unwrap_or(Value::Null)
}

async fn isolate<F>(future: F) -> Result<()>
where
    F: std::future::Future<Output = Result<()>> + Send,
{
    match AssertUnwindSafe(future).catch_unwind().await {
        Ok(result) => result,
        Err(panic) => {
            let message = panic
                .downcast_ref::<&str>()
                .map(|value| (*value).to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic".to_string());
            tracing::error!(panic = %message, "action handler panicked");
            Ok(())
        }
    }
}
