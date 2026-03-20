use serde::{Deserialize, Serialize};

// ---------- Source ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Usb,
    Wifi,
    Bluetooth,
    Tv,
    Optical,
    Coaxial,
    Analog,
    Standby,
}

impl Source {
    pub const ALL: &[Source] = &[
        Source::Wifi,
        Source::Bluetooth,
        Source::Tv,
        Source::Optical,
        Source::Coaxial,
        Source::Analog,
        Source::Usb,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            Source::Usb => "USB",
            Source::Wifi => "Wi-Fi",
            Source::Bluetooth => "Bluetooth",
            Source::Tv => "TV (HDMI)",
            Source::Optical => "Optical",
            Source::Coaxial => "Coaxial",
            Source::Analog => "Analog",
            Source::Standby => "Standby",
        }
    }
}

// ---------- Cable Mode ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum CableMode {
    #[default]
    Wired,
    Wireless,
}

impl CableMode {
    pub fn display_name(self) -> &'static str {
        match self {
            CableMode::Wired => "Wired",
            CableMode::Wireless => "Wireless",
        }
    }
}

// ---------- Standby Mode ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StandbyMode {
    #[serde(rename = "standby_20mins")]
    TwentyMinutes,
    #[serde(rename = "standby_30mins")]
    ThirtyMinutes,
    #[serde(rename = "standby_60mins")]
    #[default]
    SixtyMinutes,
    #[serde(rename = "standby_off")]
    Never,
}

impl StandbyMode {
    pub fn display_name(self) -> &'static str {
        match self {
            StandbyMode::TwentyMinutes => "20 minutes",
            StandbyMode::ThirtyMinutes => "30 minutes",
            StandbyMode::SixtyMinutes => "60 minutes",
            StandbyMode::Never => "Never",
        }
    }

    pub fn cycle_next(self) -> Self {
        match self {
            StandbyMode::TwentyMinutes => StandbyMode::ThirtyMinutes,
            StandbyMode::ThirtyMinutes => StandbyMode::SixtyMinutes,
            StandbyMode::SixtyMinutes => StandbyMode::Never,
            StandbyMode::Never => StandbyMode::TwentyMinutes,
        }
    }

    pub fn cycle_prev(self) -> Self {
        match self {
            StandbyMode::TwentyMinutes => StandbyMode::Never,
            StandbyMode::ThirtyMinutes => StandbyMode::TwentyMinutes,
            StandbyMode::SixtyMinutes => StandbyMode::ThirtyMinutes,
            StandbyMode::Never => StandbyMode::SixtyMinutes,
        }
    }
}

// ---------- Speaker Status ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SpeakerStatus {
    #[default]
    PowerOn,
    Standby,
}

// ---------- API Value (typed-value wrapper) ----------

/// The KEF API wraps values in a tagged union like:
/// `{"type": "i32_", "i32_": 50}`
/// `{"type": "kefPhysicalSource", "kefPhysicalSource": "usb"}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ApiValue {
    #[serde(rename = "i32_")]
    I32 {
        #[serde(rename = "i32_")]
        value: i32,
    },
    #[serde(rename = "i64_")]
    I64 {
        #[serde(rename = "i64_")]
        value: i64,
    },
    #[serde(rename = "string_")]
    String {
        #[serde(rename = "string_")]
        value: String,
    },
    #[serde(rename = "bool_")]
    Bool {
        #[serde(rename = "bool_")]
        value: bool,
    },
    #[serde(rename = "kefPhysicalSource")]
    PhysicalSource {
        #[serde(rename = "kefPhysicalSource")]
        value: Source,
    },
    #[serde(rename = "kefSpeakerStatus")]
    SpeakerStatus {
        #[serde(rename = "kefSpeakerStatus")]
        value: SpeakerStatus,
    },
    #[serde(rename = "kefCableMode")]
    CableMode {
        #[serde(rename = "kefCableMode")]
        value: CableMode,
    },
    #[serde(rename = "kefStandbyMode")]
    StandbyMode {
        #[serde(rename = "kefStandbyMode")]
        value: StandbyMode,
    },
}

