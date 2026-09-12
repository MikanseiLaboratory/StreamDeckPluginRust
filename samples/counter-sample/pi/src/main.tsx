import React from "react";
import ReactDOM from "react-dom/client";
import { StreamDeckProvider } from "@mikanseilaboratory/streamdeck-pi-client";
import { App } from "./App";
import "./sdpi.css";
import "./app.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <StreamDeckProvider>
      <App />
    </StreamDeckProvider>
  </React.StrictMode>
);
