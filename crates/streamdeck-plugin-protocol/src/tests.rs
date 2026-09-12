use crate::events::{Controller, DeviceType, EventNames, IncomingMessage};
use crate::payload::{parse_action_payload, parse_dial_rotate};
use crate::registration::RegistrationArguments;
use crate::{json, CommandNames, OutgoingCommand};

#[test]
fn incoming_key_down_round_trips_core_fields() {
    let json = r#"{
        "action": "dev.test.counter",
        "context": "ctx-1",
        "device": "dev-1",
        "event": "keyDown",
        "payload": {
            "controller": "Keypad",
            "coordinates": { "column": 2, "row": 1 },
            "isInMultiAction": false,
            "settings": { "increment": 3 }
        }
    }"#;

    let message: IncomingMessage = json::from_str(json).unwrap();
    assert_eq!(message.event, EventNames::KEY_DOWN);
    assert_eq!(message.context.as_deref(), Some("ctx-1"));
    let payload = parse_action_payload(message.payload.as_ref().unwrap());
    assert_eq!(payload.controller, Controller::Keypad);
    assert_eq!(payload.coordinates.unwrap().column, 2);
    assert_eq!(
        payload.settings.get("increment").and_then(|v| v.as_i64()),
        Some(3)
    );
}

#[test]
fn incoming_device_did_connect_reads_device_info() {
    let json = r#"{
        "event": "deviceDidConnect",
        "device": "abc",
        "deviceInfo": {
            "name": "Stream Deck XL",
            "size": { "columns": 8, "rows": 4 },
            "type": 2
        }
    }"#;
    let message: IncomingMessage = json::from_str(json).unwrap();
    assert_eq!(message.event, EventNames::DEVICE_DID_CONNECT);
    assert_eq!(message.device.as_deref(), Some("abc"));
    let info = message.device_info.unwrap();
    assert_eq!(info.r#type, Some(DeviceType::StreamDeckXl));
    assert_eq!(info.size.columns, 8);
}

#[test]
fn incoming_dial_rotate_and_deep_link_parse() {
    let rotate: IncomingMessage = json::from_str(
        r#"{
            "event": "dialRotate",
            "action": "dev.test.dial",
            "context": "c",
            "device": "d",
            "payload": { "controller": "Encoder", "ticks": -2, "pressed": true, "settings": {} }
        }"#,
    )
    .unwrap();
    let payload = parse_dial_rotate(rotate.payload.as_ref().unwrap());
    assert_eq!(payload.ticks, -2);
    assert!(payload.pressed);
    assert_eq!(payload.action.controller, Controller::Encoder);

    let deep_link: IncomingMessage =
        json::from_str(r#"{ "event": "didReceiveDeepLink", "payload": { "url": "hello/world" } }"#)
            .unwrap();
    assert_eq!(
        deep_link
            .payload
            .unwrap()
            .get("url")
            .and_then(|v| v.as_str()),
        Some("hello/world")
    );
}

#[test]
fn registration_info_deserializes() {
    let args = RegistrationArguments::parse([
        "-port",
        "28196",
        "-pluginUUID",
        "ABC123",
        "-registerEvent",
        "registerPlugin",
        "-info",
        r#"{"plugin":{"uuid":"dev.example","version":"1.0.0"},"application":{"platform":"mac"},"devices":[]}"#,
    ])
    .unwrap();
    assert_eq!(args.port, 28196);
    assert_eq!(args.plugin_uuid, "ABC123");
    assert_eq!(args.register_event, "registerPlugin");
    assert_eq!(args.info.application.platform, "mac");
    assert_eq!(args.info.plugin.uuid, "dev.example");
}

#[test]
fn registration_parse_skips_program_name() {
    let args = RegistrationArguments::parse([
        r"C:\Plugins\counter.exe",
        "-port",
        "28196",
        "-pluginUUID",
        "ABC123",
        "-registerEvent",
        "registerPlugin",
    ])
    .unwrap();
    assert_eq!(args.port, 28196);
    assert_eq!(args.plugin_uuid, "ABC123");
}

#[test]
fn command_serialization_uses_event_property() {
    let command = OutgoingCommand::new(CommandNames::SET_TITLE)
        .with_context("c")
        .with_payload(serde_json::json!({ "title": "Hi" }));
    let json = json::to_string(&command).unwrap().replace(' ', "");
    assert!(json.contains(r#""event":"setTitle""#));
    assert!(json.contains(r#""title":"Hi""#));
}
