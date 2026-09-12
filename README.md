# StreamDeckPluginRust

Cross-platform Rust SDK for [Elgato Stream Deck](https://docs.elgato.com/streamdeck/sdk/) **WebSocket plugins**.

This repository is not a USB HID driver. Stream Deck Module / direct hardware protocols are out of scope. The SDK talks to the Stream Deck application over the official plugin WebSocket.

Protocol types were implemented independently. [streamdeck-rs](https://github.com/mdonoughe/streamdeck-rs) (Apache-2.0 / MIT) was used as a reference for device-type coverage and plugins that omit `states`.

## Crates

| Crate | Purpose |
| --- | --- |
| `streamdeck-plugin` | Actions, registry, dispatcher, logging, manifest checks. Depend on this. |
| `streamdeck-plugin-protocol` | Registration, events, commands, optional Tokio transport |
| `streamdeck-plugin-macros` | `#[streamdeck_action]` (re-exported by `streamdeck-plugin`) |

Property Inspector bindings live in [`@mikanseilaboratory/streamdeck-pi-client`](https://github.com/MikanseiLaboratory/streamdeck-pi-client).

## Quick start

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    streamdeck_plugin::Plugin::builder(std::env::args().skip(1))
        .state(AppState::default())
        .add_registered_actions()
        .run()
        .await?;
    Ok(())
}

#[derive(Default)]
struct CounterAction;

#[streamdeck_action(
    uuid = "dev.example.plugin.counter",
    settings = CounterSettings,
    state = AppState,
    controller = Keypad,
)]
impl CounterAction {
    async fn on_key_down(&mut self, _p: &ActionPayload, ctx: &ActionContext<'_>) -> Result<()> {
        ctx.state().store.add(ctx.settings().increment.max(1));
        ctx.show_ok()
    }
}
```

`Action` is also public if you prefer to implement it by hand and call `.add_action::<T>()`.

## Sample

[`samples/counter-sample`](samples/counter-sample) shows a keypad action and a Stream Deck + dial sharing one store, plus a React inspector.

```powershell
./samples/counter-sample/publish.ps1 -Install -Pack
```

## Documentation

- [Getting started](docs/getting-started.md)
- [Property Inspector](docs/property-inspector.md)

## License

Apache License 2.0
