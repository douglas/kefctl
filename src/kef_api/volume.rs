use crate::error::KefError;
use super::KefClient;
use super::paths;
use super::types::ApiValue;

impl KefClient {
    pub async fn get_volume(&self) -> Result<i32, KefError> {
        self.get_i32(paths::VOLUME).await
    }

    pub async fn set_volume(&self, volume: i32) -> Result<(), KefError> {
        self.set_data(paths::VOLUME, ApiValue::i32(volume)).await
    }

    pub async fn get_mute(&self) -> Result<bool, KefError> {
        self.get_bool(paths::MUTE).await
    }

    pub async fn set_mute(&self, muted: bool) -> Result<(), KefError> {
        self.set_data(paths::MUTE, ApiValue::bool(muted)).await
    }

    pub async fn get_max_volume(&self) -> Result<i32, KefError> {
        self.get_i32(paths::MAX_VOLUME).await
    }
}
