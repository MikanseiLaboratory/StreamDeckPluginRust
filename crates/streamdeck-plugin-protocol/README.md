# streamdeck-plugin-protocol

Wire types, registration arguments, and optional Tokio WebSocket transport for Elgato Stream Deck **plugin** protocol v2.

This crate talks to the Stream Deck application. It is not a USB HID driver.

Protocol details were implemented independently, with reference to [streamdeck-rs](https://github.com/mdonoughe/streamdeck-rs) (Apache-2.0 / MIT) for device-type coverage and plugins that omit `states`.
