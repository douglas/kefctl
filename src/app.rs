use std::net::IpAddr;

use ratatui::widgets::ListState;

use crate::kef_api::types::{
    BassExtension, CableMode, EqProfile, Source, StandbyMode, SubPolarity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Panel {
    #[default]
    Status,
    Source,
    Eq,
    Settings,
    Network,
}

impl Panel {
    pub const ALL: &[Panel] = &[
        Panel::Status,
        Panel::Source,
        Panel::Eq,
        Panel::Settings,
        Panel::Network,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Panel::Status => "Status",
            Panel::Source => "Source",
            Panel::Eq => "EQ / DSP",
            Panel::Settings => "Settings",
            Panel::Network => "Network",
        }
    }

    pub fn index(self) -> usize {
        Panel::ALL.iter().position(|&p| p == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Sidebar,
    Main,
}

#[derive(Debug, Clone)]
pub struct SpeakerState {
    pub name: String,
    pub model: String,
    pub firmware: String,
    pub ip: IpAddr,
    pub mac: String,
    pub source: Source,
    pub volume: i32,
    pub muted: bool,
    pub cable_mode: CableMode,
    pub standby_mode: StandbyMode,
    pub max_volume: i32,
    pub front_led: bool,
    pub startup_tone: bool,
    pub eq_profile: EqProfile,
    pub artist: Option<String>,
    pub track: Option<String>,
    pub duration: Option<u32>,
    pub position: Option<u32>,
    pub playing: bool,
}

impl Default for SpeakerState {
    fn default() -> Self {
        Self {
            name: String::new(),
            model: String::new(),
            firmware: String::new(),
            ip: IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
            mac: String::new(),
            source: Source::Wifi,
            volume: 0,
            muted: false,
            cable_mode: CableMode::default(),
            standby_mode: StandbyMode::default(),
            max_volume: 100,
            front_led: true,
            startup_tone: true,
            eq_profile: EqProfile::default(),
            artist: None,
            track: None,
            duration: None,
            position: None,
            playing: false,
        }
    }
}

impl SpeakerState {
    pub fn demo() -> Self {
        Self {
            name: "Living Room LSX II".to_string(),
            model: "LSX II".to_string(),
            firmware: "4.3.1.0240".to_string(),
            ip: IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 50, 17)),
            mac: "AA:BB:CC:DD:EE:FF".to_string(),
            source: Source::Wifi,
            volume: 35,
            muted: false,
            cable_mode: CableMode::Wireless,
            standby_mode: StandbyMode::SixtyMinutes,
            max_volume: 80,
            front_led: true,
            startup_tone: true,
            eq_profile: EqProfile {
                name: "Standard".to_string(),
                treble: 0.0,
                bass_extension: BassExtension::Standard,
                desk_mode: true,
                desk_db: -3.0,
                wall_mode: false,
                wall_db: 0.0,
                sub_out: false,
                sub_gain: 0.0,
                sub_polarity: SubPolarity::Positive,
                sub_crossover: 80,
                phase_correction: true,
            },
            artist: Some("Nils Frahm".to_string()),
            track: Some("Says".to_string()),
            duration: Some(582),
            position: Some(127),
            playing: true,
        }
    }
}

#[derive(Debug)]
pub struct DiscoveredSpeaker {
    pub name: String,
    pub ip: IpAddr,
    pub port: u16,
}

pub struct App {
    pub speaker: SpeakerState,
    pub panel: Panel,
    pub connection: ConnectionState,
    pub focus: Focus,
    pub sidebar_state: ListState,
    pub source_list_state: ListState,
    pub eq_focus: usize,
    pub settings_focus: usize,
    pub network_speakers: Vec<DiscoveredSpeaker>,
    pub notification: Option<String>,
    pub should_quit: bool,
    pub demo: bool,
}

impl App {
    pub fn new_demo() -> Self {
        let mut sidebar_state = ListState::default();
        sidebar_state.select(Some(0));

        Self {
            speaker: SpeakerState::demo(),
            panel: Panel::Status,
            connection: ConnectionState::Connected,
            focus: Focus::Sidebar,
            sidebar_state,
            source_list_state: ListState::default(),
            eq_focus: 0,
            settings_focus: 0,
            network_speakers: vec![],
            notification: None,
            should_quit: false,
            demo: true,
        }
    }

    pub fn select_panel(&mut self, panel: Panel) {
        self.panel = panel;
        self.sidebar_state.select(Some(panel.index()));
    }

    pub fn next_panel(&mut self) {
        let idx = (self.panel.index() + 1) % Panel::ALL.len();
        self.select_panel(Panel::ALL[idx]);
    }

    pub fn prev_panel(&mut self) {
        let idx = if self.panel.index() == 0 {
            Panel::ALL.len() - 1
        } else {
            self.panel.index() - 1
        };
        self.select_panel(Panel::ALL[idx]);
    }
}
