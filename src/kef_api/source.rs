//! Source get/set.

use crate::error::KefError;
use super::KefClient;
use super::paths;
use super::types::{ApiValue, Source};

impl KefClient {
    pub async fn get_source(&self) -> Result<Source, KefError> {
        let data = self.get_data(paths::SOURCE).await?;
        match data.into_iter().next() {
            Some(ApiValue::PhysicalSource { value }) => Ok(value),
            _ => Ok(Source::Standby),
        }
    }

    pub async fn set_source(&self, source: Source) -> Result<(), KefError> {
        self.set_data(paths::SOURCE, ApiValue::source(source)).await
    }
}
