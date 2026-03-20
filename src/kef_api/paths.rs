//! API path string constants for KEF speaker endpoints.

pub const DEVICE_NAME: &str = "settings:/deviceName";
pub const FIRMWARE: &str = "settings:/releasetext";
pub const MAC_ADDRESS: &str = "settings:/system/primaryMacAddress";
pub const VOLUME: &str = "player:volume";
pub const SOURCE: &str = "settings:/kef/play/physicalSource";
pub const MUTE: &str = "settings:/mediaPlayer/mute";
pub const CABLE_MODE: &str = "settings:/kef/host/cableMode";
pub const STANDBY_MODE: &str = "settings:/kef/host/standbyMode";
pub const MAX_VOLUME: &str = "settings:/kef/host/maximumVolume";
pub const FRONT_LED: &str = "settings:/kef/host/disableFrontStandbyLED";
pub const STARTUP_TONE: &str = "settings:/kef/host/startupTone";
pub const PLAYER_DATA: &str = "player:player/data";
pub const PLAYER_CONTROL: &str = "player:player/control";
pub const SPEAKER_STATUS: &str = "settings:/kef/host/speakerStatus";
