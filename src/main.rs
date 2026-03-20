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
use std::time::Duration;

use clap::Parser;

use app::App;
use cli::{Cli, Commands};
use config::Config;
use event::{Event, EventHandler};
use kef_api::KefClient;
use kef_api::types::Source;

fn main() {
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
        Some(Commands::Discover) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(cmd_discover());
        }
        Some(Commands::Status) => {
            let ip = parse_speaker_ip(require_speaker(speaker_ip));
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(cmd_status(ip));
        }
        Some(Commands::Source { source: Some(s) }) => {
            let ip = parse_speaker_ip(require_speaker(speaker_ip));
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(cmd_set_source(ip, &s));
        }
        Some(Commands::Source { source: None }) => {
            let ip = parse_speaker_ip(require_speaker(speaker_ip));
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(cmd_get_source(ip));
        }
        Some(Commands::Volume { level: Some(v) }) => {
            let ip = parse_speaker_ip(require_speaker(speaker_ip));
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(cmd_set_volume(ip, v));
        }
        Some(Commands::Volume { level: None }) => {
            let ip = parse_speaker_ip(require_speaker(speaker_ip));
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(cmd_get_volume(ip));
        }
        None => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            if cli.demo {
                rt.block_on(run_tui(App::new_demo(), config));
            } else {
                let _ip = speaker_ip;
                eprintln!("TUI with live speaker not yet wired — use --demo for now");
                std::process::exit(1);
            }
        }
    }
}

async fn run_tui(mut app: App, config: Config) {
    let mut terminal = tui::init().expect("failed to init terminal");

    // Install panic hook that restores terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = tui::restore();
        original_hook(panic_info);
    }));

    let tick_rate = Duration::from_millis(config.ui.refresh_ms);
    let mut events = EventHandler::new(tick_rate);

    loop {
        terminal
            .draw(|frame| ui::draw(frame, &mut app))
            .expect("failed to draw");

        match events.next().await {
            Some(Event::Key(key)) => {
                app.handle_key(key);
                if app.should_quit {
                    break;
                }
            }
            Some(Event::Tick) => {
                app.tick();
            }
            Some(Event::Resize(_, _)) => {
                // Terminal auto-redraws on next loop
            }
            Some(Event::SpeakerUpdate(state)) => {
                app.speaker = *state;
            }
            Some(Event::SpeakerError(msg)) => {
                app.notification = Some(msg);
            }
            None => break,
        }
    }

    tui::restore().expect("failed to restore terminal");
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

fn require_speaker(ip: Option<&str>) -> &str {
    ip.unwrap_or_else(|| {
        eprintln!(
            "No speaker specified. Use --speaker <ip>, \
             set ip in ~/.config/kefctl/config.toml, \
             or run 'kefctl discover' to find speakers."
        );
        std::process::exit(1);
    })
}

fn parse_speaker_ip(ip_str: &str) -> IpAddr {
    ip_str.parse().unwrap_or_else(|_| {
        eprintln!("Invalid IP address: {ip_str}");
        std::process::exit(1);
    })
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

async fn cmd_set_source(ip: IpAddr, source_name: &str) {
    let source = match source_name.to_lowercase().as_str() {
        "usb" => Source::Usb,
        "wifi" | "wi-fi" => Source::Wifi,
        "bluetooth" | "bt" => Source::Bluetooth,
        "tv" | "hdmi" => Source::Tv,
        "optical" | "opt" => Source::Optical,
        "coaxial" | "coax" => Source::Coaxial,
        "analog" | "aux" => Source::Analog,
        other => {
            eprintln!(
                "Unknown source '{other}'. Valid: usb, wifi, bluetooth, tv, optical, coaxial, analog"
            );
            std::process::exit(1);
        }
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
