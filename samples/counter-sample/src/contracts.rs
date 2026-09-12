use serde::{Deserialize, Serialize};
use streamdeck_plugin::{export_ts, TS};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct CounterSettings {
    #[serde(default = "default_label")]
    pub label: String,
    #[serde(default = "default_increment")]
    pub increment: i32,
}

impl Default for CounterSettings {
    fn default() -> Self {
        Self {
            label: default_label(),
            increment: default_increment(),
        }
    }
}

fn default_label() -> String {
    "Count".into()
}

fn default_increment() -> i32 {
    1
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct CountChangedMessage {
    #[serde(default = "count_changed_type")]
    pub r#type: String,
    pub count: i32,
}

fn count_changed_type() -> String {
    "countChanged".into()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
pub struct PropertyInspectorCommand {
    #[serde(default)]
    pub r#type: String,
}

export_ts!(CounterSettings);
export_ts!(CountChangedMessage);
export_ts!(PropertyInspectorCommand);
