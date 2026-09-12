use serde_json::Value;
use streamdeck_plugin::{
    streamdeck_action, ActionContext, ActionPayload, DialRotatePayload, Result, TriggerDescription,
};

use crate::contracts::{CountChangedMessage, CounterSettings, PropertyInspectorCommand};
use crate::store::AppState;

#[derive(Default)]
pub struct DialAction;

#[streamdeck_action(
    uuid = "dev.flowingspdg.countersample.rust.dial",
    settings = CounterSettings,
    state = AppState,
    controller = Encoder,
)]
impl DialAction {
    async fn on_will_appear(
        &mut self,
        _payload: &ActionPayload,
        ctx: &ActionContext<'_>,
    ) -> Result<()> {
        ctx.set_trigger_description(TriggerDescription {
            rotate: Some("Change shared count".into()),
            push: Some("Reset shared count".into()),
            ..Default::default()
        })?;
        refresh(ctx)
    }

    async fn on_dial_rotate(
        &mut self,
        payload: &DialRotatePayload,
        ctx: &ActionContext<'_>,
    ) -> Result<()> {
        ctx.state()
            .store
            .add(payload.ticks * ctx.settings().increment.max(1));
        Ok(())
    }

    async fn on_dial_down(
        &mut self,
        _payload: &ActionPayload,
        ctx: &ActionContext<'_>,
    ) -> Result<()> {
        ctx.state().store.add(-ctx.state().store.count());
        ctx.show_ok()
    }

    async fn on_settings_changed(
        &mut self,
        _prev: &CounterSettings,
        ctx: &ActionContext<'_>,
    ) -> Result<()> {
        refresh(ctx)
    }

    async fn on_property_inspector_did_appear(&mut self, ctx: &ActionContext<'_>) -> Result<()> {
        send_count(ctx)
    }

    async fn on_property_inspector_message(
        &mut self,
        payload: &Value,
        ctx: &ActionContext<'_>,
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
    ctx.set_title(format!("{label}: {}", ctx.state().store.count()))?;
    ctx.set_feedback(&serde_json::json!({
        "title": label,
        "value": ctx.state().store.count().to_string(),
        "indicator": ctx.state().store.count().clamp(0, 100)
    }))?;
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
