use std::time::Duration;

use reqwest::Client;

use crate::error::KefError;
use super::KefClient;
use super::types::EventSubscribeResponse;

impl KefClient {
    pub async fn subscribe(&self, paths: &[&str]) -> Result<String, KefError> {
        let url = format!("{}/api/event/modifyQueue", self.base_url);
        let subscribe_param = serde_json::to_string(paths).unwrap_or_default();
        let resp = self
            .client
            .get(&url)
            .query(&[("subscribe", &subscribe_param), ("queueId", &String::new())])
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(KefError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        let sub: EventSubscribeResponse = resp.json().await?;
        Ok(sub.queue_id)
    }

    pub async fn poll_events(
        &self,
        queue_id: &str,
        timeout_ms: u64,
    ) -> Result<serde_json::Value, KefError> {
        let url = format!("{}/api/event/pollQueue", self.base_url);
        // Poll uses a longer timeout since the server holds the connection
        let poll_client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms + 5000))
            .build()
            .expect("failed to create poll client");

        let resp = poll_client
            .get(&url)
            .query(&[
                ("queueId", queue_id),
                ("timeout", &timeout_ms.to_string()),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(KefError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.json().await?)
    }

    pub async fn unsubscribe(&self, queue_id: &str) -> Result<(), KefError> {
        let url = format!("{}/api/event/modifyQueue", self.base_url);
        let _ = self
            .client
            .get(&url)
            .query(&[("unsubscribe", "true"), ("queueId", queue_id)])
            .send()
            .await;
        Ok(())
    }
}
