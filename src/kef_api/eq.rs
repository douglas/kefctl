use crate::error::KefError;
use super::KefClient;

impl KefClient {
    pub async fn get_eq_raw(&self) -> Result<serde_json::Value, KefError> {
        let data = self.get_data("kef:eqProfile/v2").await?;
        // EQ data comes as a complex nested structure; return raw JSON
        // for now and parse into EqProfile during TUI integration
        Ok(serde_json::to_value(&data).unwrap_or_default())
    }
}
