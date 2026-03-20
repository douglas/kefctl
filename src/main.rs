mod app;
mod cli;
mod config;
mod error;
mod kef_api;

use std::net::IpAddr;

use clap::Parser;

use cli::{Cli, Commands};
use config::Config;
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
            println!("Discovering KEF speakers on the network...");
            println!("(not yet implemented — see Phase 4)");
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
            if cli.demo {
                println!("Launching TUI in demo mode...");
                println!("(not yet implemented — see Phase 5)");
            } else {
                let _ip = speaker_ip;
                println!("Launching TUI...");
                println!("(not yet implemented — see Phase 5)");
            }
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