impl ApiValue {
    pub fn i32(value: i32) -> Self {
        ApiValue::I32 { value }
    }

    pub fn i64(value: i64) -> Self {
        ApiValue::I64 { value }
    }

    pub fn string(value: impl Into<String>) -> Self {
        ApiValue::String {
            value: value.into(),
        }
    }

    pub fn bool(value: bool) -> Self {
        ApiValue::Bool { value }
    }

    pub fn source(value: Source) -> Self {
        ApiValue::PhysicalSource { value }
    }

    pub fn speaker_status(value: SpeakerStatus) -> Self {
        ApiValue::SpeakerStatus { value }
    }

    pub fn cable_mode(value: CableMode) -> Self {
        ApiValue::CableMode { value }
    }
}

// ---------- getData Response ----------

/// Response from `/api/getData`. The API returns an array where each element
/// has a `type` field and a same-named field with the value.
pub type GetDataResponse = Vec<ApiValue>;

// ---------- setData Request ----------

#[derive(Debug, Clone, Serialize)]
pub struct SetDataRequest {
    pub path: String,
    pub roles: String,
    pub value: ApiValue,
}

impl SetDataRequest {
    pub fn new(path: impl Into<String>, value: ApiValue) -> Self {
        Self {
            path: path.into(),
            roles: "value".to_string(),
            value,
        }
    }
}

