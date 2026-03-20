use crate::error::KefError;
use super::KefClient;
use super::types::ApiValue;

impl KefClient {
    pub async fn play(&self) -> Result<(), KefError> {
        self.set_data("player:player/control", ApiValue::string("play"))
            .await
    }

    pub async fn pause(&self) -> Result<(), KefError> {
        self.set_data("player:player/control", ApiValue::string("pause"))
            .await
    }

    pub async fn next_track(&self) -> Result<(), KefError> {
        self.set_data("player:player/control", ApiValue::string("next"))
            .await
    }

    pub async fn previous_track(&self) -> Result<(), KefError> {
        self.set_data("player:player/control", ApiValue::string("previous"))
            .await
    }

    pub async fn seek(&self, position_secs: i64) -> Result<(), KefError> {
        self.set_data(
            "player:player/control",
            ApiValue::string(format!("seekTime:{position_secs}")),
        )
        .await
    }
}
