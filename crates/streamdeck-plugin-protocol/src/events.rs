use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// Incoming WebSocket event names defined by the Stream Deck plugin protocol.
pub struct EventNames;

impl EventNames {
    /// `applicationDidLaunch`
    pub const APPLICATION_DID_LAUNCH: &'static str = "applicationDidLaunch";
    /// `applicationDidTerminate`
    pub const APPLICATION_DID_TERMINATE: &'static str = "applicationDidTerminate";
    /// `deviceDidChange`
    pub const DEVICE_DID_CHANGE: &'static str = "deviceDidChange";
    /// `deviceDidConnect`
    pub const DEVICE_DID_CONNECT: &'static str = "deviceDidConnect";
    /// `deviceDidDisconnect`
    pub const DEVICE_DID_DISCONNECT: &'static str = "deviceDidDisconnect";
    /// `dialDown`
    pub const DIAL_DOWN: &'static str = "dialDown";
    /// `dialRotate`
    pub const DIAL_ROTATE: &'static str = "dialRotate";
    /// `dialUp`
    pub const DIAL_UP: &'static str = "dialUp";
    /// `didReceiveDeepLink`
    pub const DID_RECEIVE_DEEP_LINK: &'static str = "didReceiveDeepLink";
    /// `didReceiveGlobalSettings`
    pub const DID_RECEIVE_GLOBAL_SETTINGS: &'static str = "didReceiveGlobalSettings";
    /// `sendToPlugin`
    pub const SEND_TO_PLUGIN: &'static str = "sendToPlugin";
    /// `didReceiveResources`
    pub const DID_RECEIVE_RESOURCES: &'static str = "didReceiveResources";
    /// `didReceiveSecrets`
    pub const DID_RECEIVE_SECRETS: &'static str = "didReceiveSecrets";
    /// `didReceiveSettings`
    pub const DID_RECEIVE_SETTINGS: &'static str = "didReceiveSettings";
    /// `keyDown`
    pub const KEY_DOWN: &'static str = "keyDown";
    /// `keyUp`
    pub const KEY_UP: &'static str = "keyUp";
    /// `propertyInspectorDidAppear`
    pub const PROPERTY_INSPECTOR_DID_APPEAR: &'static str = "propertyInspectorDidAppear";
    /// `propertyInspectorDidDisappear`
    pub const PROPERTY_INSPECTOR_DID_DISAPPEAR: &'static str = "propertyInspectorDidDisappear";
    /// `systemDidWakeUp`
    pub const SYSTEM_DID_WAKE_UP: &'static str = "systemDidWakeUp";
    /// `titleParametersDidChange`
    pub const TITLE_PARAMETERS_DID_CHANGE: &'static str = "titleParametersDidChange";
    /// `touchTap`
    pub const TOUCH_TAP: &'static str = "touchTap";
    /// `willAppear`
    pub const WILL_APPEAR: &'static str = "willAppear";
    /// `willDisappear`
    pub const WILL_DISAPPEAR: &'static str = "willDisappear";
}

/// Hardware family identifiers used by the WebSocket plugin protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DeviceType {
    /// Classic Stream Deck.
    StreamDeck = 0,
    /// Stream Deck Mini.
    StreamDeckMini = 1,
    /// Stream Deck XL.
    StreamDeckXl = 2,
    /// Stream Deck Mobile.
    StreamDeckMobile = 3,
    /// Corsair G Keys.
    CorsairGKeys = 4,
    /// Stream Deck Pedal.
    StreamDeckPedal = 5,
    /// Corsair Voyager.
    CorsairVoyager = 6,
    /// Stream Deck +.
    StreamDeckPlus = 7,
    /// SCUF controller.
    ScufController = 8,
    /// Stream Deck Neo.
    StreamDeckNeo = 9,
    /// Stream Deck Studio.
    StreamDeckStudio = 10,
    /// Virtual Stream Deck.
    VirtualStreamDeck = 11,
    /// Galleon 100 SD.
    Galleon100Sd = 12,
    /// Stream Deck + XL.
    StreamDeckPlusXl = 13,
}

impl DeviceType {
    fn from_u32(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::StreamDeck,
            1 => Self::StreamDeckMini,
            2 => Self::StreamDeckXl,
            3 => Self::StreamDeckMobile,
            4 => Self::CorsairGKeys,
            5 => Self::StreamDeckPedal,
            6 => Self::CorsairVoyager,
            7 => Self::StreamDeckPlus,
            8 => Self::ScufController,
            9 => Self::StreamDeckNeo,
            10 => Self::StreamDeckStudio,
            11 => Self::VirtualStreamDeck,
            12 => Self::Galleon100Sd,
            13 => Self::StreamDeckPlusXl,
            _ => return None,
        })
    }
}

