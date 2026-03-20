use crate::error::KefError;
use super::KefClient;
use super::types::{ApiValue, Source};

impl KefClient {
    pub async fn get_source(&self) -> Result<Source, KefError> {
        let data = self.get_data("settings:/kef/play/physicalSource").await?;
        match data.into_iter().next() {
            Some(ApiValue::PhysicalSource { value }) => Ok(value),
            _ => Ok(Source::Standby),
        }
    }

    pub async fn set_source(&self, source: Source) -> Result<(), KefError> {
        self.set_data(
            "settings:/kef/play/physicalSource",
            ApiValue::source(source),
        )
        .await
    }
}
