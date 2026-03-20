use clap::{Parser, Subcommand};

/// KEF LSX II TUI Controller
#[derive(Debug, Parser)]
#[command(name = "kefctl", version, about)]
pub struct Cli {
    /// Speaker IP address (overrides config and discovery)
    #[arg(long)]
    pub speaker: Option<String>,

    /// Run with mock data (no speaker required)
    #[arg(long)]
    pub demo: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List KEF speakers on the network
    Discover,

    /// Print current speaker status
    Status,

    /// Get or set the active source
    Source {
        /// Source to switch to (usb, wifi, bluetooth, tv, optical, coaxial, analog)
        source: Option<String>,
    },

    /// Get or set the volume
    Volume {
        /// Volume level (0-100)
        level: Option<i32>,
    },
}
