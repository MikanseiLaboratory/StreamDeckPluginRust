use std::sync::Arc;

use crate::action::ActionEvent;
use crate::dispatch::Dispatcher;
use crate::protocol::{json as sdjson, CommandSender, IncomingMessage};
use crate::registry::Registry;
use crate::{Action, ActionContext, ActionPayload, Result, TypedInstance};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct RecordingSettings {
    increment: i32,
}

#[derive(Default)]
struct RecordingState {
    events: std::sync::Mutex<Vec<String>>,
}

impl RecordingState {
    fn push(&self, event: impl Into<String>) {
        self.events.lock().unwrap().push(event.into());
    }

    fn snapshot(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }
}

#[derive(Default)]
struct RecordingAction;

#[async_trait]
impl Action for RecordingAction {
    type Settings = RecordingSettings;
    type State = RecordingState;
    const UUID: &'static str = "dev.test.counter";

    fn new(_state: Arc<Self::State>) -> Self {
        Self
    }

    async fn on_will_appear(
        &mut self,
        _payload: &ActionPayload,
        ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        ctx.state().push("willAppear");
        Ok(())
    }

    async fn on_key_down(
        &mut self,
        _payload: &ActionPayload,
        ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        ctx.state()
            .push(format!("keyDown:{}", ctx.settings().increment));
        Ok(())
    }

    async fn on_settings_changed(
        &mut self,
        previous: &Self::Settings,
        ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        ctx.state().push(format!(
            "settings:{}->{}",
            previous.increment,
            ctx.settings().increment
        ));
        Ok(())
    }

    async fn on_property_inspector_message(
        &mut self,
        _payload: &serde_json::Value,
        ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        ctx.state().push("pi");
        Ok(())
    }
}

fn parse(json: &str) -> IncomingMessage {
    sdjson::from_str(json).unwrap()
}

fn dispatcher() -> (
    Dispatcher,
    tokio::sync::mpsc::UnboundedReceiver<String>,
    Arc<RecordingState>,
) {
    let mut registry = Registry::new();
    registry.add(crate::ActionRegistration {
        uuid: RecordingAction::UUID,
        controller: crate::Controller::Keypad,
        factory: |state| {
            let state = state.downcast::<RecordingState>().unwrap();
            Box::new(TypedInstance::<RecordingAction>::new(state))
        },
    });
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let sender = CommandSender::new(tx, "plugin");
    let state = Arc::new(RecordingState::default());
    let dispatcher = Dispatcher::new(
        registry,
        Arc::clone(&state) as Arc<dyn std::any::Any + Send + Sync>,
        sender,
        Vec::new(),
    );
    (dispatcher, rx, state)
}

#[tokio::test]
async fn will_appear_then_key_down_uses_typed_settings() {
    let (mut dispatcher, _rx, state) = dispatcher();
    dispatcher
        .dispatch(parse(
            r#"{
              "event": "willAppear",
              "action": "dev.test.counter",
              "context": "ctx",
              "device": "dev",
              "payload": { "controller": "Keypad", "settings": { "increment": 5 } }
            }"#,
        ))
        .await;
    dispatcher
        .dispatch(parse(
            r#"{
              "event": "keyDown",
              "action": "dev.test.counter",
              "context": "ctx",
              "device": "dev",
              "payload": { "controller": "Keypad", "settings": { "increment": 5 } }
            }"#,
        ))
        .await;
    let events = state.snapshot();
    assert!(events.contains(&"willAppear".to_string()));
    assert!(events.contains(&"keyDown:5".to_string()));
}

#[tokio::test]
async fn did_receive_settings_updates_typed_settings_and_raises_hook() {
    let (mut dispatcher, _rx, state) = dispatcher();
    dispatcher
        .dispatch(parse(
            r#"{
              "event": "willAppear",
              "action": "dev.test.counter",
              "context": "ctx",
              "device": "dev",
              "payload": { "settings": { "increment": 1 } }
            }"#,
        ))
        .await;
    dispatcher
        .dispatch(parse(
            r#"{
              "event": "didReceiveSettings",
              "action": "dev.test.counter",
              "context": "ctx",
              "device": "dev",
              "payload": { "settings": { "increment": 9 } }
            }"#,
        ))
        .await;
    assert!(state.snapshot().contains(&"settings:1->9".to_string()));
}

#[tokio::test]
async fn send_to_plugin_reaches_action() {
    let (mut dispatcher, _rx, state) = dispatcher();
    dispatcher
        .dispatch(parse(
            r#"{
              "event": "willAppear",
              "action": "dev.test.counter",
              "context": "ctx",
              "device": "dev",
              "payload": { "settings": {} }
            }"#,
        ))
        .await;
    dispatcher
        .dispatch(parse(
            r#"{
              "event": "sendToPlugin",
              "action": "dev.test.counter",
              "context": "ctx",
              "payload": { "type": "refresh" }
            }"#,
        ))
        .await;
    assert!(state.snapshot().contains(&"pi".to_string()));
}

#[tokio::test]
async fn will_disappear_removes_instance() {
    let (mut dispatcher, _rx, _state) = dispatcher();
    dispatcher
        .dispatch(parse(
            r#"{
              "event": "willAppear",
              "action": "dev.test.counter",
              "context": "ctx",
              "device": "dev",
              "payload": { "settings": {} }
            }"#,
        ))
        .await;
    dispatcher
        .dispatch(parse(
            r#"{
              "event": "willDisappear",
              "action": "dev.test.counter",
              "context": "ctx",
              "device": "dev",
              "payload": { "settings": {} }
            }"#,
        ))
        .await;
    dispatcher
        .dispatch(parse(
            r#"{
              "event": "keyDown",
              "action": "dev.test.counter",
              "context": "ctx",
              "device": "dev",
              "payload": { "settings": {} }
            }"#,
        ))
        .await;
    assert!(dispatcher.instance("ctx").is_none());
}

#[test]
fn action_event_from_message_reads_key_down() {
    let message = parse(r#"{"event":"keyDown","payload":{"settings":{"increment":1}}}"#);
    assert!(matches!(
        ActionEvent::from_message(&message),
        Some(ActionEvent::KeyDown(_))
    ));
}
