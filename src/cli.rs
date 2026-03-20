//! CLI argument parsing (clap derive).

use clap::{Parser, Subcommand, ValueEnum};

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
        /// Source to switch to
        #[arg(value_enum)]
        source: Option<SourceArg>,
    },

    /// Get or set the volume
    Volume {
        /// Volume level (0-100)
        level: Option<i32>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SourceArg {
    Usb,
    Wifi,
    Bluetooth,
    Tv,
    Optical,
    Coaxial,
    Analog,
}
