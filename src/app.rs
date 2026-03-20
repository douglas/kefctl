use std::net::IpAddr;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use crate::kef_api::types::{
    BassExtension, CableMode, EqProfile, Source, StandbyMode, SubPolarity,
};

#[derive(Debug, Clone)]
pub enum Action {
    SetVolume(i32),
    ToggleMute(bool),
    SetSource(Source),
    Play,
    Pause,
    NextTrack,
    PreviousTrack,
    SeekForward,
    SeekBackward,
    SetCableMode(CableMode),
    SetStandbyMode(i32),
    SetMaxVolume(i32),
    SetFrontLed(bool),
    SetStartupTone(bool),
}

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
    pub notification_ttl: u8,
    pub show_help: bool,
    pub should_quit: bool,
    pub demo: bool,
}

impl App {
    pub fn new_live(state: SpeakerState) -> Self {
        let mut sidebar_state = ListState::default();
        sidebar_state.select(Some(0));
        let mut source_list_state = ListState::default();
        source_list_state.select(Some(0));

        Self {
            speaker: state,
            panel: Panel::Status,
            connection: ConnectionState::Connected,
            focus: Focus::Sidebar,
            sidebar_state,
            source_list_state,
            eq_focus: 0,
            settings_focus: 0,
            network_speakers: vec![],
            notification: None,
            notification_ttl: 0,
            show_help: false,
            should_quit: false,
            demo: false,
        }
    }

    pub fn new_demo() -> Self {
        let mut sidebar_state = ListState::default();
        sidebar_state.select(Some(0));
        let mut source_list_state = ListState::default();
        source_list_state.select(Some(0));

        Self {
            speaker: SpeakerState::demo(),
            panel: Panel::Status,
            connection: ConnectionState::Connected,
            focus: Focus::Sidebar,
            sidebar_state,
            source_list_state,
            eq_focus: 0,
            settings_focus: 0,
            network_speakers: vec![],
            notification: None,
            notification_ttl: 0,
            show_help: false,
            should_quit: false,
            demo: true,
        }
    }

