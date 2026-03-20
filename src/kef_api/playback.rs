//! Playback control: play, pause, next, previous, seek.

use crate::error::KefError;
use super::KefClient;
use super::paths;
use super::types::ApiValue;

impl KefClient {
    pub async fn play(&self) -> Result<(), KefError> {
        self.set_data(paths::PLAYER_CONTROL, ApiValue::string("play"))
            .await
    }

    pub async fn pause(&self) -> Result<(), KefError> {
        self.set_data(paths::PLAYER_CONTROL, ApiValue::string("pause"))
            .await
    }

    pub async fn next_track(&self) -> Result<(), KefError> {
        self.set_data(paths::PLAYER_CONTROL, ApiValue::string("next"))
            .await
    }

    pub async fn previous_track(&self) -> Result<(), KefError> {
        self.set_data(paths::PLAYER_CONTROL, ApiValue::string("previous"))
            .await
    }

    pub async fn seek(&self, position_secs: i64) -> Result<(), KefError> {
        self.set_data(
            paths::PLAYER_CONTROL,
            ApiValue::string(format!("seekTime:{position_secs}")),
        )
        .await
    }
}
