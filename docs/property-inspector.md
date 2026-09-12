# Property Inspector bindings

The goal is that plugin authors never write WebSocket glue. Settings are a Rust struct and a matching React form.

## Rust contract

```rust
use serde::{Deserialize, Serialize};
use streamdeck_plugin::{export_ts, TS};

#[derive(Clone, Default, Serialize, Deserialize, TS)]
pub struct CounterSettings {
    pub increment: i32,
}

export_ts!(CounterSettings);
```

`ctx.settings()` returns the current snapshot. `ctx.update_settings(|s| ...)` persists it. Custom messages still use `ctx.send_to_property_inspector` and `on_property_inspector_message`.

## Generate TypeScript

```bash
cargo run --bin typegen
```

The generator writes camelCase interfaces collected through `inventory`.

## React inspector

Use [`@mikanseilaboratory/streamdeck-pi-client`](https://github.com/MikanseiLaboratory/streamdeck-pi-client):

```tsx
import { StreamDeckProvider, useSettings } from "@mikanseilaboratory/streamdeck-pi-client";
import type { CounterSettings } from "./generated/contracts";

export function App() {
  const { settings, bind } = useSettings<CounterSettings>({ increment: 1 });
  return (
    <div>
      <input type="number" {...bind("increment")} />
      <span>Step: {settings.increment}</span>
    </div>
  );
}
```

`StreamDeckProvider` owns `connectElgatoStreamDeckSocket`. Build the inspector with Vite (`base: './'`) and place the output in `*.sdPlugin/ui/`.
