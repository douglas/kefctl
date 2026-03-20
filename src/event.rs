use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent};
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::app::SpeakerState;
use crate::kef_api::KefClient;
use crate::kef_api::paths as api;

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Resize,
    Tick,
    SpeakerUpdate(Box<SpeakerState>),
    SpeakerError(String),
    ThemeChanged,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration, client: Option<Arc<KefClient>>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        // Terminal event + tick task
        let tx_term = tx.clone();
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_rate);

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        if tx_term.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    event = reader.next() => {
                        match event {
                            Some(Ok(CrosstermEvent::Key(key))) => {
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

        // Speaker poll task (only if we have a client)
        if let Some(client) = client {
            let tx_speaker = tx.clone();
            tokio::spawn(async move {
                speaker_poll_loop(client, tx_speaker).await;
            });
        }

        // SIGUSR1 theme reload listener
        #[cfg(unix)]
        {
            let tx_signal = tx;
            tokio::spawn(async move {
                use tokio::signal::unix::{SignalKind, signal};
                let Ok(mut stream) = signal(SignalKind::user_defined1()) else {
                    return;
                };
                loop {
                    stream.recv().await;
                    if tx_signal.send(Event::ThemeChanged).is_err() {
                        break;
                    }
                }
            });
        }

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}

async fn speaker_poll_loop(client: Arc<KefClient>, tx: mpsc::UnboundedSender<Event>) {
    // Subscribe to key state changes
    let paths = &[
        api::VOLUME,
        api::PLAYER_DATA,
        api::SOURCE,
        api::SPEAKER_STATUS,
        api::MUTE,
        api::CABLE_MODE,
    ];

    loop {
        let queue_id = match client.subscribe(paths).await {
            Ok(id) => id,
            Err(e) => {
                let _ = tx.send(Event::SpeakerError(format!("Subscribe failed: {e}")));
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        // Poll loop
        loop {
            match client.poll_events(&queue_id).await {
                Ok(Some(_)) => {
                    // On any event, re-fetch full state for simplicity
                    match client.fetch_full_state().await {
                        Ok(state) => {
                            if tx.send(Event::SpeakerUpdate(Box::new(state))).is_err() {
                                return;
                            }
                        }
                        Err(e) => {
                            let _ =
                                tx.send(Event::SpeakerError(format!("State fetch failed: {e}")));
                        }
                    }
                }
                Ok(None) => {} // Timeout, no events — just re-poll
                Err(e) => {
                    let _ = tx.send(Event::SpeakerError(format!("Poll failed: {e}")));
                    // Break inner loop to re-subscribe
                    break;
                }
            }
        }

        // Unsubscribe (best effort) and retry after delay
        let _ = client.unsubscribe(&queue_id).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
