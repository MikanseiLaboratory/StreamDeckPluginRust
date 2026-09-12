use serde_json::Value;
use streamdeck_plugin::{streamdeck_action, ActionContext, ActionPayload, KeypadAction, Result};

use crate::contracts::{CountChangedMessage, CounterSettings, PropertyInspectorCommand};
use crate::store::AppState;

#[derive(Default)]
pub struct CounterAction;

#[streamdeck_action(
    uuid = "dev.flowingspdg.countersample.rust.counter",
    settings = CounterSettings,
    state = AppState,
)]
impl KeypadAction for CounterAction {
    async fn on_will_appear(
        &mut self,
        _payload: &ActionPayload,
        ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        refresh(ctx)
    }

    async fn on_key_down(
        &mut self,
        _payload: &ActionPayload,
        ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        ctx.state().store.add(ctx.settings().increment.max(1));
        refresh(ctx)
    }

    async fn on_settings_changed(
        &mut self,
        _prev: &Self::Settings,
        ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        refresh(ctx)
    }

    async fn on_property_inspector_did_appear(
        &mut self,
        ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        send_count(ctx)
    }

    async fn on_property_inspector_message(
        &mut self,
        payload: &Value,
        ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        apply_inspector(ctx, payload);
        send_count(ctx)
    }
}

fn refresh(ctx: &ActionContext<'_, CounterSettings, AppState>) -> Result<()> {
    let settings = ctx.settings();
    let label = if settings.label.trim().is_empty() {
        "Count"
    } else {
        settings.label.as_str()
    };
    let increment = settings.increment.max(1);
    ctx.set_title(format!(
        "{label}\n{} (+{increment})",
        ctx.state().store.count()
    ))?;
    send_count(ctx)
}

fn send_count(ctx: &ActionContext<'_, CounterSettings, AppState>) -> Result<()> {
    ctx.send_to_property_inspector(&CountChangedMessage {
        r#type: "countChanged".into(),
        count: ctx.state().store.count(),
    })
}

fn apply_inspector(ctx: &ActionContext<'_, CounterSettings, AppState>, payload: &Value) {
    let command =
        serde_json::from_value::<PropertyInspectorCommand>(payload.clone()).unwrap_or_default();
    let increment = ctx.settings().increment.max(1);
    match command.r#type.as_str() {
        "reset" => {
            ctx.state().store.add(-ctx.state().store.count());
        }
        "add" => {
            ctx.state().store.add(increment);
        }
        _ => {}
    }
}