// ---------- Player Data ----------

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PlayerData {
    #[serde(rename = "trackRoles")]
    pub track_roles: Option<TrackRoles>,
    pub state: Option<String>,
    #[serde(rename = "mediaRoles")]
    pub media_roles: Option<MediaRoles>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TrackRoles {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub icon: Option<String>,
    pub media_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MediaRoles {
    pub duration: Option<f64>,
    #[serde(rename = "playTime")]
    pub play_time: Option<f64>,
}

// ---------- EQ ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqProfile {
    pub name: String,
    pub treble: f64,
    pub bass_extension: BassExtension,
    pub desk_mode: bool,
    pub desk_db: f64,
    pub wall_mode: bool,
    pub wall_db: f64,
    pub sub_out: bool,
    pub sub_gain: f64,
    pub sub_polarity: SubPolarity,
    pub sub_crossover: i32,
    pub phase_correction: bool,
}

impl Default for EqProfile {
    fn default() -> Self {
        Self {
            name: "Standard".to_string(),
            treble: 0.0,
            bass_extension: BassExtension::Standard,
            desk_mode: false,
            desk_db: 0.0,
            wall_mode: false,
            wall_db: 0.0,
            sub_out: false,
            sub_gain: 0.0,
            sub_polarity: SubPolarity::Positive,
            sub_crossover: 80,
            phase_correction: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum BassExtension {
    Less,
    #[default]
    Standard,
    More,
}

impl BassExtension {
    pub fn display_name(self) -> &'static str {
        match self {
            BassExtension::Less => "Less",
            BassExtension::Standard => "Standard",
            BassExtension::More => "More",
        }
    }

    pub fn cycle_next(self) -> Self {
        match self {
            BassExtension::Less => BassExtension::Standard,
            BassExtension::Standard => BassExtension::More,
            BassExtension::More => BassExtension::Less,
        }
    }

    pub fn cycle_prev(self) -> Self {
        match self {
            BassExtension::Less => BassExtension::More,
            BassExtension::Standard => BassExtension::Less,
            BassExtension::More => BassExtension::Standard,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SubPolarity {
    #[default]
    Positive,
    Negative,
}

impl SubPolarity {
    pub fn display_name(self) -> &'static str {
        match self {
            SubPolarity::Positive => "+",
            SubPolarity::Negative => "−",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            SubPolarity::Positive => SubPolarity::Negative,
            SubPolarity::Negative => SubPolarity::Positive,
        }
    }
}

// ---------- Event types ----------

#[derive(Debug, Clone, Deserialize)]
pub struct EventSubscribeResponse {
    #[serde(rename = "queueId")]
    pub queue_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PollEvent {
    pub path: String,
    #[serde(flatten)]
    pub value: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_value_i32_roundtrip() {
        let val = ApiValue::i32(50);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, r#"{"type":"i32_","i32_":50}"#);

        let parsed: ApiValue = serde_json::from_str(&json).unwrap();
        match parsed {
            ApiValue::I32 { value } => assert_eq!(value, 50),
            _ => panic!("expected I32"),
        }
    }

    #[test]
    fn api_value_bool_roundtrip() {
        let val = ApiValue::bool(true);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, r#"{"type":"bool_","bool_":true}"#);

        let parsed: ApiValue = serde_json::from_str(&json).unwrap();
        match parsed {
            ApiValue::Bool { value } => assert!(value),
            _ => panic!("expected Bool"),
        }
    }

    #[test]
    fn api_value_source_roundtrip() {
        let val = ApiValue::source(Source::Usb);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(
            json,
            r#"{"type":"kefPhysicalSource","kefPhysicalSource":"usb"}"#
        );

        let parsed: ApiValue = serde_json::from_str(&json).unwrap();
        match parsed {
            ApiValue::PhysicalSource { value } => assert_eq!(value, Source::Usb),
            _ => panic!("expected PhysicalSource"),
        }
    }

    #[test]
    fn api_value_cable_mode_roundtrip() {
        let val = ApiValue::cable_mode(CableMode::Wired);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(
            json,
            r#"{"type":"kefCableMode","kefCableMode":"wired"}"#
        );

        let parsed: ApiValue = serde_json::from_str(&json).unwrap();
        match parsed {
            ApiValue::CableMode { value } => assert_eq!(value, CableMode::Wired),
            _ => panic!("expected CableMode"),
        }
    }

    #[test]
    fn api_value_speaker_status_roundtrip() {
        let val = ApiValue::speaker_status(SpeakerStatus::PowerOn);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(
            json,
            r#"{"type":"kefSpeakerStatus","kefSpeakerStatus":"powerOn"}"#
        );
    }

    #[test]
    fn set_data_request_serialization() {
        let req = SetDataRequest::new("player:volume", ApiValue::i32(50));
        let json = serde_json::to_string(&req).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["path"], "player:volume");
        assert_eq!(parsed["roles"], "value");
        assert_eq!(parsed["value"]["type"], "i32_");
        assert_eq!(parsed["value"]["i32_"], 50);
    }

    #[test]
    fn get_data_response_parsing() {
        let json = r#"[{"type": "kefCableMode", "kefCableMode": "wired"}]"#;
        let resp: GetDataResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.len(), 1);
        match &resp[0] {
            ApiValue::CableMode { value } => assert_eq!(*value, CableMode::Wired),
            _ => panic!("expected CableMode"),
        }
    }

    #[test]
    fn source_display_names() {
        assert_eq!(Source::Usb.display_name(), "USB");
        assert_eq!(Source::Wifi.display_name(), "Wi-Fi");
        assert_eq!(Source::Bluetooth.display_name(), "Bluetooth");
    }

    #[test]
    fn standby_mode_cycling() {
        let mode = StandbyMode::TwentyMinutes;
        assert_eq!(mode.cycle_next(), StandbyMode::ThirtyMinutes);
        assert_eq!(mode.cycle_next().cycle_next(), StandbyMode::SixtyMinutes);
        assert_eq!(
            mode.cycle_next().cycle_next().cycle_next(),
            StandbyMode::Never
        );
        assert_eq!(
            mode.cycle_next().cycle_next().cycle_next().cycle_next(),
            StandbyMode::TwentyMinutes
        );

        assert_eq!(mode.cycle_prev(), StandbyMode::Never);
    }
}
