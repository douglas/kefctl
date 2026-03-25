//! Device settings: cable mode, standby, LED, startup tone, EQ profile.

use crate::error::KefError;
use super::KefClient;
use super::paths;
use super::types::{ApiValue, CableMode, EqProfile, StandbyMode};

impl KefClient {
    pub async fn get_cable_mode(&self) -> Result<CableMode, KefError> {
        let data = self.get_data(paths::CABLE_MODE).await?;
        match data.into_iter().next() {
            Some(ApiValue::CableMode { value }) => Ok(value),
            _ => Ok(CableMode::default()),
        }
    }

    pub async fn set_cable_mode(&self, mode: CableMode) -> Result<(), KefError> {
        self.set_data(paths::CABLE_MODE, ApiValue::CableMode { value: mode })
            .await
    }

    pub async fn get_standby_mode(&self) -> Result<StandbyMode, KefError> {
        let data = self.get_data(paths::STANDBY_MODE).await?;
        match data.into_iter().next() {
            Some(ApiValue::StandbyMode { value }) => Ok(value),
            _ => Ok(StandbyMode::default()),
        }
    }

    pub async fn set_standby_mode(&self, mode: StandbyMode) -> Result<(), KefError> {
        self.set_data(paths::STANDBY_MODE, ApiValue::StandbyMode { value: mode })
            .await
    }

    pub async fn get_front_led_disabled(&self) -> Result<bool, KefError> {
        self.get_bool(paths::FRONT_LED).await
    }

    pub async fn set_front_led_disabled(&self, disabled: bool) -> Result<(), KefError> {
        self.set_data(paths::FRONT_LED, ApiValue::bool(disabled))
            .await
    }

    pub async fn get_startup_tone(&self) -> Result<bool, KefError> {
        self.get_bool(paths::STARTUP_TONE).await
    }

    pub async fn set_startup_tone(&self, enabled: bool) -> Result<(), KefError> {
        self.set_data(paths::STARTUP_TONE, ApiValue::bool(enabled))
            .await
    }

    pub async fn set_eq_profile(&self, profile: EqProfile) -> Result<(), KefError> {
        self.set_data(paths::EQ_PROFILE, ApiValue::EqProfile { value: profile })
            .await
    }
}
