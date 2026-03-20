//! KEF HTTP API client — `KefClient`, `get_data`/`set_data`, typed extraction.

pub(crate) mod events;
pub(crate) mod paths;
pub(crate) mod playback;
pub(crate) mod settings;
pub(crate) mod source;
pub(crate) mod types;
pub(crate) mod volume;

use std::net::IpAddr;
use std::time::Duration;

use reqwest::Client;

use crate::app::SpeakerState;
use crate::error::KefError;
use types::{ApiValue, GetDataResponse, SetDataRequest};

pub(crate) struct KefClient {
    base_url: String,
    ip: IpAddr,
    client: Client,
    poll_client: Client,
}

impl KefClient {
    pub fn new(ip: IpAddr) -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(5))
            .build()
            .expect("failed to create HTTP client");

        let poll_client = Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(60))
            .build()
            .expect("failed to create poll HTTP client");

        Self {
            base_url: format!("http://{ip}"),
            ip,
            client,
            poll_client,
        }
    }

    #[tracing::instrument(skip(self), fields(path))]
    pub async fn get_data(&self, path: &str) -> Result<GetDataResponse, KefError> {
        let url = format!("{}/api/getData", self.base_url);
        let resp = self
            .client
            .get(&url)
            .query(&[("path", path), ("roles", "value")])
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

    #[tracing::instrument(skip(self, value), fields(path))]
    pub async fn set_data(&self, path: &str, value: ApiValue) -> Result<(), KefError> {
        let url = format!("{}/api/setData", self.base_url);
        let req = SetDataRequest::new(path, value);
        let resp = self.client.post(&url).json(&req).send().await?;

        if !resp.status().is_success() {
            return Err(KefError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    pub async fn get_string(&self, path: &str) -> Result<String, KefError> {
        let data = self.get_data(path).await?;
        extract_string(data)
    }

    pub async fn get_i32(&self, path: &str) -> Result<i32, KefError> {
        let data = self.get_data(path).await?;
        extract_i32(data)
    }

    pub async fn get_bool(&self, path: &str) -> Result<bool, KefError> {
        let data = self.get_data(path).await?;
        extract_bool(data)
    }

    #[tracing::instrument(skip(self))]
    pub async fn fetch_full_state(&self) -> Result<SpeakerState, KefError> {
        let (name, firmware, mac, source, volume, muted, cable_mode, max_volume) = tokio::try_join!(
            self.get_string(paths::DEVICE_NAME),
            self.get_string(paths::FIRMWARE),
            self.get_string(paths::MAC_ADDRESS),
            self.get_source(),
            self.get_volume(),
            self.get_mute(),
            self.get_cable_mode(),
            self.get_max_volume(),
        )?;

        let (standby_mode, front_led_disabled, startup_tone) = tokio::try_join!(
            self.get_standby_mode(),
            self.get_front_led_disabled(),
            self.get_startup_tone(),
        )?;

        Ok(SpeakerState {
            name,
            model: "LSX II".to_string(),
            firmware,
            ip: self.ip,
            mac,
            source,
            volume,
            muted,
            cable_mode,
            standby_mode,
            max_volume,
            front_led: !front_led_disabled,
            startup_tone,
            eq_profile: types::EqProfile::default(),
            artist: None,
            track: None,
            duration: None,
            position: None,
            playing: false,
        })
    }
}

// Pure extraction functions — testable without HTTP
fn extract_string(data: GetDataResponse) -> Result<String, KefError> {
    match data.into_iter().next() {
        Some(ApiValue::String { value }) => Ok(value),
        Some(other) => Err(KefError::TypeMismatch {
            expected: "string",
            got: format!("{other:?}"),
        }),
        None => Err(KefError::TypeMismatch {
            expected: "string",
            got: "empty response".to_string(),
        }),
    }
}

fn extract_i32(data: GetDataResponse) -> Result<i32, KefError> {
    match data.into_iter().next() {
        Some(ApiValue::I32 { value }) => Ok(value),
        Some(other) => Err(KefError::TypeMismatch {
            expected: "i32",
            got: format!("{other:?}"),
        }),
        None => Err(KefError::TypeMismatch {
            expected: "i32",
            got: "empty response".to_string(),
        }),
    }
}

fn extract_bool(data: GetDataResponse) -> Result<bool, KefError> {
    match data.into_iter().next() {
        Some(ApiValue::Bool { value }) => Ok(value),
        Some(other) => Err(KefError::TypeMismatch {
            expected: "bool",
            got: format!("{other:?}"),
        }),
        None => Err(KefError::TypeMismatch {
            expected: "bool",
            got: "empty response".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::Source;

    #[test]
    fn get_string_returns_string() {
        let data = vec![ApiValue::string("hello")];
        assert_eq!(extract_string(data).unwrap(), "hello");
    }

    #[test]
    fn get_string_type_mismatch() {
        let data = vec![ApiValue::i32(42)];
        let err = extract_string(data).unwrap_err();
        assert!(matches!(err, KefError::TypeMismatch { expected: "string", .. }));
    }

    #[test]
    fn get_string_empty_response() {
        let data: GetDataResponse = vec![];
        let err = extract_string(data).unwrap_err();
        assert!(matches!(err, KefError::TypeMismatch { expected: "string", .. }));
    }

    #[test]
    fn get_i32_returns_i32() {
        let data = vec![ApiValue::i32(50)];
        assert_eq!(extract_i32(data).unwrap(), 50);
    }

    #[test]
    fn get_i32_type_mismatch() {
        let data = vec![ApiValue::string("nope")];
        let err = extract_i32(data).unwrap_err();
        assert!(matches!(err, KefError::TypeMismatch { expected: "i32", .. }));
    }

    #[test]
    fn get_bool_returns_bool() {
        let data = vec![ApiValue::bool(true)];
        assert!(extract_bool(data).unwrap());
    }

    #[test]
    fn get_bool_type_mismatch() {
        let data = vec![ApiValue::string("nope")];
        let err = extract_bool(data).unwrap_err();
        assert!(matches!(err, KefError::TypeMismatch { expected: "bool", .. }));
    }

    #[test]
    fn get_bool_empty_response() {
        let data: GetDataResponse = vec![];
        let err = extract_bool(data).unwrap_err();
        assert!(matches!(err, KefError::TypeMismatch { expected: "bool", .. }));
    }

    #[test]
    fn extract_ignores_extra_elements() {
        // API returns array — we only use the first element
        let data = vec![ApiValue::i32(10), ApiValue::i32(20)];
        assert_eq!(extract_i32(data).unwrap(), 10);
    }

    #[test]
    fn extract_source_from_physical_source() {
        // Source extraction is in source.rs but test the pattern here
        let data: GetDataResponse = vec![ApiValue::source(Source::Bluetooth)];
        match data.into_iter().next() {
            Some(ApiValue::PhysicalSource { value }) => {
                assert_eq!(value, Source::Bluetooth);
            }
            _ => panic!("expected PhysicalSource"),
        }
    }
}
