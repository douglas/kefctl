//! kefctl — TUI controller for KEF W2-platform speakers.
#![forbid(unsafe_code)]
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
use cli::{Cli, Commands, MuteArg, SourceArg};
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
        Some(Commands::Mute { state }) => {
            let ip = resolve_speaker(speaker_ip);
            cmd_mute(ip, state).await;
        }
        Some(Commands::Toggle) => {
            let ip = resolve_speaker(speaker_ip);
            cmd_toggle(ip, &config).await;
        }
        Some(Commands::Waybar) => {
            cmd_waybar(speaker_ip).await;
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
/// Uses a short 2-second timeout so fallback to discovery is fast.
async fn try_cached_ip() -> Option<IpAddr> {
    let ip = config::load_cached_ip()?;
    eprintln!("Trying cached speaker {ip}...");

    // Quick probe with short timeout — don't make the user wait
    let probe_client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(1))
        .timeout(Duration::from_secs(2))
        .build()
        .ok()?;
    let url = format!("http://{ip}/api/getData?path=settings%3A%2FdeviceName&roles=value");
    if probe_client.get(&url).send().await.is_ok() {
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
            Action::SetSource(s) => {
                config::save_last_source(s.serde_name());
                client.set_source(s).await
            }
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
            Action::SetEqProfile(eq) => client.set_eq_profile(eq).await,
        };
        if let Err(e) = result {
            tracing::warn!("API action failed: {e}");
            let _ = tx.send(Event::SpeakerError(format!("Action failed: {e}")));
        }
    });
}

fn init_logging() {
    // Log to file so stdout stays clean for TUI
    let Some(state_dir) = dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .map(|d| d.join("kefctl"))
    else {
        return;
    };
    #[cfg(unix)]
    let _ = {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new().recursive(true).mode(0o700).create(&state_dir)
    };
    #[cfg(not(unix))]
    let _ = std::fs::create_dir_all(&state_dir);
    let log_path = state_dir.join("kefctl.log");

    let mut opts = std::fs::OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    if let Ok(file) = opts.open(&log_path) {
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
        eprintln!("Using cached speaker {cached} (use --speaker to override)");
        return cached.to_string();
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

fn unwrap_or_exit<T>(result: Result<T, impl std::fmt::Display>) -> T {
    match result {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn cmd_status(ip: IpAddr) {
    let client = KefClient::new(ip);
    let state = unwrap_or_exit(client.fetch_full_state().await);
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

async fn cmd_get_source(ip: IpAddr) {
    let client = KefClient::new(ip);
    let source = unwrap_or_exit(client.get_source().await);
    println!("{}", source.display_name());
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
    unwrap_or_exit(client.set_source(source).await);
    config::save_last_source(source.serde_name());
    println!("Source set to {}", source.display_name());
}

async fn cmd_get_volume(ip: IpAddr) {
    let client = KefClient::new(ip);
    let vol = unwrap_or_exit(client.get_volume().await);
    println!("{vol}");
}

async fn cmd_set_volume(ip: IpAddr, level: i32) {
    if !(0..=100).contains(&level) {
        eprintln!("Volume must be between 0 and 100");
        std::process::exit(1);
    }

    let client = KefClient::new(ip);
    unwrap_or_exit(client.set_volume(level).await);
    println!("Volume set to {level}");
}

async fn cmd_mute(ip: IpAddr, state: Option<MuteArg>) {
    let client = KefClient::new(ip);
    let target = match state {
        Some(MuteArg::On) => true,
        Some(MuteArg::Off) => false,
        None => !unwrap_or_exit(client.get_mute().await),
    };
    unwrap_or_exit(client.set_mute(target).await);
    println!("{}", if target { "Muted" } else { "Unmuted" });
}

/// Resolve which source to wake the speaker to.
/// Priority: last-used source > config `default_source` > USB.
fn resolve_wake_source(config: &Config) -> Source {
    if let Some(name) = config::load_last_source() {
        if let Some(source) = Source::from_serde_name(&name) {
            if source != Source::Standby {
                return source;
            }
        }
    }
    if let Some(ref name) = config.speaker.default_source {
        if let Some(source) = Source::from_serde_name(name) {
            if source == Source::Standby {
                tracing::warn!("default_source cannot be 'standby', using USB");
            } else {
                return source;
            }
        } else {
            tracing::warn!("Unknown default_source '{name}', using USB");
        }
    }
    Source::Usb
}

async fn cmd_toggle(ip: IpAddr, config: &Config) {
    let client = KefClient::new(ip);
    let current = unwrap_or_exit(client.get_source().await);

    if current == Source::Standby {
        let wake = resolve_wake_source(config);
        unwrap_or_exit(client.set_source(wake).await);
        config::save_last_source(wake.serde_name());
        println!("Waking speaker to {}", wake.display_name());
    } else {
        config::save_last_source(current.serde_name());
        unwrap_or_exit(client.set_source(Source::Standby).await);
        println!("Speaker entering standby");
    }
}

async fn cmd_waybar(speaker_ip: Option<&str>) {
    let Some(ip) = resolve_waybar_ip(speaker_ip) else {
        print_waybar_json("\u{f04c4}", "KEF \u{00b7} Offline", "off");
        return;
    };

    let kef = KefClient::new(ip);

    match kef.get_source().await {
        Ok(Source::Standby) => {
            print_waybar_json("\u{f04c4}", "KEF \u{00b7} Standby", "off");
        }
        Ok(source) => {
            let tooltip = match (kef.get_volume().await, kef.get_mute().await) {
                (Ok(vol), Ok(true)) => {
                    format!(
                        "KEF \u{00b7} {} \u{00b7} {}% [MUTED]",
                        source.display_name(),
                        vol
                    )
                }
                (Ok(vol), _) => {
                    format!(
                        "KEF \u{00b7} {} \u{00b7} {}%",
                        source.display_name(),
                        vol
                    )
                }
                _ => format!("KEF \u{00b7} {}", source.display_name()),
            };
            print_waybar_json("\u{f04c3}", &tooltip, "on");
        }
        Err(e) => {
            tracing::debug!("Waybar: speaker unreachable: {e}");
            print_waybar_json("\u{f04c4}", "KEF \u{00b7} Offline", "off");
        }
    }
}

fn print_waybar_json(text: &str, tooltip: &str, class: &str) {
    let obj = serde_json::json!({
        "text": text,
        "tooltip": tooltip,
        "class": class,
        "alt": class,
    });
    println!("{obj}");
}

fn resolve_waybar_ip(speaker_ip: Option<&str>) -> Option<IpAddr> {
    if let Some(s) = speaker_ip {
        return s.parse().ok();
    }
    if let Ok(config) = Config::load() {
        if let Some(ref ip) = config.speaker.ip {
            return ip.parse().ok();
        }
    }
    config::load_cached_ip()
}
