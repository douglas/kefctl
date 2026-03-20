//! Status panel: speaker info, now playing, progress bar.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{LineGauge, Paragraph},
};

use crate::app::App;
use crate::ui::theme::Theme;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Min(3),
        Constraint::Min(3),
        Constraint::Min(8),
    ])
    .split(area);

    draw_speaker_info(frame, app, chunks[0]);
    draw_settings_summary(frame, app, chunks[1]);
    draw_now_playing(frame, app, chunks[2]);
}

fn draw_speaker_info(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let s = &app.speaker;
    let block = theme.section_block(" Speaker Info ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical(vec![Constraint::Length(1); 5]).split(inner);

    let ip_str = s.ip.to_string();
    let fields = [
        ("Name", s.name.as_str()),
        ("Model", s.model.as_str()),
        ("Firmware", s.firmware.as_str()),
        ("IP", ip_str.as_str()),
        ("MAC", s.mac.as_str()),
    ];

    for (i, (label, value)) in fields.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        frame.render_widget(Paragraph::new(theme.info_row(label, value)), rows[i]);
    }
}

fn draw_now_playing(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let s = &app.speaker;
    let block = theme.section_block(" Now Playing ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Min(1),    // track info (fills space)
        Constraint::Length(1), // progress bar
        Constraint::Length(1), // controls hint
    ])
    .split(inner);

    // Track info — centered vertically in the available space
    let artist = s.artist.as_deref().unwrap_or("—");
    let track = s.track.as_deref().unwrap_or("No track");
    let state_icon = if s.playing { "▶" } else { "⏸" };

    let track_lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                format!("  {state_icon}  "),
                Style::default().fg(theme.accent),
            ),
            Span::styled(
                track,
                Style::default()
                    .fg(theme.fg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            format!("     {artist}"),
            Style::default().fg(theme.fg_dim),
        )),
    ];
    frame.render_widget(Paragraph::new(track_lines), chunks[0]);

    // Progress bar
    let (position, duration) = (s.position.unwrap_or(0), s.duration.unwrap_or(0));
    let ratio = if duration > 0 {
        (f64::from(position) / f64::from(duration)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let time_label = format!(
        " {}:{:02} / {}:{:02}",
        position / 60,
        position % 60,
        duration / 60,
        duration % 60
    );
    let gauge = LineGauge::default()
        .label(time_label)
        .ratio(ratio)
        .filled_style(Style::default().fg(theme.progress_filled))
        .unfilled_style(Style::default().fg(theme.progress_empty));
    frame.render_widget(gauge, chunks[1]);

    // Controls hint
    draw_controls_hint(frame, theme, chunks[2]);
}

fn draw_controls_hint(frame: &mut Frame, theme: &Theme, area: Rect) {
    let pairs = [
        ("Space", "play/pause"),
        ("n/p", "next/prev"),
        ("f/b", "seek"),
        ("+/-", "volume"),
    ];

    let mut spans = Vec::new();
    spans.push(Span::raw(" "));
    for (i, (key, desc)) in pairs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(theme.fg_dim)));
        }
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(theme.accent_secondary)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {desc}"),
            Style::default().fg(theme.fg_dim),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_settings_summary(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let s = &app.speaker;

    let block = theme.section_block(" Settings ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mute_str = if s.muted { " [MUTED]" } else { "" };
    let vol_bar_width = 20;
    #[allow(clippy::cast_sign_loss)] // volume and max_volume are always non-negative
    let filled =
        (s.volume as usize * vol_bar_width / s.max_volume.max(1) as usize).min(vol_bar_width);
    let vol_bar: String = "█".repeat(filled) + &"░".repeat(vol_bar_width - filled);

    let fields: Vec<(&str, String)> = vec![
        ("Source", s.source.display_name().to_string()),
        ("Volume", format!("{vol_bar} {}{mute_str}", s.volume)),
        ("Cable", s.cable_mode.display_name().to_string()),
        ("Standby", s.standby_mode.display_name().to_string()),
        (
            "LED",
            format!(
                "{}   Startup tone: {}",
                if s.front_led { "on" } else { "off" },
                if s.startup_tone { "on" } else { "off" }
            ),
        ),
    ];

    let rows = Layout::vertical(
        vec![Constraint::Length(1); fields.len().max(inner.height as usize)],
    )
    .split(inner);

    for (i, (label, value)) in fields.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        frame.render_widget(Paragraph::new(theme.info_row(label, value)), rows[i]);
    }
}
