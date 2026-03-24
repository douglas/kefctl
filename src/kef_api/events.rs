//! Event subscribe/poll/unsubscribe (long-poll).

use crate::error::KefError;
use super::KefClient;

impl KefClient {
    #[tracing::instrument(skip(self))]
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
                message: super::sanitize(resp.text().await.unwrap_or_default()),
            });
        }

        // API returns a plain JSON string like "{uuid}" — strip braces
        let bytes = resp.bytes().await?;
        if bytes.len() > super::MAX_RESPONSE_BYTES {
            return Err(KefError::Api {
                status: 0,
                message: "response body too large".into(),
            });
        }
        let queue_id: String = serde_json::from_slice(&bytes)?;
        Ok(queue_id.trim_matches(|c| c == '{' || c == '}').to_string())
    }

    /// Long-polls the speaker for events. Returns Ok(Some(value)) on events,
    /// Ok(None) on timeout (no events), or Err on real failures.
    #[tracing::instrument(skip(self))]
    pub async fn poll_events(
        &self,
        queue_id: &str,
    ) -> Result<Option<serde_json::Value>, KefError> {
        let url = format!("{}/api/event/pollQueue", self.base_url);
        let result = self.poll_client
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
                message: super::sanitize(resp.text().await.unwrap_or_default()),
            });
        }

        let body = resp.text().await.unwrap_or_default();
        if body.is_empty() || body == "[]" {
            return Ok(None);
        }
        if body.len() > super::MAX_RESPONSE_BYTES {
            tracing::warn!("poll response too large ({} bytes), ignoring", body.len());
            return Ok(None);
        }

        match serde_json::from_str(&body) {
            Ok(val) => Ok(Some(val)),
            Err(e) => {
                tracing::debug!("Failed to parse poll response: {e}");
                Ok(None)
            }
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
