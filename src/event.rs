//! Async event multiplexer: terminal input, tick timer, speaker polling, SIGUSR1.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::app::{DiscoveredSpeaker, SpeakerState};
use crate::kef_api::KefClient;
use crate::kef_api::paths as api;

#[derive(Debug)]
pub(crate) enum Event {
    Key(KeyEvent),
    Resize,
    Tick,
    SpeakerUpdate {
        ip: IpAddr,
        state: Box<SpeakerState>,
    },
    SpeakerError {
        ip: IpAddr,
        message: String,
    },
    SpeakerSwitchFinished {
        ip: IpAddr,
        result: Result<Box<SpeakerState>, String>,
    },
    SpeakersDiscovered(Result<Vec<DiscoveredSpeaker>, String>),
    ThemeChanged,
}

pub(crate) struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    tx: mpsc::UnboundedSender<Event>,
    cancel: CancellationToken,
    speaker_cancel: Option<CancellationToken>,
}

impl EventHandler {
    pub(crate) fn new(tick_rate: Duration, client: Option<Arc<KefClient>>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let cancel = CancellationToken::new();

        // Terminal event + tick task
        let tx_term = tx.clone();
        let token = cancel.clone();
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_rate);

            loop {
                tokio::select! {
                    () = token.cancelled() => break,
                    _ = tick_interval.tick() => {
                        if tx_term.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    event = reader.next() => {
                        match event {
                            Some(Ok(CrosstermEvent::Key(key)))
                                if key.kind == KeyEventKind::Press =>
                            {
                                if tx_term.send(Event::Key(key)).is_err() {
                                    break;
                                }
                            }
                            Some(Ok(CrosstermEvent::Resize(_, _))) => {
                                if tx_term.send(Event::Resize).is_err() {
                                    break;
                                }
                            }
                            Some(Ok(_)) => {}
                            Some(Err(_)) | None => break,
                        }
                    }
                }
            }
        });

        // SIGUSR1 theme reload listener
        #[cfg(unix)]
        {
            let tx_signal = tx.clone();
            let token = cancel.clone();
            tokio::spawn(async move {
                use tokio::signal::unix::{SignalKind, signal};
                let Ok(mut stream) = signal(SignalKind::user_defined1()) else {
                    return;
                };
                loop {
                    tokio::select! {
                        () = token.cancelled() => break,
                        _ = stream.recv() => {
                            if tx_signal.send(Event::ThemeChanged).is_err() {
                                break;
                            }
                        }
                    }
                }
            });
        }

        let mut handler = Self {
            rx,
            tx,
            cancel,
            speaker_cancel: None,
        };
        if let Some(client) = client {
            handler.set_speaker(client);
        }
        handler
    }

    pub(crate) async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub(crate) fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self.tx.clone()
    }

    pub(crate) fn set_speaker(&mut self, client: Arc<KefClient>) {
        self.clear_speaker();
        let tx = self.tx.clone();
        let token = self.cancel.child_token();
        self.speaker_cancel = Some(token.clone());
        tokio::spawn(async move {
            speaker_poll_loop(client, tx, token).await;
        });
    }

    fn clear_speaker(&mut self) {
        if let Some(token) = self.speaker_cancel.take() {
            token.cancel();
        }
    }

    pub(crate) fn shutdown(&mut self) {
        self.clear_speaker();
        self.cancel.cancel();
    }
}

async fn speaker_poll_loop(
    client: Arc<KefClient>,
    tx: mpsc::UnboundedSender<Event>,
    cancel: CancellationToken,
) {
    let ip = client.ip();
    // Subscribe to key state changes
    let paths = [
        api::VOLUME,
        api::SOURCE,
        api::SPEAKER_STATUS,
        api::MUTE,
        api::CABLE_MODE,
        api::STANDBY_MODE,
        api::MAX_VOLUME,
        api::EQ_PROFILE,
        api::WAKE_UP_SOURCE,
    ];
    let fallback_paths = &paths[..paths.len() - 1];

    loop {
        if cancel.is_cancelled() {
            return;
        }

        let queue_id = match client.subscribe(&paths).await {
            Ok(id) => id,
            Err(extended_err) => {
                tracing::warn!(
                    "Subscribe with extended paths failed ({extended_err}); retrying with core paths"
                );
                match client.subscribe(fallback_paths).await {
                    Ok(id) => id,
                    Err(e) => {
                        let _ = tx.send(Event::SpeakerError {
                            ip,
                            message: format!("Subscribe failed: {e}"),
                        });
                        tokio::select! {
                            () = cancel.cancelled() => return,
                            () = tokio::time::sleep(Duration::from_secs(5)) => continue,
                        }
                    }
                }
            }
        };

        // Poll loop
        loop {
            if cancel.is_cancelled() {
                let _ = client.unsubscribe(&queue_id).await;
                return;
            }

            match client.poll_events(&queue_id).await {
                Ok(Some(_)) => {
                    // On any event, re-fetch full state for simplicity
                    match client.fetch_full_state().await {
                        Ok(state) => {
                            if tx
                                .send(Event::SpeakerUpdate {
                                    ip,
                                    state: Box::new(state),
                                })
                                .is_err()
                            {
                                return;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Event::SpeakerError {
                                ip,
                                message: format!("State fetch failed: {e}"),
                            });
                        }
                    }
                }
                Ok(None) => {} // Timeout, no events — just re-poll
                Err(e) => {
                    let _ = tx.send(Event::SpeakerError {
                        ip,
                        message: format!("Poll failed: {e}"),
                    });
                    // Break inner loop to re-subscribe
                    break;
                }
            }
        }

        // Unsubscribe (best effort) and retry after delay
        let _ = client.unsubscribe(&queue_id).await;
        tokio::select! {
            () = cancel.cancelled() => return,
            () = tokio::time::sleep(Duration::from_secs(5)) => {},
        }
    }
}
