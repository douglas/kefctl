use std::net::IpAddr;
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceEvent};

use crate::app::DiscoveredSpeaker;
use crate::error::KefError;

pub async fn discover_speakers(
    timeout: Duration,
) -> Result<Vec<DiscoveredSpeaker>, KefError> {
    let mdns = ServiceDaemon::new().map_err(|e| KefError::Discovery(e.to_string()))?;
    let receiver = mdns
        .browse("_kef-info._tcp.local.")
        .map_err(|e| KefError::Discovery(e.to_string()))?;

    let mut speakers = Vec::new();
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }

        let recv = receiver.clone();
        match tokio::time::timeout(
            remaining,
            tokio::task::spawn_blocking(move || recv.recv_timeout(Duration::from_millis(500))),
        )
        .await
        {
            Ok(Ok(Ok(ServiceEvent::ServiceResolved(info)))) => {
                let name = info.get_fullname().to_string();
                for addr in info.get_addresses_v4() {
                    speakers.push(DiscoveredSpeaker {
                        name: info
                            .get_property_val_str("fn")
                            .unwrap_or(&name)
                            .to_string(),
                        ip: IpAddr::V4(addr),
                        port: info.get_port(),
                    });
                }
            }
            Ok(Ok(Ok(_) | Err(_))) => {} // other events or recv timeout
            Ok(Err(_)) | Err(_) => break, // spawn_blocking or overall timeout
        }
    }

    let _ = mdns.shutdown();
    Ok(speakers)
}
