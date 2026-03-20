use crate::error::KefError;
use super::KefClient;
use super::types::{ApiValue, CableMode};

impl KefClient {
    pub async fn get_cable_mode(&self) -> Result<CableMode, KefError> {
        let data = self.get_data("settings:/kef/host/cableMode").await?;
        match data.into_iter().next() {
            Some(ApiValue::CableMode { value }) => Ok(value),
            _ => Ok(CableMode::default()),
        }
    }

    pub async fn get_standby_mode(&self) -> Result<i32, KefError> {
        self.get_i32("settings:/kef/host/standbyMode").await
    }

    pub async fn set_standby_mode(&self, minutes: i32) -> Result<(), KefError> {
        self.set_data(
            "settings:/kef/host/standbyMode",
            ApiValue::i32(minutes),
        )
        .await
    }

    pub async fn get_front_led_disabled(&self) -> Result<bool, KefError> {
        self.get_bool("settings:/kef/host/disableFrontStandbyLED").await
    }

    pub async fn set_front_led_disabled(&self, disabled: bool) -> Result<(), KefError> {
        self.set_data(
            "settings:/kef/host/disableFrontStandbyLED",
            ApiValue::bool(disabled),
        )
        .await
    }

    pub async fn get_startup_tone(&self) -> Result<bool, KefError> {
        self.get_bool("settings:/kef/host/startupTone").await
    }

    pub async fn set_startup_tone(&self, enabled: bool) -> Result<(), KefError> {
        self.set_data(
            "settings:/kef/host/startupTone",
            ApiValue::bool(enabled),
        )
        .await
    }
}
