use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::events::{Controller, Coordinates};

/// Common payload fields shared by action-scoped Stream Deck events.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionPayload {
    /// Controller that owns the instance.
    pub controller: Controller,
    /// Slot coordinates when present.
    pub coordinates: Option<Coordinates>,
    /// True when the event came from a multi-action.
    pub is_in_multi_action: bool,
    /// Current state index.
    pub state: Option<i32>,
    /// User-desired state index.
    pub user_desired_state: Option<i32>,
    /// Resource map from `didReceiveResources`.
    pub resources: BTreeMap<String, String>,
    /// Raw settings object.
    pub settings: Value,
}

/// Encoder rotation payload.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DialRotatePayload {
    /// Shared action fields.
    pub action: ActionPayload,
    /// Whether the encoder is pressed while rotating.
    pub pressed: bool,
    /// Signed tick count.
    pub ticks: i32,
}

/// Touch-strip tap payload. Protocol uses `tapPos: [x, y]`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TouchTapPayload {
    /// Shared action fields.
    pub action: ActionPayload,
    /// True when the tap was a hold.
    pub hold: bool,
    /// X position.
    pub tap_x: i32,
    /// Y position.
    pub tap_y: i32,
}

/// Title style tokens.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleParameters {
    /// Font family.
    #[serde(default)]
    pub font_family: String,
    /// Font size.
    #[serde(default)]
    pub font_size: f64,
    /// Font style.
    #[serde(default)]
    pub font_style: String,
    /// Underline flag.
    #[serde(default)]
    pub font_underline: bool,
    /// Whether the title is shown.
    #[serde(default)]
    pub show_title: bool,
    /// Alignment.
    #[serde(default)]
    pub title_alignment: String,
    /// Color.
    #[serde(default)]
    pub title_color: String,
}

/// `titleParametersDidChange` payload.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TitleParametersPayload {
    /// Shared action fields.
    pub action: ActionPayload,
    /// Current title text.
    pub title: String,
    /// Style tokens.
    pub title_parameters: TitleParameters,
}

/// Parse a common action payload from a JSON object.
pub fn parse_action_payload(payload: &Value) -> ActionPayload {
    let object = payload.as_object();
    ActionPayload {
        controller: object
            .and_then(|o| o.get("controller"))
            .and_then(Value::as_str)
            .map(|value| match value {
                "Encoder" => Controller::Encoder,
                "Neo" => Controller::Neo,
                _ => Controller::Keypad,
            })
            .unwrap_or_default(),
        coordinates: object.and_then(|o| o.get("coordinates")).and_then(|value| {
            Some(Coordinates {
                column: value.get("column")?.as_i64()? as i32,
                row: value.get("row")?.as_i64()? as i32,
            })
        }),
        is_in_multi_action: object
            .and_then(|o| o.get("isInMultiAction"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        state: object
            .and_then(|o| o.get("state"))
            .and_then(Value::as_i64)
            .map(|v| v as i32),
        user_desired_state: object
            .and_then(|o| o.get("userDesiredState"))
            .and_then(Value::as_i64)
            .map(|v| v as i32),
        resources: object
            .and_then(|o| o.get("resources"))
            .and_then(Value::as_object)
            .map(|map| {
                map.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        settings: object
            .and_then(|o| o.get("settings"))
            .cloned()
            .unwrap_or(Value::Null),
    }
}

/// Parse a `dialRotate` payload.
pub fn parse_dial_rotate(payload: &Value) -> DialRotatePayload {
    DialRotatePayload {
        action: parse_action_payload(payload),
        pressed: payload
            .get("pressed")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        ticks: payload.get("ticks").and_then(Value::as_i64).unwrap_or(0) as i32,
    }
}

/// Parse a `touchTap` payload.
pub fn parse_touch_tap(payload: &Value) -> TouchTapPayload {
    let tap = payload.get("tapPos").and_then(Value::as_array);
    TouchTapPayload {
        action: parse_action_payload(payload),
        hold: payload
            .get("hold")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        tap_x: tap
            .and_then(|a| a.first())
            .and_then(Value::as_i64)
            .unwrap_or(0) as i32,
        tap_y: tap
            .and_then(|a| a.get(1))
            .and_then(Value::as_i64)
            .unwrap_or(0) as i32,
    }
}

/// Parse a `titleParametersDidChange` payload.
pub fn parse_title_parameters(payload: &Value) -> TitleParametersPayload {
    TitleParametersPayload {
        action: parse_action_payload(payload),
        title: payload
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        title_parameters: payload
            .get("titleParameters")
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default(),
    }
}
