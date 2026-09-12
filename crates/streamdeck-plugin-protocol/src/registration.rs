use crate::events::RegistrationInfo;
use crate::{Error, Result};

/// Command-line arguments Stream Deck supplies when launching a native plugin.
#[derive(Debug, Clone)]
pub struct RegistrationArguments {
    /// WebSocket port.
    pub port: u16,
    /// Plugin UUID.
    pub plugin_uuid: String,
    /// Registration event name, typically `registerPlugin`.
    pub register_event: String,
    /// Raw `-info` JSON.
    pub info_json: String,
    /// Parsed `-info` payload.
    pub info: RegistrationInfo,
}

impl RegistrationArguments {
    /// Parse `-port`, `-pluginUUID`, `-registerEvent`, and `-info`.
    pub fn parse<I, S>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
        let mut port = 0u16;
        let mut plugin_uuid = String::new();
        let mut register_event = String::new();
        let mut info_json = String::new();

        let mut index = 0;
        while index < args.len() {
            let key = args[index].as_str();
            if !key.starts_with('-') {
                index += 1;
                continue;
            }
            if index + 1 >= args.len() {
                break;
            }
            let value = args[index + 1].as_str();
            index += 2;
            match key {
                "-port" => {
                    port = value.parse().map_err(|_| {
                        Error::Registration(
                            "Stream Deck did not provide a valid -port argument.".into(),
                        )
                    })?;
                }
                "-pluginUUID" => plugin_uuid = value.to_string(),
                "-registerEvent" => register_event = value.to_string(),
                "-info" => info_json = value.to_string(),
                _ => {}
            }
        }

        if port == 0 {
            return Err(Error::Registration(
                "Stream Deck did not provide a valid -port argument.".into(),
            ));
        }
        if plugin_uuid.trim().is_empty() {
            return Err(Error::Registration(
                "Stream Deck did not provide a -pluginUUID argument.".into(),
            ));
        }
        if register_event.trim().is_empty() {
            return Err(Error::Registration(
                "Stream Deck did not provide a -registerEvent argument.".into(),
            ));
        }

        let info = if info_json.trim().is_empty() {
            RegistrationInfo::default()
        } else {
            serde_json::from_str(&info_json).unwrap_or_default()
        };

        Ok(Self {
            port,
            plugin_uuid,
            register_event,
            info_json,
            info,
        })
    }
}
