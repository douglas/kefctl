mod app;
mod cli;
mod config;
mod error;
mod kef_api;

use clap::Parser;

use cli::{Cli, Commands};
use config::Config;

fn main() {
    let cli = Cli::parse();
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {e}");
            std::process::exit(1);
        }
    };

    let speaker_ip = cli
        .speaker
        .as_deref()
        .or(config.speaker.ip.as_deref());

    match cli.command {
        Some(Commands::Discover) => {
            println!("Discovering KEF speakers on the network...");
            println!("(not yet implemented)");
        }
        Some(Commands::Status) => {
            let ip = require_speaker(speaker_ip);
            println!("Fetching status from {ip}...");
            println!("(not yet implemented)");
        }
        Some(Commands::Source { source: Some(s) }) => {
            let ip = require_speaker(speaker_ip);
            println!("Setting source to '{s}' on {ip}...");
            println!("(not yet implemented)");
        }
        Some(Commands::Source { source: None }) => {
            let ip = require_speaker(speaker_ip);
            println!("Fetching current source from {ip}...");
            println!("(not yet implemented)");
        }
        Some(Commands::Volume { level: Some(v) }) => {
            let ip = require_speaker(speaker_ip);
            println!("Setting volume to {v} on {ip}...");
            println!("(not yet implemented)");
        }
        Some(Commands::Volume { level: None }) => {
            let ip = require_speaker(speaker_ip);
            println!("Fetching current volume from {ip}...");
            println!("(not yet implemented)");
        }
        None => {
            if cli.demo {
                println!("Launching TUI in demo mode...");
                println!("(not yet implemented)");
            } else {
                let _ip = speaker_ip;
                println!("Launching TUI...");
                println!("(not yet implemented)");
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
