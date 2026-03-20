//! kefctl — TUI controller for KEF W2-platform speakers.
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

mod app;
mod cli;
mod config;
mod discovery;
mod error;
mod event;
mod kef_api;
mod tui;
mod ui;

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;

use app::{Action, App};
use cli::{Cli, Commands, SourceArg};
use config::Config;
use event::{Event, EventHandler};
use kef_api::KefClient;
use kef_api::types::Source;

#[tokio::main]
async fn main() {
    init_logging();
    let cli = Cli::parse();
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {e}");
            std::process::exit(1);
        }
    };

    let speaker_ip = cli.speaker.as_deref().or(config.speaker.ip.as_deref());

    match cli.command {
        Some(Commands::Discover) => cmd_discover().await,
        Some(Commands::Status) => {
            let ip = resolve_speaker(speaker_ip);
            cmd_status(ip).await;
        }
        Some(Commands::Source { source: Some(s) }) => {
            let ip = resolve_speaker(speaker_ip);
            cmd_set_source(ip, s).await;
        }
        Some(Commands::Source { source: None }) => {
            let ip = resolve_speaker(speaker_ip);
            cmd_get_source(ip).await;
        }
        Some(Commands::Volume { level: Some(v) }) => {
            let ip = resolve_speaker(speaker_ip);
            cmd_set_volume(ip, v).await;
        }
        Some(Commands::Volume { level: None }) => {
            let ip = resolve_speaker(speaker_ip);
            cmd_get_volume(ip).await;
        }
        None => {
            if cli.demo {
                run_tui_demo(config).await;
            } else {
                let ip_str = speaker_ip.map(String::from);
                run_tui_live(ip_str, config).await;
            }
        }
    }
}

async fn run_tui_demo(config: Config) {
    let app = App::new_demo();
    let tick_rate = Duration::from_millis(config.ui.refresh_ms);
    run_tui_loop(app, None, tick_rate).await;
}

async fn run_tui_live(ip_str: Option<String>, config: Config) {
    // Resolve speaker IP: flag/config > cached IP > mDNS discovery
    let ip: IpAddr = if let Some(ref s) = ip_str {
        s.parse().unwrap_or_else(|_| {
            eprintln!("Invalid IP address: {s}");
            std::process::exit(1);
        })
    } else if let Some(cached) = try_cached_ip().await {
        cached
    } else {
        eprintln!("Discovering speakers...");
        match discovery::discover_speakers(Duration::from_secs(5)).await {
            Ok(speakers) if speakers.len() == 1 => speakers[0].ip,
            Ok(speakers) if speakers.is_empty() => {
                eprintln!(
                    "No KEF speakers found. Use --speaker <ip> or set ip in config."
                );
                std::process::exit(1);
            }
            Ok(speakers) => {
                eprintln!("Multiple speakers found:");
                for s in &speakers {
                    eprintln!("  {} — {}", s.name, s.ip);
                }
                eprintln!("Use --speaker <ip> to select one.");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Discovery error: {e}");
                std::process::exit(1);
            }
        }
    };

    let client = Arc::new(KefClient::new(ip));

    // Fetch initial state
    eprintln!("Connecting to {ip}...");
    let state = match client.fetch_full_state().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect: {e}");
            std::process::exit(1);
        }
    };

    // Cache the working IP for next launch
    config::save_cached_ip(&ip);

    let app = App::new_live(state);
    let tick_rate = Duration::from_millis(config.ui.refresh_ms);
    run_tui_loop(app, Some(client), tick_rate).await;
}

async fn run_tui_loop(mut app: App, client: Option<Arc<KefClient>>, tick_rate: Duration) {
    let mut terminal = match tui::init() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to init terminal: {e}");
            return;
        }
    };

    // Install panic hook that restores terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = tui::restore();
        original_hook(panic_info);
    }));

    let mut events = EventHandler::new(tick_rate, client.clone());
    let event_tx = events.sender();

    loop {
        if let Err(e) = terminal.draw(|frame| ui::draw(frame, &mut app)) {
            events.shutdown();
            let _ = tui::restore();
            eprintln!("Draw failed: {e}");
            return;
        }

        match events.next().await {
            Some(Event::Key(key)) => {
                let action = app.handle_key(key);
                if app.should_quit {
                    break;
                }
                if let (Some(action), Some(client)) = (action, client.as_ref()) {
                    dispatch_action(client.clone(), action, event_tx.clone());
                }
            }
            Some(Event::Tick) => {
                app.tick();
            }
            Some(Event::ThemeChanged) => {
                app.theme = ui::theme::Theme::load();
            }
            Some(Event::Resize) => {}
            Some(Event::SpeakerUpdate(state)) => {
                app.speaker = *state;
                app.connection = app::ConnectionState::Connected;
            }
            Some(Event::SpeakerError(msg)) => {
                app.set_notification(msg);
                app.connection = app::ConnectionState::Disconnected;
            }
            None => break,
        }
    }

    events.shutdown();
    if let Err(e) = tui::restore() {
        eprintln!("Failed to restore terminal: {e}");
    }
}

/// Try the cached speaker IP — quick probe to see if the speaker is still there.
async fn try_cached_ip() -> Option<IpAddr> {
    let cached = config::load_cached_ip()?;
    let ip: IpAddr = cached.parse().ok()?;
    eprintln!("Trying cached speaker {ip}...");
    let client = KefClient::new(ip);
    if client.fetch_full_state().await.is_ok() {
        Some(ip)
    } else {
        tracing::debug!("Cached speaker {ip} unreachable, falling back to discovery");
        None
    }
}

