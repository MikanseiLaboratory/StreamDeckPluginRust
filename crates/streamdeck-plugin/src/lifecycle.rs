use async_trait::async_trait;
use serde_json::Value;

use crate::protocol::{CommandSender, DeviceInfo};
use crate::Result;

/// Plugin-wide events that are not bound to an action instance.
#[async_trait]
pub trait PluginLifecycle: Send + Sync + 'static {
    /// `applicationDidLaunch`
    async fn on_application_did_launch(&mut self, _application: &str) -> Result<()> {
        Ok(())
    }

    /// `applicationDidTerminate`
    async fn on_application_did_terminate(&mut self, _application: &str) -> Result<()> {
        Ok(())
    }

    /// `deviceDidConnect`
    async fn on_device_did_connect(&mut self, _device_id: &str, _info: &DeviceInfo) -> Result<()> {
        Ok(())
    }

    /// `deviceDidDisconnect`
    async fn on_device_did_disconnect(&mut self, _device_id: &str) -> Result<()> {
        Ok(())
    }

    /// `deviceDidChange`
    async fn on_device_did_change(&mut self, _device_id: &str, _info: &DeviceInfo) -> Result<()> {
        Ok(())
    }

    /// `systemDidWakeUp`
    async fn on_system_did_wake_up(&mut self) -> Result<()> {
        Ok(())
    }

    /// `didReceiveDeepLink`
    async fn on_did_receive_deep_link(&mut self, _url: &str) -> Result<()> {
        Ok(())
    }

    /// `didReceiveGlobalSettings`
    async fn on_did_receive_global_settings(&mut self, _settings: &Value) -> Result<()> {
        Ok(())
    }

    /// `didReceiveSecrets`
    async fn on_did_receive_secrets(&mut self, _secrets: &Value) -> Result<()> {
        Ok(())
    }
}

/// Optional start/stop hooks for shared services.
///
/// Put connections on [`crate::PluginBuilder::state`] and implement this so every
/// action sees the same client through `ctx.state()`. `sender` is the plugin-wide
/// Stream Deck command channel (`IStreamDeckConnection` in the C# SDK).
#[async_trait]
pub trait PluginService: Send + Sync + 'static {
    /// Called before the WebSocket loop starts.
    async fn start(&self, _sender: CommandSender) -> Result<()> {
        Ok(())
    }

    /// Called after the WebSocket loop ends.
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl PluginService for () {}
