pub mod eq;
pub mod events;
pub mod playback;
pub mod settings;
pub mod source;
pub mod types;
pub mod volume;

use std::net::IpAddr;
use std::time::Duration;

use reqwest::Client;

use crate::app::SpeakerState;
use crate::error::KefError;
use types::{ApiValue, GetDataResponse, SetDataRequest};

pub struct KefClient {
    base_url: String,
    ip: IpAddr,
    client: Client,
}

impl KefClient {
    pub fn new(ip: IpAddr) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("failed to create HTTP client");

        Self {
            base_url: format!("http://{ip}"),
            ip,
            client,
        }
    }

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
        match data.into_iter().next() {
            Some(ApiValue::String { value }) => Ok(value),
            _ => Ok(String::new()),
        }
    }

    pub async fn get_i32(&self, path: &str) -> Result<i32, KefError> {
        let data = self.get_data(path).await?;
        match data.into_iter().next() {
            Some(ApiValue::I32 { value }) => Ok(value),
            _ => Ok(0),
        }
    }

    pub async fn get_bool(&self, path: &str) -> Result<bool, KefError> {
        let data = self.get_data(path).await?;
        match data.into_iter().next() {
            Some(ApiValue::Bool { value }) => Ok(value),
            _ => Ok(false),
        }
    }

    pub async fn fetch_full_state(&self) -> Result<SpeakerState, KefError> {
        let (name, firmware, mac, source, volume, muted, cable_mode, max_volume) = tokio::try_join!(
            self.get_string("settings:/deviceName"),
            self.get_string("settings:/releasetext"),
            self.get_string("settings:/system/primaryMacAddress"),
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
