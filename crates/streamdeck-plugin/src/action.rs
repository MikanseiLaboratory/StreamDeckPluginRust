use std::sync::Arc;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::context::{ActionContext, ActionIdentity};
use crate::protocol::{
    parse_action_payload, parse_dial_rotate, parse_title_parameters, parse_touch_tap,
    ActionPayload, CommandSender, Coordinates, DialRotatePayload, IncomingMessage,
    TitleParametersPayload, TouchTapPayload,
};
use crate::Result;

/// Typed action implemented by plugin authors.
#[async_trait]
pub trait Action: Send + Sync + 'static {
    /// Settings deserialized from Stream Deck.
    type Settings: Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static;
    /// Shared plugin state.
    type State: Send + Sync + 'static;
    /// Manifest / protocol UUID.
    const UUID: &'static str;

    /// Create an instance. The host already holds `state`; use it only if the
    /// action struct itself needs a copy.
    fn new(state: Arc<Self::State>) -> Self;

    /// `willAppear`
    async fn on_will_appear(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `willDisappear`
    async fn on_will_disappear(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `didReceiveSettings` after typed settings are applied.
    async fn on_did_receive_settings(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// Fired after settings change. `previous` is the last snapshot.
    async fn on_settings_changed(
        &mut self,
        _previous: &Self::Settings,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `propertyInspectorDidAppear`
    async fn on_property_inspector_did_appear(
        &mut self,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `propertyInspectorDidDisappear`
    async fn on_property_inspector_did_disappear(
        &mut self,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `sendToPlugin`
    async fn on_property_inspector_message(
        &mut self,
        _payload: &Value,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `titleParametersDidChange`
    async fn on_title_parameters_did_change(
        &mut self,
        _payload: &TitleParametersPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `didReceiveResources`
    async fn on_did_receive_resources(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `keyDown`
    async fn on_key_down(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `keyUp`
    async fn on_key_up(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `dialDown`
    async fn on_dial_down(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `dialUp`
    async fn on_dial_up(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `dialRotate`
    async fn on_dial_rotate(
        &mut self,
        _payload: &DialRotatePayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `touchTap`
    async fn on_touch_tap(
        &mut self,
        _payload: &TouchTapPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
            Ok(())
    }
}

/// Keypad handlers. Implement this for keys so rust-analyzer can complete methods.
#[async_trait]
pub trait KeypadAction: Send + Sync + 'static {
    /// Settings deserialized from Stream Deck.
    type Settings: Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static;
    /// Shared plugin state.
    type State: Send + Sync + 'static;

    /// `willAppear`
    async fn on_will_appear(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `willDisappear`
    async fn on_will_disappear(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `didReceiveSettings` after typed settings are applied.
    async fn on_did_receive_settings(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// Fired after settings change. `previous` is the last snapshot.
    async fn on_settings_changed(
        &mut self,
        _previous: &Self::Settings,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `propertyInspectorDidAppear`
    async fn on_property_inspector_did_appear(
        &mut self,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `propertyInspectorDidDisappear`
    async fn on_property_inspector_did_disappear(
        &mut self,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `sendToPlugin`
    async fn on_property_inspector_message(
        &mut self,
        _payload: &Value,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `titleParametersDidChange`
    async fn on_title_parameters_did_change(
        &mut self,
        _payload: &TitleParametersPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `didReceiveResources`
    async fn on_did_receive_resources(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `keyDown`
    async fn on_key_down(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `keyUp`
    async fn on_key_up(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }
}

/// Encoder handlers. Implement this for dials / touch strips.
#[async_trait]
pub trait EncoderAction: Send + Sync + 'static {
    /// Settings deserialized from Stream Deck.
    type Settings: Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static;
    /// Shared plugin state.
    type State: Send + Sync + 'static;

    /// `willAppear`
    async fn on_will_appear(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `willDisappear`
    async fn on_will_disappear(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `didReceiveSettings` after typed settings are applied.
    async fn on_did_receive_settings(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// Fired after settings change. `previous` is the last snapshot.
    async fn on_settings_changed(
        &mut self,
        _previous: &Self::Settings,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `propertyInspectorDidAppear`
    async fn on_property_inspector_did_appear(
        &mut self,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `propertyInspectorDidDisappear`
    async fn on_property_inspector_did_disappear(
        &mut self,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `sendToPlugin`
    async fn on_property_inspector_message(
        &mut self,
        _payload: &Value,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `titleParametersDidChange`
    async fn on_title_parameters_did_change(
        &mut self,
        _payload: &TitleParametersPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `didReceiveResources`
    async fn on_did_receive_resources(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `dialDown`
    async fn on_dial_down(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `dialUp`
    async fn on_dial_up(
        &mut self,
        _payload: &ActionPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `dialRotate`
    async fn on_dial_rotate(
        &mut self,
        _payload: &DialRotatePayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }

    /// `touchTap`
    async fn on_touch_tap(
        &mut self,
        _payload: &TouchTapPayload,
        _ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        Ok(())
    }
}

/// Object-safe instance stored by the dispatcher.
#[async_trait]
pub trait ActionInstance: Send {
    /// Bind context metadata from `willAppear`.
    fn attach(
        &mut self,
        context: String,
        action_id: String,
        device_id: String,
        coordinates: Option<Coordinates>,
    );
    /// Replace typed settings from a JSON object.
    fn apply_settings(&mut self, settings: &Value);
    /// Current settings snapshot.
    fn current_settings(&self) -> Value;
    /// Dispatch one action-scoped event.
    async fn handle(&mut self, event: ActionEvent, sender: CommandSender) -> Result<()>;
}

/// Normalized action-scoped event.
#[derive(Debug, Clone)]
pub enum ActionEvent {
    /// `willAppear`
    WillAppear(ActionPayload),
    /// `willDisappear`
    WillDisappear(ActionPayload),
    /// `keyDown`
    KeyDown(ActionPayload),
    /// `keyUp`
    KeyUp(ActionPayload),
    /// `dialDown`
    DialDown(ActionPayload),
    /// `dialUp`
    DialUp(ActionPayload),
    /// `dialRotate`
    DialRotate(DialRotatePayload),
    /// `touchTap`
    TouchTap(TouchTapPayload),
    /// `didReceiveSettings`
    DidReceiveSettings(ActionPayload),
    /// `sendToPlugin`
    PropertyInspectorMessage(Value),
    /// `propertyInspectorDidAppear`
    PropertyInspectorDidAppear,
    /// `propertyInspectorDidDisappear`
    PropertyInspectorDidDisappear,
    /// `titleParametersDidChange`
    TitleParametersDidChange(TitleParametersPayload),
    /// `didReceiveResources`
    DidReceiveResources(ActionPayload),
}

impl ActionEvent {
    /// Build an event from a raw inbound message.
    pub fn from_message(message: &IncomingMessage) -> Option<Self> {
        use crate::protocol::EventNames;
        let payload = message.payload.clone().unwrap_or(Value::Null);
        Some(match message.event.as_str() {
            EventNames::WILL_APPEAR => Self::WillAppear(parse_action_payload(&payload)),
            EventNames::WILL_DISAPPEAR => Self::WillDisappear(parse_action_payload(&payload)),
            EventNames::KEY_DOWN => Self::KeyDown(parse_action_payload(&payload)),
            EventNames::KEY_UP => Self::KeyUp(parse_action_payload(&payload)),
            EventNames::DIAL_DOWN => Self::DialDown(parse_action_payload(&payload)),
            EventNames::DIAL_UP => Self::DialUp(parse_action_payload(&payload)),
            EventNames::DIAL_ROTATE => Self::DialRotate(parse_dial_rotate(&payload)),
            EventNames::TOUCH_TAP => Self::TouchTap(parse_touch_tap(&payload)),
            EventNames::DID_RECEIVE_SETTINGS => {
                Self::DidReceiveSettings(parse_action_payload(&payload))
            }
            EventNames::SEND_TO_PLUGIN => Self::PropertyInspectorMessage(payload),
            EventNames::PROPERTY_INSPECTOR_DID_APPEAR => Self::PropertyInspectorDidAppear,
            EventNames::PROPERTY_INSPECTOR_DID_DISAPPEAR => Self::PropertyInspectorDidDisappear,
            EventNames::TITLE_PARAMETERS_DID_CHANGE => {
                Self::TitleParametersDidChange(parse_title_parameters(&payload))
            }
            EventNames::DID_RECEIVE_RESOURCES => {
                Self::DidReceiveResources(parse_action_payload(&payload))
            }
            _ => return None,
        })
    }
}

/// Adapter from a typed [`Action`] to [`ActionInstance`].
pub struct TypedInstance<A: Action> {
    action: A,
    settings: A::Settings,
    state: Arc<A::State>,
    identity: ActionIdentity,
}

impl<A: Action> TypedInstance<A> {
    /// Create an instance from shared state.
    pub fn new(state: Arc<A::State>) -> Self {
        Self {
            action: A::new(Arc::clone(&state)),
            settings: A::Settings::default(),
            state,
            identity: ActionIdentity {
                context: String::new(),
                action_id: A::UUID.to_string(),
                device_id: String::new(),
                coordinates: None,
            },
        }
    }
}

#[async_trait]
impl<A: Action> ActionInstance for TypedInstance<A> {
    fn attach(
        &mut self,
        context: String,
        action_id: String,
        device_id: String,
        coordinates: Option<Coordinates>,
    ) {
        self.identity = ActionIdentity {
            context,
            action_id,
            device_id,
            coordinates,
        };
    }

    fn apply_settings(&mut self, settings: &Value) {
        self.settings = if settings.is_object() {
            serde_json::from_value(settings.clone()).unwrap_or_default()
        } else {
            A::Settings::default()
        };
    }

    fn current_settings(&self) -> Value {
        serde_json::to_value(&self.settings).unwrap_or(Value::Null)
    }

    async fn handle(&mut self, event: ActionEvent, sender: CommandSender) -> Result<()> {
        if let ActionEvent::DidReceiveSettings(payload) = &event {
            let previous = self.settings.clone();
            self.apply_settings(&payload.settings);
            let ctx = ActionContext::new(
                self.identity.clone(),
                self.settings.clone(),
                &self.state,
                sender,
            );
            self.action.on_did_receive_settings(payload, &ctx).await?;
            let result = self.action.on_settings_changed(&previous, &ctx).await;
            self.settings = ctx.into_settings();
            return result;
        }

        let ctx = ActionContext::new(
            self.identity.clone(),
            self.settings.clone(),
            &self.state,
            sender,
        );
        let result = match &event {
            ActionEvent::WillAppear(payload) => self.action.on_will_appear(payload, &ctx).await,
            ActionEvent::WillDisappear(payload) => {
                self.action.on_will_disappear(payload, &ctx).await
            }
            ActionEvent::KeyDown(payload) => self.action.on_key_down(payload, &ctx).await,
            ActionEvent::KeyUp(payload) => self.action.on_key_up(payload, &ctx).await,
            ActionEvent::DialDown(payload) => self.action.on_dial_down(payload, &ctx).await,
            ActionEvent::DialUp(payload) => self.action.on_dial_up(payload, &ctx).await,
            ActionEvent::DialRotate(payload) => self.action.on_dial_rotate(payload, &ctx).await,
            ActionEvent::TouchTap(payload) => self.action.on_touch_tap(payload, &ctx).await,
            ActionEvent::DidReceiveSettings(_) => unreachable!(),
            ActionEvent::PropertyInspectorMessage(payload) => {
                self.action
                    .on_property_inspector_message(payload, &ctx)
                    .await
            }
            ActionEvent::PropertyInspectorDidAppear => {
                self.action.on_property_inspector_did_appear(&ctx).await
            }
            ActionEvent::PropertyInspectorDidDisappear => {
                self.action.on_property_inspector_did_disappear(&ctx).await
            }
            ActionEvent::TitleParametersDidChange(payload) => {
                self.action
                    .on_title_parameters_did_change(payload, &ctx)
                    .await
            }
            ActionEvent::DidReceiveResources(payload) => {
                self.action.on_did_receive_resources(payload, &ctx).await
            }
        };
        self.settings = ctx.into_settings();
        result
    }
}