fn dispatch_action(
    client: Arc<KefClient>,
    action: Action,
    tx: tokio::sync::mpsc::UnboundedSender<Event>,
) {
    tokio::spawn(async move {
        let result = match action {
            Action::SetVolume(v) => client.set_volume(v).await,
            Action::ToggleMute(m) => client.set_mute(m).await,
            Action::SetSource(s) => client.set_source(s).await,
            Action::Play => client.play().await,
            Action::Pause => client.pause().await,
            Action::NextTrack => client.next_track().await,
            Action::PreviousTrack => client.previous_track().await,
            Action::SeekForward => client.seek(10).await,
            Action::SeekBackward => client.seek(-10).await,
            Action::SetCableMode => Ok(()), // Cable mode is read-only in practice
            Action::SetStandbyMode(m) => client.set_standby_mode(m).await,
            Action::SetMaxVolume(v) => {
                client
                    .set_data(
                        kef_api::paths::MAX_VOLUME,
                        kef_api::types::ApiValue::i32(v),
                    )
                    .await
            }
            Action::SetFrontLed(on) => client.set_front_led_disabled(!on).await,
            Action::SetStartupTone(on) => client.set_startup_tone(on).await,
        };
        if let Err(e) = result {
            tracing::warn!("API action failed: {e}");
            let _ = tx.send(Event::SpeakerError(format!("Action failed: {e}")));
        }
    });
}

fn init_logging() {
    // Log to file so stdout stays clean for TUI
    let state_dir = dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("kefctl");
    let _ = std::fs::create_dir_all(&state_dir);
    let log_path = state_dir.join("kefctl.log");

    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::{EnvFilter, fmt};

        let filter = EnvFilter::try_from_env("KEFCTL_LOG")
            .unwrap_or_else(|_| EnvFilter::new("kefctl=info"));

        let file_layer = fmt::layer()
            .with_writer(std::sync::Mutex::new(file))
            .with_ansi(false);

        tracing_subscriber::registry()
            .with(file_layer.with_filter(filter))
            .init();
        tracing::debug!("kefctl logging initialized to {}", log_path.display());
    }
}

async fn cmd_discover() {
    println!("Discovering KEF speakers on the network...");
    match discovery::discover_speakers(Duration::from_secs(5)).await {
        Ok(speakers) if speakers.is_empty() => {
            println!("No KEF speakers found.");
        }
        Ok(speakers) => {
            for s in &speakers {
                println!("  {} — {}:{}", s.name, s.ip, s.port);
            }
        }
        Err(e) => {
            eprintln!("Discovery error: {e}");
            std::process::exit(1);
        }
    }
}

fn require_speaker(ip: Option<&str>) -> String {
    if let Some(s) = ip {
        return s.to_string();
    }
    if let Some(cached) = config::load_cached_ip() {
        return cached;
    }
    eprintln!(
        "No speaker specified. Use --speaker <ip>, \
         set ip in ~/.config/kefctl/config.toml, \
         or run 'kefctl discover' to find speakers."
    );
    std::process::exit(1);
}

fn parse_speaker_ip(ip_str: &str) -> IpAddr {
    ip_str.parse().unwrap_or_else(|_| {
        eprintln!("Invalid IP address: {ip_str}");
        std::process::exit(1);
    })
}

fn resolve_speaker(ip: Option<&str>) -> IpAddr {
    let s = require_speaker(ip);
    parse_speaker_ip(&s)
}

async fn cmd_status(ip: IpAddr) {
    let client = KefClient::new(ip);
    match client.fetch_full_state().await {
        Ok(state) => {
            println!("Speaker: {} ({})", state.name, state.model);
            println!("Firmware: {}", state.firmware);
            println!("IP: {}  MAC: {}", state.ip, state.mac);
            println!("Source: {}", state.source.display_name());
            println!(
                "Volume: {}{}  (max: {})",
                state.volume,
                if state.muted { " [MUTED]" } else { "" },
                state.max_volume,
            );
            println!("Cable: {}", state.cable_mode.display_name());
            println!("Standby: {}", state.standby_mode.display_name());
            println!(
                "Front LED: {}  Startup tone: {}",
                if state.front_led { "on" } else { "off" },
                if state.startup_tone { "on" } else { "off" },
            );
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn cmd_get_source(ip: IpAddr) {
    let client = KefClient::new(ip);
    match client.get_source().await {
        Ok(source) => println!("{}", source.display_name()),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn cmd_set_source(ip: IpAddr, source_arg: SourceArg) {
    let source = match source_arg {
        SourceArg::Usb => Source::Usb,
        SourceArg::Wifi => Source::Wifi,
        SourceArg::Bluetooth => Source::Bluetooth,
        SourceArg::Tv => Source::Tv,
        SourceArg::Optical => Source::Optical,
        SourceArg::Coaxial => Source::Coaxial,
        SourceArg::Analog => Source::Analog,
    };

    let client = KefClient::new(ip);
    match client.set_source(source).await {
        Ok(()) => println!("Source set to {}", source.display_name()),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn cmd_get_volume(ip: IpAddr) {
    let client = KefClient::new(ip);
    match client.get_volume().await {
        Ok(vol) => println!("{vol}"),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn cmd_set_volume(ip: IpAddr, level: i32) {
    if !(0..=100).contains(&level) {
        eprintln!("Volume must be between 0 and 100");
        std::process::exit(1);
    }

    let client = KefClient::new(ip);
    match client.set_volume(level).await {
        Ok(()) => println!("Volume set to {level}"),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
