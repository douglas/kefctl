//! Status panel: speaker info and settings summary.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Min(3),
        Constraint::Min(3),
    ])
    .split(area);

    draw_speaker_info(frame, app, chunks[0]);
    draw_settings_summary(frame, app, chunks[1]);
}

fn draw_speaker_info(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let s = &app.speaker;
    let block = theme.section_block(" Speaker Info ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical(vec![Constraint::Length(1); 5]).split(inner);

    let ip_str = s.ip.to_string();

    // Name row — show editable text field when in editing mode
    let name_widget = if app.editing_name {
        let before = &app.name_buf[..app.name_cursor];
        let after = &app.name_buf[app.name_cursor..];
        let line = Line::from(vec![
            Span::styled(format!("  {:<10}", "Name"), Style::default().fg(theme.fg_dim)),
            Span::styled(before, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("▏", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(after, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]);
        Paragraph::new(line)
    } else {
        Paragraph::new(theme.info_row("Name", s.name.as_str()))
    };
    if !rows.is_empty() {
        frame.render_widget(name_widget, rows[0]);
    }

    let other_fields = [
        ("Model", s.model.as_str()),
        ("Firmware", s.firmware.as_str()),
        ("IP", ip_str.as_str()),
        ("MAC", s.mac.as_str()),
    ];
    for (i, (label, value)) in other_fields.iter().enumerate() {
        let row_idx = i + 1;
        if row_idx >= rows.len() {
            break;
        }
        frame.render_widget(Paragraph::new(theme.info_row(label, value)), rows[row_idx]);
    }
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
        frame.render_widget(
            Paragraph::new(theme.info_row(label, value)),
            rows[i],
        );
    }
}
