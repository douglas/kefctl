//! API types: `ApiValue` tagged union, `Source`, `EqProfile`, domain enums.

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

// ---------- EQ ----------

#[allow(clippy::struct_excessive_bools)] // mirrors KEF API EQ structure
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
        let val = ApiValue::CableMode { value: CableMode::Wired };
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
        let val = ApiValue::SpeakerStatus { value: SpeakerStatus::PowerOn };
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

    #[test]
    fn api_value_string_roundtrip() {
        let val = ApiValue::string("hello");
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, r#"{"type":"string_","string_":"hello"}"#);

        let parsed: ApiValue = serde_json::from_str(&json).unwrap();
        match parsed {
            ApiValue::String { value } => assert_eq!(value, "hello"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn api_value_standby_mode_roundtrip() {
        let val = ApiValue::StandbyMode {
            value: StandbyMode::SixtyMinutes,
        };
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(
            json,
            r#"{"type":"kefStandbyMode","kefStandbyMode":"standby_60mins"}"#
        );

        let parsed: ApiValue = serde_json::from_str(&json).unwrap();
        match parsed {
            ApiValue::StandbyMode { value } => assert_eq!(value, StandbyMode::SixtyMinutes),
            _ => panic!("expected StandbyMode"),
        }
    }

    #[test]
    fn bass_extension_cycling() {
        let b = BassExtension::Less;
        assert_eq!(b.cycle_next().cycle_next().cycle_next(), BassExtension::Less);
        assert_eq!(b.cycle_prev().cycle_prev().cycle_prev(), BassExtension::Less);
    }

    #[test]
    fn bass_extension_display_names() {
        assert_eq!(BassExtension::Less.display_name(), "Less");
        assert_eq!(BassExtension::Standard.display_name(), "Standard");
        assert_eq!(BassExtension::More.display_name(), "More");
    }

    #[test]
    fn cable_mode_display_names() {
        assert_eq!(CableMode::Wired.display_name(), "Wired");
        assert_eq!(CableMode::Wireless.display_name(), "Wireless");
    }

    #[test]
    fn standby_mode_display_names() {
        assert_eq!(StandbyMode::TwentyMinutes.display_name(), "20 minutes");
        assert_eq!(StandbyMode::ThirtyMinutes.display_name(), "30 minutes");
        assert_eq!(StandbyMode::SixtyMinutes.display_name(), "60 minutes");
        assert_eq!(StandbyMode::Never.display_name(), "Never");
    }

    #[test]
    fn source_all_excludes_standby() {
        assert!(!Source::ALL.contains(&Source::Standby));
        assert!(Source::ALL.contains(&Source::Wifi));
        assert!(Source::ALL.contains(&Source::Usb));
        assert_eq!(Source::ALL.len(), 7);
    }

    #[test]
    fn eq_profile_default() {
        let eq = EqProfile::default();
        assert_eq!(eq.name, "Standard");
        assert_eq!(eq.treble, 0.0);
        assert_eq!(eq.bass_extension, BassExtension::Standard);
        assert!(!eq.desk_mode);
        assert!(!eq.wall_mode);
        assert!(!eq.sub_out);
        assert_eq!(eq.sub_polarity, SubPolarity::Positive);
        assert_eq!(eq.sub_crossover, 80);
        assert!(eq.phase_correction);
    }

    #[test]
    fn unknown_api_value_type_fails() {
        let json = r#"{"type":"unknown_type","unknown_type":"foo"}"#;
        let result = serde_json::from_str::<ApiValue>(json);
        assert!(result.is_err());
    }

    #[test]
    fn eq_profile_serde_roundtrip() {
        let eq = EqProfile {
            name: "Custom".to_string(),
            treble: 2.5,
            bass_extension: BassExtension::More,
            desk_mode: true,
            desk_db: -3.0,
            wall_mode: false,
            wall_db: 0.0,
            sub_out: true,
            sub_gain: 1.5,
            sub_polarity: SubPolarity::Negative,
            sub_crossover: 120,
            phase_correction: false,
        };
        let json = serde_json::to_string(&eq).unwrap();
        let parsed: EqProfile = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "Custom");
        assert_eq!(parsed.treble, 2.5);
        assert_eq!(parsed.bass_extension, BassExtension::More);
        assert!(parsed.desk_mode);
        assert_eq!(parsed.desk_db, -3.0);
        assert!(!parsed.wall_mode);
        assert!(parsed.sub_out);
        assert_eq!(parsed.sub_gain, 1.5);
        assert_eq!(parsed.sub_polarity, SubPolarity::Negative);
        assert_eq!(parsed.sub_crossover, 120);
        assert!(!parsed.phase_correction);
    }

    #[test]
    fn api_value_i64_roundtrip() {
        let val = ApiValue::I64 { value: 123_456_789 };
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, r#"{"type":"i64_","i64_":123456789}"#);

        let parsed: ApiValue = serde_json::from_str(&json).unwrap();
        match parsed {
            ApiValue::I64 { value } => assert_eq!(value, 123_456_789),
            _ => panic!("expected I64"),
        }
    }

    #[test]
    fn api_value_missing_inner_field() {
        // type present but value field missing
        let json = r#"{"type":"i32_"}"#;
        let result = serde_json::from_str::<ApiValue>(json);
        assert!(result.is_err());
    }

    #[test]
    fn source_all_display_names_exhaustive() {
        let names: Vec<&str> = Source::ALL.iter().map(|s| s.display_name()).collect();
        // All unique
        let mut deduped = names.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(names.len(), deduped.len(), "display names should be unique");
        assert_eq!(names.len(), 7, "should have 7 selectable sources");
    }
}