    pub fn set_notification(&mut self, msg: String) {
        self.notification = Some(msg);
        self.notification_ttl = 3;
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

    pub fn tick(&mut self) {
        if self.speaker.playing
            && let (Some(pos), Some(dur)) = (self.speaker.position, self.speaker.duration)
            && pos < dur
        {
            self.speaker.position = Some(pos + 1);
        }

        // Auto-dismiss notifications
        if self.notification.is_some() {
            if self.notification_ttl == 0 {
                self.notification = None;
            } else {
                self.notification_ttl -= 1;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        // Help overlay intercepts all keys
        if self.show_help {
            match key.code {
                KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                    self.show_help = false;
                }
                _ => {}
            }
            return None;
        }

        // Global keys
        match key.code {
            KeyCode::Char('?') => {
                self.show_help = true;
                return None;
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
                return None;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return None;
            }
            KeyCode::Tab => {
                self.next_panel();
                return None;
            }
            KeyCode::BackTab => {
                self.prev_panel();
                return None;
            }
            // Global playback controls
            KeyCode::Char('m') => {
                self.speaker.muted = !self.speaker.muted;
                return Some(Action::ToggleMute(self.speaker.muted));
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if self.speaker.volume < self.speaker.max_volume {
                    self.speaker.volume += 1;
                }
                return Some(Action::SetVolume(self.speaker.volume));
            }
            KeyCode::Char('-') => {
                if self.speaker.volume > 0 {
                    self.speaker.volume -= 1;
                }
                return Some(Action::SetVolume(self.speaker.volume));
            }
            KeyCode::Char(' ') => {
                self.speaker.playing = !self.speaker.playing;
                return if self.speaker.playing {
                    Some(Action::Play)
                } else {
                    Some(Action::Pause)
                };
            }
            KeyCode::Char('n') if self.focus == Focus::Sidebar => {
                return Some(Action::NextTrack);
            }
            KeyCode::Char('p') if self.focus == Focus::Sidebar => {
                return Some(Action::PreviousTrack);
            }
            KeyCode::Char('f') => return Some(Action::SeekForward),
            KeyCode::Char('b') if self.focus == Focus::Sidebar => {
                return Some(Action::SeekBackward)
            }
            _ => {}
        }

        match self.focus {
            Focus::Sidebar => {
                self.handle_key_sidebar(key);
                None
            }
            Focus::Main => self.handle_key_main(key),
        }
    }

    fn handle_key_sidebar(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.next_panel(),
            KeyCode::Char('k') | KeyCode::Up => self.prev_panel(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                self.focus = Focus::Main;
            }
            _ => {}
        }
    }

    fn handle_key_main(&mut self, key: KeyEvent) -> Option<Action> {
        // Esc always returns to sidebar
        if key.code == KeyCode::Esc {
            self.focus = Focus::Sidebar;
            return None;
        }

        match self.panel {
            Panel::Status => {
                if key.code == KeyCode::Char('h') {
                    self.focus = Focus::Sidebar;
                }
                None
            }
            Panel::Source => self.handle_key_source(key),
            Panel::Eq => {
                self.handle_key_eq(key);
                None // EQ API integration deferred — complex nested structure
            }
            Panel::Settings => self.handle_key_settings(key),
            Panel::Network => {
                if key.code == KeyCode::Char('h') {
                    self.focus = Focus::Sidebar;
                }
                None
            }
        }
    }

    fn handle_key_source(&mut self, key: KeyEvent) -> Option<Action> {
        let count = Source::ALL.len();
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let i = self.source_list_state.selected().unwrap_or(0);
                self.source_list_state.select(Some((i + 1) % count));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = self.source_list_state.selected().unwrap_or(0);
                self.source_list_state
                    .select(Some(if i == 0 { count - 1 } else { i - 1 }));
            }
            KeyCode::Enter => {
                if let Some(i) = self.source_list_state.selected() {
                    if i < count {
                        let source = Source::ALL[i];
                        self.speaker.source = source;
                        return Some(Action::SetSource(source));
                    }
                }
            }
            KeyCode::Char('h') => self.focus = Focus::Sidebar,
            _ => {}
        }
        None
    }

    fn handle_key_eq(&mut self, key: KeyEvent) {
        let max_focus = 6; // 0-6 are the editable rows
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.eq_focus < max_focus {
                    self.eq_focus += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.eq_focus > 0 {
                    self.eq_focus -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => self.eq_adjust(1),
            KeyCode::Left | KeyCode::Char('h') if self.eq_focus > 0 => self.eq_adjust(-1),
            KeyCode::Char('h') => self.focus = Focus::Sidebar,
            _ => {}
        }
    }

    fn eq_adjust(&mut self, dir: i32) {
        let eq = &mut self.speaker.eq_profile;
        match self.eq_focus {
            0 => {} // Profile name — not adjustable inline
            1 => {
                // Treble: -5.0 to +5.0 in 0.5 steps
                eq.treble = (eq.treble + dir as f64 * 0.5).clamp(-5.0, 5.0);
            }
            2 => {
                // Bass extension
                eq.bass_extension = if dir > 0 {
                    eq.bass_extension.cycle_next()
                } else {
                    eq.bass_extension.cycle_prev()
                };
            }
            3 => {
                // Desk mode: toggle on/off, or adjust dB if on
                if !eq.desk_mode {
                    eq.desk_mode = true;
                } else if dir > 0 {
                    eq.desk_db = (eq.desk_db + 0.5).clamp(-6.0, 0.0);
                } else {
                    eq.desk_db = (eq.desk_db - 0.5).clamp(-6.0, 0.0);
                    if eq.desk_db <= -6.0 {
                        eq.desk_mode = false;
                    }
                }
            }
            4 => {
                // Wall mode: same pattern as desk
                if !eq.wall_mode {
                    eq.wall_mode = true;
                } else if dir > 0 {
                    eq.wall_db = (eq.wall_db + 0.5).clamp(-6.0, 0.0);
                } else {
                    eq.wall_db = (eq.wall_db - 0.5).clamp(-6.0, 0.0);
                    if eq.wall_db <= -6.0 {
                        eq.wall_mode = false;
                    }
                }
            }
            5 => {
                // Sub out toggle
                eq.sub_out = !eq.sub_out;
            }
            6 => {
                // Phase correction toggle
                eq.phase_correction = !eq.phase_correction;
            }
            _ => {}
        }
    }

    fn handle_key_settings(&mut self, key: KeyEvent) -> Option<Action> {
        let max_focus = 4; // 0-4 are the settings rows
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.settings_focus < max_focus {
                    self.settings_focus += 1;
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.settings_focus > 0 {
                    self.settings_focus -= 1;
                }
                None
            }
            KeyCode::Right | KeyCode::Char('l') => self.settings_cycle(1),
            KeyCode::Left | KeyCode::Char('h') if self.settings_focus > 0 => {
                self.settings_cycle(-1)
            }
            KeyCode::Char('h') => {
                self.focus = Focus::Sidebar;
                None
            }
            _ => None,
        }
    }

    fn settings_cycle(&mut self, dir: i32) -> Option<Action> {
        match self.settings_focus {
            0 => {
                self.speaker.cable_mode = match self.speaker.cable_mode {
                    CableMode::Wired => CableMode::Wireless,
                    CableMode::Wireless => CableMode::Wired,
                };
                Some(Action::SetCableMode(self.speaker.cable_mode))
            }
            1 => {
                self.speaker.standby_mode = if dir > 0 {
                    self.speaker.standby_mode.cycle_next()
                } else {
                    self.speaker.standby_mode.cycle_prev()
                };
                let minutes = match self.speaker.standby_mode {
                    StandbyMode::TwentyMinutes => 20,
                    StandbyMode::SixtyMinutes => 60,
                    StandbyMode::Never => 0,
                };
                Some(Action::SetStandbyMode(minutes))
            }
            2 => {
                self.speaker.max_volume =
                    (self.speaker.max_volume + dir * 5).clamp(10, 100);
                Some(Action::SetMaxVolume(self.speaker.max_volume))
            }
            3 => {
                self.speaker.front_led = !self.speaker.front_led;
                Some(Action::SetFrontLed(self.speaker.front_led))
            }
            4 => {
                self.speaker.startup_tone = !self.speaker.startup_tone;
                Some(Action::SetStartupTone(self.speaker.startup_tone))
            }
            _ => None,
        }
    }
}
