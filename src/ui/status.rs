use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, LineGauge, Paragraph},
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let chunks =
        Layout::vertical([Constraint::Length(7), Constraint::Length(6), Constraint::Min(5)])
            .split(area);

    draw_speaker_info(frame, app, chunks[0]);
    draw_now_playing(frame, app, chunks[1]);
    draw_settings_summary(frame, app, chunks[2]);
}

fn draw_speaker_info(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.speaker;
    let text = format!(
        " Name:     {}\n Model:    {}\n Firmware: {}\n IP:       {}\n MAC:      {}",
        s.name, s.model, s.firmware, s.ip, s.mac
    );
    let block = Block::default()
        .title(" Speaker Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let para = Paragraph::new(text).block(block);
    frame.render_widget(para, area);
}

fn draw_now_playing(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.speaker;
    let block = Block::default()
        .title(" Now Playing ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks =
        Layout::vertical([Constraint::Length(2), Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

    // Track info
    let artist = s.artist.as_deref().unwrap_or("—");
    let track = s.track.as_deref().unwrap_or("No track");
    let state_icon = if s.playing { "▶" } else { "⏸" };
    let track_text = format!(" {state_icon} {artist} — {track}");
    frame.render_widget(
        Paragraph::new(track_text).style(Style::default().fg(Color::White)),
        chunks[0],
    );

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
        Paragraph::new(controls)
            .style(Style::default().fg(Color::DarkGray)),
        chunks[2],
    );
}

fn draw_settings_summary(frame: &mut Frame, app: &App, area: Rect) {
    let s = &app.speaker;

    let mute_str = if s.muted { " [MUTED]" } else { "" };
    let vol_bar_width = 20;
    let filled = (s.volume as usize * vol_bar_width / s.max_volume.max(1) as usize)
        .min(vol_bar_width);
    let vol_bar: String = "█".repeat(filled) + &"░".repeat(vol_bar_width - filled);

    let text = format!(
        " Source:  {}\n Volume:  {} {}{}\n Cable:   {}\n Standby: {}\n LED: {}  Startup tone: {}",
        s.source.display_name(),
        vol_bar,
        s.volume,
        mute_str,
        s.cable_mode.display_name(),
        s.standby_mode.display_name(),
        if s.front_led { "on" } else { "off" },
        if s.startup_tone { "on" } else { "off" },
    );

    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let para = Paragraph::new(text).block(block);
    frame.render_widget(para, area);
}
