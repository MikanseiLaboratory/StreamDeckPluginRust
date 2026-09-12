import { usePluginMessage, useSendToPlugin, useSettings } from "@mikanseilaboratory/streamdeck-pi-client";
import { useState } from "react";
import type { CountChangedMessage, CounterSettings, PropertyInspectorCommand } from "./generated/contracts";

export function App() {
  const { settings, bind } = useSettings<CounterSettings>({ label: "Count", increment: 1 });
  const [count, setCount] = useState(0);
  const sendToPlugin = useSendToPlugin();

  usePluginMessage<CountChangedMessage>((message) => {
    if (message.type === "countChanged") {
      setCount(message.count);
    }
  });

  const command = (type: PropertyInspectorCommand["type"]) => {
    sendToPlugin<PropertyInspectorCommand>({ type });
  };

  return (
    <div className="sdpi-wrapper">
      <div className="sdpi-heading">SETTINGS</div>
      <div className="sdpi-item">
        <div className="sdpi-item-label">Label</div>
        <input className="sdpi-item-value" type="text" {...bind("label")} />
      </div>
      <div className="sdpi-item">
        <div className="sdpi-item-label">Increment</div>
        <input className="sdpi-item-value" type="number" {...bind("increment")} />
      </div>

      <div className="sdpi-heading">COUNTER</div>
      <div className="sdpi-item">
        <div className="sdpi-item-label">Value</div>
        <input className="sdpi-item-value" type="text" value={String(count)} readOnly />
      </div>
      <div className="sdpi-item">
        <div className="sdpi-item-label">Actions</div>
        <button className="sdpi-item-value" type="button" onClick={() => command("add")}>
          Add {settings.increment || 1}
        </button>
        <button className="sdpi-item-value" type="button" onClick={() => command("reset")}>
          Reset
        </button>
        <button className="sdpi-item-value" type="button" onClick={() => command("refresh")}>
          Refresh
        </button>
      </div>
    </div>
  );
}
