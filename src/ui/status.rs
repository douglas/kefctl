use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, LineGauge, Paragraph},
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(7),
        Constraint::Min(8),
        Constraint::Length(9),
    ])
    .split(area);

    draw_speaker_info(frame, app, chunks[0]);
    draw_now_playing(frame, app, chunks[1]);
    draw_settings_summary(frame, app, chunks[2]);
}

fn draw_speaker_info(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.speaker;
    let block = Block::default()
        .title(" Speaker Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical(vec![Constraint::Length(1); 5]).split(inner);

    let fields = [
        ("Name", s.name.as_str()),
        ("Model", s.model.as_str()),
        ("Firmware", s.firmware.as_str()),
        ("IP", &s.ip.to_string()),
        ("MAC", s.mac.as_str()),
    ];

    for (i, (label, value)) in fields.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let line = Line::from(vec![
            Span::styled(
                format!("  {label:<10}"),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(*value, Style::default().fg(Color::White)),
        ]);
        frame.render_widget(Paragraph::new(line), rows[i]);
    }
}

fn draw_now_playing(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.speaker;
    let block = Block::default()
        .title(" Now Playing ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

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
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                track,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            format!("     {artist}"),
            Style::default().fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(Paragraph::new(track_lines), chunks[0]);

    // Progress bar
    let (position, duration) = (s.position.unwrap_or(0), s.duration.unwrap_or(0));
    let ratio = if duration > 0 {
        (position as f64 / duration as f64).clamp(0.0, 1.0)
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
        .filled_style(Style::default().fg(Color::Cyan))
        .unfilled_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(gauge, chunks[1]);

    // Controls hint
    let controls = " [Space] play/pause  [n/p] next/prev  [f/b] seek  [+/-] volume";
    frame.render_widget(
        Paragraph::new(controls).style(Style::default().fg(Color::DarkGray)),
        chunks[2],
    );
}

fn draw_settings_summary(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.speaker;

    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mute_str = if s.muted { " [MUTED]" } else { "" };
    let vol_bar_width = 20;
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

    let rows = Layout::vertical(vec![Constraint::Length(1); fields.len().max(inner.height as usize)])
        .split(inner);

    for (i, (label, value)) in fields.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let line = Line::from(vec![
            Span::styled(
                format!("  {label:<10}"),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(value.as_str(), Style::default().fg(Color::White)),
        ]);
        frame.render_widget(Paragraph::new(line), rows[i]);
    }
}
