use std::time::Duration;

use reqwest::Client;

use crate::error::KefError;
use super::KefClient;

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

        // API returns a plain JSON string like "{uuid}" — strip braces
        let queue_id: String = resp.json().await?;
        Ok(queue_id.trim_matches(|c| c == '{' || c == '}').to_string())
    }

    /// Long-polls the speaker for events. Returns Ok(Some(value)) on events,
    /// Ok(None) on timeout (no events), or Err on real failures.
    pub async fn poll_events(
        &self,
        queue_id: &str,
    ) -> Result<Option<serde_json::Value>, KefError> {
        let url = format!("{}/api/event/pollQueue", self.base_url);
        // Speaker holds connection for up to 30s; give reqwest 60s
        let poll_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("failed to create poll client");

        let result = poll_client
            .get(&url)
            .query(&[
                ("queueId", queue_id),
                ("timeout", "30000"),
            ])
            .send()
            .await;

        let resp = match result {
            Ok(resp) => resp,
            Err(e) if e.is_timeout() => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        if !resp.status().is_success() {
            return Err(KefError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        let body = resp.text().await.unwrap_or_default();
        if body.is_empty() || body == "[]" {
            return Ok(None);
        }

        match serde_json::from_str(&body) {
            Ok(val) => Ok(Some(val)),
            Err(_) => Ok(None),
        }
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
