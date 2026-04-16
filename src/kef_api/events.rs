//! Event subscribe/poll/unsubscribe (long-poll).

use super::KefClient;
use crate::error::KefError;
use serde::Deserialize;

impl KefClient {
    #[tracing::instrument(skip(self))]
    pub async fn subscribe(&self, paths: &[&str]) -> Result<String, KefError> {
        let url = format!("{}/api/event/modifyQueue", self.base_url);
        let subscribe_param = serde_json::to_string(paths).unwrap_or_default();
        let resp = send_event_request(
            &self.client,
            &url,
            &[("subscribe", subscribe_param.as_str()), ("queueId", "")],
        )
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
        Ok(parse_subscribe_queue_id(&bytes)?)
    }

    /// Long-polls the speaker for events. Returns Ok(Some(value)) on events,
    /// Ok(None) on timeout (no events), or Err on real failures.
    #[tracing::instrument(skip(self))]
    pub async fn poll_events(&self, queue_id: &str) -> Result<Option<serde_json::Value>, KefError> {
        let url = format!("{}/api/event/pollQueue", self.base_url);
        let result = send_event_request(
            &self.poll_client,
            &url,
            &[("queueId", queue_id), ("timeout", "30000")],
        )
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
        let _ = send_event_request(
            &self.client,
            &url,
            &[("unsubscribe", "true"), ("queueId", queue_id)],
        )
        .await;
        Ok(())
    }
}

async fn send_event_request(
    client: &reqwest::Client,
    url: &str,
    query: &[(&str, &str)],
) -> Result<reqwest::Response, reqwest::Error> {
    let resp = client.get(url).query(query).send().await?;
    if resp.status() == reqwest::StatusCode::NOT_IMPLEMENTED {
        tracing::debug!("event endpoint rejected GET with 501, retrying as POST json body");
        let json_body = event_query_as_json(query);
        let json_resp = client.post(url).json(&json_body).send().await?;
        if json_resp.status() != reqwest::StatusCode::BAD_REQUEST {
            return Ok(json_resp);
        }

        tracing::debug!("event endpoint POST json returned 400, retrying as POST query");
        return client.post(url).query(query).send().await;
    }
    Ok(resp)
}

fn event_query_as_json(query: &[(&str, &str)]) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    for (key, value) in query {
        let parsed = match *key {
            "subscribe" => serde_json::from_str::<serde_json::Value>(value)
                .unwrap_or_else(|_| serde_json::Value::String((*value).to_string())),
            "timeout" => value.parse::<u64>().map_or_else(
                |_| serde_json::Value::String((*value).to_string()),
                |v| serde_json::Value::Number(v.into()),
            ),
            "unsubscribe" => match *value {
                "true" => serde_json::Value::Bool(true),
                "false" => serde_json::Value::Bool(false),
                _ => serde_json::Value::String((*value).to_string()),
            },
            _ => serde_json::Value::String((*value).to_string()),
        };
        obj.insert((*key).to_string(), parsed);
    }
    serde_json::Value::Object(obj)
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SubscribeQueueResponse {
    Plain(String),
    Object {
        #[serde(rename = "queueId", alias = "queue_id", alias = "queueID")]
        queue_id: String,
    },
}

fn parse_subscribe_queue_id(bytes: &[u8]) -> Result<String, serde_json::Error> {
    let parsed: SubscribeQueueResponse = serde_json::from_slice(bytes)?;
    let queue_id = match parsed {
        SubscribeQueueResponse::Plain(id) => id,
        SubscribeQueueResponse::Object { queue_id } => queue_id,
    };
    // Some firmware wraps queue IDs in braces (`{uuid}`), others return plain UUIDs.
    Ok(queue_id.trim_matches(|c| c == '{' || c == '}').to_string())
}

#[cfg(test)]
mod tests {
    use super::{event_query_as_json, parse_subscribe_queue_id};

    #[test]
    fn parse_subscribe_queue_id_plain_string() {
        let queue_id = parse_subscribe_queue_id(br#""{abc-123}""#).unwrap();
        assert_eq!(queue_id, "abc-123");
    }

    #[test]
    fn parse_subscribe_queue_id_object_shape() {
        let queue_id = parse_subscribe_queue_id(br#"{"queueId":"{abc-123}"}"#).unwrap();
        assert_eq!(queue_id, "abc-123");
    }

    #[test]
    fn parse_subscribe_queue_id_plain_uuid() {
        let queue_id = parse_subscribe_queue_id(br#""abc-123""#).unwrap();
        assert_eq!(queue_id, "abc-123");
    }

    #[test]
    fn event_query_as_json_parses_special_fields() {
        let json = event_query_as_json(&[
            ("subscribe", r#"["player:volume"]"#),
            ("timeout", "30000"),
            ("unsubscribe", "true"),
            ("queueId", ""),
        ]);
        assert_eq!(json["subscribe"][0], "player:volume");
        assert_eq!(json["timeout"], 30000);
        assert_eq!(json["unsubscribe"], true);
        assert_eq!(json["queueId"], "");
    }
}
