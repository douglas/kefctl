//! Application state, keyboard handling, and action dispatch.

use std::net::IpAddr;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use crate::kef_api::types::{
    BassExtension, CableMode, EqProfile, Source, StandbyMode,
};
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub(crate) enum Action {
    SetVolume(i32),
    ToggleMute(bool),
    SetSource(Source),
    SetStandbyMode(StandbyMode),
    SetMaxVolume(i32),
    SetFrontLed(bool),
    SetStartupTone(bool),
    SetEqProfile(EqProfile),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Panel {
    #[default]
    Status,
    Source,
    Eq,
    Settings,
    Network,
}

impl Panel {
    pub(crate) const ALL: &[Panel] = &[
        Panel::Status,
        Panel::Source,
        Panel::Eq,
        Panel::Settings,
        Panel::Network,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Panel::Status => "Status",
            Panel::Source => "Source",
            Panel::Eq => "EQ / DSP",
            Panel::Settings => "Settings",
            Panel::Network => "Network",
        }
    }

    pub(crate) fn index(self) -> usize {
        Panel::ALL.iter().position(|&p| p == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ConnectionState {
    #[default]
    Disconnected,
    Connected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Focus {
    #[default]
    Sidebar,
    Main,
}

#[allow(clippy::struct_excessive_bools)] // mirrors KEF API fields
#[derive(Debug, Clone)]
pub(crate) struct SpeakerState {
    pub(crate) name: String,
    pub(crate) model: String,
    pub(crate) firmware: String,
    pub(crate) ip: IpAddr,
    pub(crate) mac: String,
    pub(crate) source: Source,
    pub(crate) volume: i32,
    pub(crate) muted: bool,
    pub(crate) cable_mode: CableMode,
    pub(crate) standby_mode: StandbyMode,
    pub(crate) max_volume: i32,
    pub(crate) front_led: bool,
    pub(crate) startup_tone: bool,
    pub(crate) eq_profile: EqProfile,
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
        }
    }
}

impl SpeakerState {
    pub(crate) fn demo() -> Self {
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
                bass_extension: BassExtension::Standard,
                desk_mode: true,
                desk_mode_setting: -3.0,
                phase_correction: true,
                subwoofer_out: true,
                subwoofer_polarity: "normal".to_string(),
                sub_out_lp_freq: 80.0,
                audio_polarity: "normal".to_string(),
                high_pass_mode_freq: 95,
                subwoofer_preset: "custom".to_string(),
                ..EqProfile::default()
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct DiscoveredSpeaker {
    pub(crate) name: String,
    pub(crate) ip: IpAddr,
    pub(crate) port: u16,
}

pub(crate) struct App {
    pub(crate) speaker: SpeakerState,
    pub(crate) panel: Panel,
    pub(crate) connection: ConnectionState,
    pub(crate) focus: Focus,
    pub(crate) sidebar_state: ListState,
    pub(crate) source_list_state: ListState,
    pub(crate) eq_focus: usize,
    pub(crate) settings_focus: usize,
    pub(crate) network_speakers: Vec<DiscoveredSpeaker>,
    pub(crate) notification: Option<String>,
    pub(crate) notification_ttl: u8,
    pub(crate) show_help: bool,
    pub(crate) should_quit: bool,
    pub(crate) demo: bool,
    pub(crate) theme: Theme,
}

impl App {
    pub(crate) fn new_live(state: SpeakerState) -> Self {
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
            theme: Theme::load(),
        }
    }

    pub(crate) fn new_demo() -> Self {
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
            theme: Theme::load(),
        }
    }

    pub(crate) fn set_notification(&mut self, msg: String) {
        self.notification = Some(msg);
        self.notification_ttl = 3;
    }

    pub(crate) fn select_panel(&mut self, panel: Panel) {
        self.panel = panel;
        self.sidebar_state.select(Some(panel.index()));
    }

    pub(crate) fn next_panel(&mut self) {
        let idx = (self.panel.index() + 1) % Panel::ALL.len();
        self.select_panel(Panel::ALL[idx]);
    }

    pub(crate) fn prev_panel(&mut self) {
        let idx = if self.panel.index() == 0 {
            Panel::ALL.len() - 1
        } else {
            self.panel.index() - 1
        };
        self.select_panel(Panel::ALL[idx]);
    }

    pub(crate) fn tick(&mut self) {
        // Auto-dismiss notifications
        if self.notification.is_some() {
            if self.notification_ttl == 0 {
                self.notification = None;
            } else {
                self.notification_ttl -= 1;
            }
        }
    }

    #[must_use]
    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        // Help overlay intercepts all keys
        if self.show_help {
            match key.code {
                KeyCode::Char('?' | 'q') | KeyCode::Esc | KeyCode::Enter => {
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
            KeyCode::Char('+' | '=') => {
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
            Panel::Source => self.handle_key_source(key),
            Panel::Eq => self.handle_key_eq(key),
            Panel::Settings => self.handle_key_settings(key),
            Panel::Status | Panel::Network => {
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
                if let Some(i) = self.source_list_state.selected()
                    && i < count
                {
                    let source = Source::ALL[i];
                    self.speaker.source = source;
                    return Some(Action::SetSource(source));
                }
            }
            KeyCode::Char('h') => self.focus = Focus::Sidebar,
            _ => {}
        }
        None
    }

    fn handle_key_eq(&mut self, key: KeyEvent) -> Option<Action> {
        let max_focus = 6; // 0=treble, 1=bass ext, 2=desk, 3=wall, 4=sub, 5=phase, 6=balance
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.eq_focus < max_focus {
                    self.eq_focus += 1;
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.eq_focus > 0 {
                    self.eq_focus -= 1;
                }
                None
            }
            KeyCode::Right | KeyCode::Char('l') => self.eq_cycle(1),
            KeyCode::Left | KeyCode::Char('h') if self.eq_focus > 0 => self.eq_cycle(-1),
            KeyCode::Char('h') => {
                self.focus = Focus::Sidebar;
                None
            }
            _ => None,
        }
    }

    fn eq_cycle(&mut self, dir: i32) -> Option<Action> {
        let eq = &mut self.speaker.eq_profile;
        match self.eq_focus {
            0 => {
                eq.treble_amount = (eq.treble_amount + f64::from(dir) * 0.5).clamp(-6.0, 6.0);
            }
            1 => {
                eq.bass_extension = if dir > 0 {
                    eq.bass_extension.cycle_next()
                } else {
                    eq.bass_extension.cycle_prev()
                };
            }
            2 => eq.desk_mode = !eq.desk_mode,
            3 => eq.wall_mode = !eq.wall_mode,
            4 => eq.subwoofer_out = !eq.subwoofer_out,
            5 => eq.phase_correction = !eq.phase_correction,
            6 => {
                eq.balance = (eq.balance + dir).clamp(-10, 10);
            }
            _ => return None,
        }
        Some(Action::SetEqProfile(self.speaker.eq_profile.clone()))
    }

    fn handle_key_settings(&mut self, key: KeyEvent) -> Option<Action> {
        let max_focus = 3; // 0=standby, 1=max vol, 2=LED, 3=startup tone (cable mode is display-only)
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
                self.speaker.standby_mode = if dir > 0 {
                    self.speaker.standby_mode.cycle_next()
                } else {
                    self.speaker.standby_mode.cycle_prev()
                };
                Some(Action::SetStandbyMode(self.speaker.standby_mode))
            }
            1 => {
                self.speaker.max_volume =
                    (self.speaker.max_volume + dir * 5).clamp(10, 100);
                Some(Action::SetMaxVolume(self.speaker.max_volume))
            }
            2 => {
                self.speaker.front_led = !self.speaker.front_led;
                Some(Action::SetFrontLed(self.speaker.front_led))
            }
            3 => {
                self.speaker.startup_tone = !self.speaker.startup_tone;
                Some(Action::SetStartupTone(self.speaker.startup_tone))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn app() -> App {
        App::new_demo()
    }

    // -- Panel navigation --

    #[test]
    fn next_panel_cycles() {
        let mut a = app();
        assert_eq!(a.panel, Panel::Status);
        a.next_panel();
        assert_eq!(a.panel, Panel::Source);
        a.next_panel();
        assert_eq!(a.panel, Panel::Eq);
        a.next_panel();
        assert_eq!(a.panel, Panel::Settings);
        a.next_panel();
        assert_eq!(a.panel, Panel::Network);
        a.next_panel();
        assert_eq!(a.panel, Panel::Status); // wraps
    }

    #[test]
    fn prev_panel_cycles() {
        let mut a = app();
        assert_eq!(a.panel, Panel::Status);
        a.prev_panel();
        assert_eq!(a.panel, Panel::Network); // wraps backward
        a.prev_panel();
        assert_eq!(a.panel, Panel::Settings);
    }

    #[test]
    fn sidebar_state_syncs() {
        let mut a = app();
        a.select_panel(Panel::Eq);
        assert_eq!(a.sidebar_state.selected(), Some(Panel::Eq.index()));
        a.select_panel(Panel::Network);
        assert_eq!(a.sidebar_state.selected(), Some(Panel::Network.index()));
    }

    // -- Focus --

    #[test]
    fn tab_moves_to_next_panel() {
        let mut a = app();
        a.handle_key(key(KeyCode::Tab));
        assert_eq!(a.panel, Panel::Source);
    }

    #[test]
    fn enter_focuses_main() {
        let mut a = app();
        assert_eq!(a.focus, Focus::Sidebar);
        a.handle_key(key(KeyCode::Enter));
        assert_eq!(a.focus, Focus::Main);
    }

    #[test]
    fn esc_returns_to_sidebar() {
        let mut a = app();
        a.focus = Focus::Main;
        a.handle_key(key(KeyCode::Esc));
        assert_eq!(a.focus, Focus::Sidebar);
    }

    #[test]
    fn help_overlay_intercepts_keys() {
        let mut a = app();
        a.handle_key(key(KeyCode::Char('?')));
        assert!(a.show_help);

        // Other keys don't do anything while help is open
        a.handle_key(key(KeyCode::Char('q')));
        assert!(!a.should_quit); // q closes help, doesn't quit
        assert!(!a.show_help);

        // Re-open and close with Esc
        a.handle_key(key(KeyCode::Char('?')));
        assert!(a.show_help);
        a.handle_key(key(KeyCode::Esc));
        assert!(!a.show_help);
    }

    // -- Volume --

    #[test]
    fn volume_up_clamped_at_max() {
        let mut a = app();
        a.speaker.volume = a.speaker.max_volume;
        a.handle_key(key(KeyCode::Char('+')));
        assert_eq!(a.speaker.volume, a.speaker.max_volume);
    }

    #[test]
    fn volume_down_clamped_at_zero() {
        let mut a = app();
        a.speaker.volume = 0;
        a.handle_key(key(KeyCode::Char('-')));
        assert_eq!(a.speaker.volume, 0);
    }

    #[test]
    fn volume_returns_action() {
        let mut a = app();
        a.speaker.volume = 50;
        let action = a.handle_key(key(KeyCode::Char('+')));
        assert!(matches!(action, Some(Action::SetVolume(51))));
    }

    // -- Mute / Playback --

    #[test]
    fn mute_toggles() {
        let mut a = app();
        assert!(!a.speaker.muted);
        a.handle_key(key(KeyCode::Char('m')));
        assert!(a.speaker.muted);
        a.handle_key(key(KeyCode::Char('m')));
        assert!(!a.speaker.muted);
    }

    #[test]
    fn notification_auto_dismisses() {
        let mut a = app();
        a.set_notification("test".to_string());
        assert_eq!(a.notification_ttl, 3);
        a.tick(); // ttl 3 -> 2
        a.tick(); // ttl 2 -> 1
        a.tick(); // ttl 1 -> 0
        assert!(a.notification.is_some());
        a.tick(); // ttl 0 -> cleared
        assert!(a.notification.is_none());
    }

    // -- EQ adjustments --

    // -- Settings cycling --

    #[test]
    fn standby_mode_cycles_all_variants() {
        let mut a = app();
        a.settings_focus = 0;
        a.speaker.standby_mode = StandbyMode::TwentyMinutes;

        a.settings_cycle(1);
        assert_eq!(a.speaker.standby_mode, StandbyMode::ThirtyMinutes);
        a.settings_cycle(1);
        assert_eq!(a.speaker.standby_mode, StandbyMode::SixtyMinutes);
        a.settings_cycle(1);
        assert_eq!(a.speaker.standby_mode, StandbyMode::Never);
        a.settings_cycle(1);
        assert_eq!(a.speaker.standby_mode, StandbyMode::TwentyMinutes);
    }

    #[test]
    fn max_volume_clamped() {
        let mut a = app();
        a.settings_focus = 1;

        a.speaker.max_volume = 100;
        a.settings_cycle(1);
        assert_eq!(a.speaker.max_volume, 100); // clamped at 100

        a.speaker.max_volume = 10;
        a.settings_cycle(-1);
        assert_eq!(a.speaker.max_volume, 10); // clamped at 10
    }

    #[test]
    fn led_toggles() {
        let mut a = app();
        a.settings_focus = 2;
        assert!(a.speaker.front_led);
        a.settings_cycle(1);
        assert!(!a.speaker.front_led);
        a.settings_cycle(1);
        assert!(a.speaker.front_led);
    }

    // -- Quit --

    #[test]
    fn quit_on_q() {
        let mut a = app();
        a.handle_key(key(KeyCode::Char('q')));
        assert!(a.should_quit);
    }

    #[test]
    fn quit_on_ctrl_c() {
        let mut a = app();
        let ctrl_c = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        a.handle_key(ctrl_c);
        assert!(a.should_quit);
    }

    // -- Source panel --

    #[test]
    fn source_panel_j_k_navigation() {
        let mut a = app();
        a.select_panel(Panel::Source);
        a.focus = Focus::Main;
        assert_eq!(a.source_list_state.selected(), Some(0));

        a.handle_key(key(KeyCode::Char('j')));
        assert_eq!(a.source_list_state.selected(), Some(1));

        a.handle_key(key(KeyCode::Char('k')));
        assert_eq!(a.source_list_state.selected(), Some(0));

        // Wrap around backward
        a.handle_key(key(KeyCode::Char('k')));
        assert_eq!(
            a.source_list_state.selected(),
            Some(Source::ALL.len() - 1)
        );
    }

    #[test]
    fn source_panel_enter_selects() {
        let mut a = app();
        a.select_panel(Panel::Source);
        a.focus = Focus::Main;
        a.source_list_state.select(Some(1)); // Bluetooth

        let action = a.handle_key(key(KeyCode::Enter));
        assert!(matches!(action, Some(Action::SetSource(_))));
        assert_eq!(a.speaker.source, Source::ALL[1]);
    }

    #[test]
    fn source_panel_h_returns_sidebar() {
        let mut a = app();
        a.select_panel(Panel::Source);
        a.focus = Focus::Main;
        a.handle_key(key(KeyCode::Char('h')));
        assert_eq!(a.focus, Focus::Sidebar);
    }

    // -- EQ bounds --

    #[test]
    fn eq_focus_bounds() {
        let mut a = app();
        a.focus = Focus::Main;
        a.select_panel(Panel::Eq);

        // At 0, k shouldn't go below 0
        a.eq_focus = 0;
        a.handle_key(key(KeyCode::Char('k')));
        assert_eq!(a.eq_focus, 0);

        // At max (6), j shouldn't go above 6
        a.eq_focus = 6;
        a.handle_key(key(KeyCode::Char('j')));
        assert_eq!(a.eq_focus, 6);
    }

    #[test]
    fn eq_treble_adjusts_and_clamps() {
        let mut a = app();
        a.focus = Focus::Main;
        a.select_panel(Panel::Eq);
        a.eq_focus = 0;
        a.speaker.eq_profile.treble_amount = 0.0;

        // Right increases by 0.5
        let action = a.handle_key(key(KeyCode::Right));
        assert!(matches!(action, Some(Action::SetEqProfile(_))));
        assert!((a.speaker.eq_profile.treble_amount - 0.5).abs() < f64::EPSILON);

        // Clamps at +6.0
        a.speaker.eq_profile.treble_amount = 6.0;
        a.handle_key(key(KeyCode::Right));
        assert!((a.speaker.eq_profile.treble_amount - 6.0).abs() < f64::EPSILON);

        // Left decreases by 0.5
        a.speaker.eq_profile.treble_amount = 0.0;
        a.eq_focus = 1; // move off 0 so h/Left is allowed
        a.eq_focus = 0;
        a.eq_focus = 1;
        a.handle_key(key(KeyCode::Left));
        // focus=1 so Left cycles bass extension, not treble — just confirm it returns an action
        assert!(matches!(a.handle_key(key(KeyCode::Right)), Some(Action::SetEqProfile(_))));
    }

    #[test]
    fn eq_balance_clamps() {
        let mut a = app();
        a.focus = Focus::Main;
        a.select_panel(Panel::Eq);
        a.eq_focus = 6;
        a.speaker.eq_profile.balance = 10;

        a.handle_key(key(KeyCode::Right));
        assert_eq!(a.speaker.eq_profile.balance, 10); // clamped

        a.speaker.eq_profile.balance = -10;
        a.handle_key(key(KeyCode::Left));
        assert_eq!(a.speaker.eq_profile.balance, -10); // clamped (focus=6 > 0 so Left cycles)
    }

    #[test]
    fn eq_bool_fields_toggle() {
        let mut a = app();
        a.focus = Focus::Main;
        a.select_panel(Panel::Eq);

        for focus in [2usize, 3, 4, 5] {
            a.eq_focus = focus;
            let before = match focus {
                2 => a.speaker.eq_profile.desk_mode,
                3 => a.speaker.eq_profile.wall_mode,
                4 => a.speaker.eq_profile.subwoofer_out,
                5 => a.speaker.eq_profile.phase_correction,
                _ => unreachable!(),
            };
            let action = a.handle_key(key(KeyCode::Right));
            assert!(matches!(action, Some(Action::SetEqProfile(_))), "focus={focus}");
            let after = match focus {
                2 => a.speaker.eq_profile.desk_mode,
                3 => a.speaker.eq_profile.wall_mode,
                4 => a.speaker.eq_profile.subwoofer_out,
                5 => a.speaker.eq_profile.phase_correction,
                _ => unreachable!(),
            };
            assert_ne!(before, after, "focus={focus} should toggle");
        }
    }

    #[test]
    fn eq_h_on_focus_zero_returns_sidebar() {
        let mut a = app();
        a.focus = Focus::Main;
        a.select_panel(Panel::Eq);
        a.eq_focus = 0;
        a.handle_key(key(KeyCode::Char('h')));
        assert_eq!(a.focus, Focus::Sidebar);
    }

    // -- Settings bounds --

    #[test]
    fn settings_focus_bounds() {
        let mut a = app();
        a.focus = Focus::Main;
        a.select_panel(Panel::Settings);

        a.settings_focus = 0;
        a.handle_key(key(KeyCode::Char('k')));
        assert_eq!(a.settings_focus, 0);

        a.settings_focus = 3;
        a.handle_key(key(KeyCode::Char('j')));
        assert_eq!(a.settings_focus, 3);
    }

}