impl Serialize for DeviceType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(*self as u32)
    }
}

impl<'de> Deserialize<'de> for DeviceType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u32::deserialize(deserializer)?;
        Self::from_u32(value)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown device type {value}")))
    }
}

/// Controller kind that owns an action instance.
///
/// The protocol sends PascalCase strings (`Keypad`, `Encoder`, `Neo`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Controller {
    /// Face buttons.
    #[default]
    Keypad,
    /// Stream Deck + encoder / touch strip.
    Encoder,
    /// Stream Deck Neo.
    Neo,
}

impl Controller {
    fn as_protocol_str(self) -> &'static str {
        match self {
            Self::Keypad => "Keypad",
            Self::Encoder => "Encoder",
            Self::Neo => "Neo",
        }
    }
}

impl Serialize for Controller {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_protocol_str())
    }
}

impl<'de> Deserialize<'de> for Controller {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "Encoder" => Self::Encoder,
            "Neo" => Self::Neo,
            _ => Self::Keypad,
        })
    }
}

/// Destination for visual updates such as titles and images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Target {
    /// Hardware and software (default).
    #[default]
    HardwareAndSoftware = 0,
    /// Hardware only.
    Hardware = 1,
    /// Software only.
    Software = 2,
}

impl Serialize for Target {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for Target {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u8::deserialize(deserializer)?;
        Ok(match value {
            1 => Self::Hardware,
            2 => Self::Software,
            _ => Self::HardwareAndSoftware,
        })
    }
}

/// Zero-based slot coordinates on a Stream Deck device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coordinates {
    /// Column index.
    pub column: i32,
    /// Row index.
    pub row: i32,
}

/// Key grid size reported for a connected device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSize {
    /// Column count.
    pub columns: i32,
    /// Row count.
    pub rows: i32,
}

/// Device metadata provided during registration and device events.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    /// Device identifier.
    #[serde(default)]
    pub id: String,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Button grid size.
    #[serde(default)]
    pub size: DeviceSize,
    /// Hardware family.
    #[serde(default)]
    pub r#type: Option<DeviceType>,
}

/// Host application metadata from `-info`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationInfo {
    /// UI font.
    #[serde(default)]
    pub font: String,
    /// UI language.
    #[serde(default)]
    pub language: String,
    /// `windows` or `mac`.
    #[serde(default)]
    pub platform: String,
    /// OS version.
    #[serde(default)]
    pub platform_version: String,
    /// Stream Deck application version.
    #[serde(default)]
    pub version: String,
}

/// Color tokens from `-info`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorInfo {
    /// Hover background.
    #[serde(default)]
    pub button_mouse_over_background_color: String,
    /// Pressed background.
    #[serde(default)]
    pub button_pressed_background_color: String,
    /// Pressed border.
    #[serde(default)]
    pub button_pressed_border_color: String,
    /// Pressed text.
    #[serde(default)]
    pub button_pressed_text_color: String,
    /// Highlight.
    #[serde(default)]
    pub highlight_color: String,
}

/// Plugin identity from `-info`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    /// Plugin UUID.
    #[serde(default)]
    pub uuid: String,
    /// Plugin version.
    #[serde(default)]
    pub version: String,
}

/// Startup information supplied through the `-info` argument.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationInfo {
    /// Host application.
    #[serde(default)]
    pub application: ApplicationInfo,
    /// Color tokens.
    #[serde(default)]
    pub colors: ColorInfo,
    /// Display scale.
    #[serde(default)]
    pub device_pixel_ratio: f64,
    /// Connected devices at launch.
    #[serde(default)]
    pub devices: Vec<DeviceInfo>,
    /// Plugin identity.
    #[serde(default)]
    pub plugin: PluginInfo,
}

/// Raw inbound Stream Deck message before action-specific dispatch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingMessage {
    /// Event name.
    pub event: String,
    /// Action UUID.
    #[serde(default)]
    pub action: Option<String>,
    /// Instance context.
    #[serde(default)]
    pub context: Option<String>,
    /// Device identifier.
    #[serde(default)]
    pub device: Option<String>,
    /// Optional request id.
    #[serde(default)]
    pub id: Option<String>,
    /// Present on device connect/change.
    #[serde(default)]
    pub device_info: Option<DeviceInfo>,
    /// Event payload.
    #[serde(default)]
    pub payload: Option<Value>,
}

impl IncomingMessage {
    /// True when a JSON payload object or value is present.
    pub fn has_payload(&self) -> bool {
        !matches!(self.payload, None | Some(Value::Null))
    }
}
