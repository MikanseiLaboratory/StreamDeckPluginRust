# Getting started

StreamDeckPluginRust is a Rust SDK for **Stream Deck WebSocket plugins**. It does not talk to Stream Deck hardware over USB HID.

## 1. Add the crate

```toml
[dependencies]
streamdeck-plugin = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## 2. Define an action

```rust
use streamdeck_plugin::{streamdeck_action, ActionContext, ActionPayload, KeypadAction, Result};

#[derive(Default)]
struct CounterAction;

#[streamdeck_action(
    uuid = "dev.example.myplugin.counter",
    settings = CounterSettings,
    state = AppState,
)]
impl KeypadAction for CounterAction {
    async fn on_key_down(
        &mut self,
        _payload: &ActionPayload,
        ctx: &ActionContext<'_, Self::Settings, Self::State>,
    ) -> Result<()> {
        ctx.update_settings(|settings| settings.count += 1)?;
        ctx.set_title(ctx.settings().count.to_string())
    }
}
```

Settings are kept in sync automatically. When the Property Inspector changes a field, `on_settings_changed` runs. When the plugin updates settings, call `ctx.update_settings` so Stream Deck and the inspector both receive the new values.

The same handlers live on `Action` if you want one trait for every event. Register a handwritten impl with `.add_action::<CounterAction>()`.

## 3. Start the host

```rust
streamdeck_plugin::Plugin::builder(std::env::args().skip(1))
    .state(AppState::default())
    .add_registered_actions()
    .run()
    .await?;
```

Stream Deck launches your executable with `-port`, `-pluginUUID`, `-registerEvent`, and `-info`. The host connects to `ws://127.0.0.1:{port}` and registers itself.

## 4. Share state

Put shared connections and counters on `AppState` and hold them in `Arc` internally. Inject that state once on the builder. Every action instance receives `ctx.state()`.

Implement `PluginService` and pass it to `.service(...)` when something must start and stop with the process.

## 5. Package for Windows and macOS

Your `manifest.json` should include both platforms and `SDKVersion` 2. See `samples/counter-sample/publish.ps1`.
