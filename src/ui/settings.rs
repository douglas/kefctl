//! Settings editor panel: cable mode, standby, max volume, LED, startup tone.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::Paragraph,
};

use crate::app::{App, Focus};

const SETTINGS_ROWS: usize = 6;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let focused = app.focus == Focus::Main;

    let block = theme.block(" Settings ", focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let s = &app.speaker;
    let focus = app.settings_focus;

    let rows =
        Layout::vertical(vec![Constraint::Length(1); SETTINGS_ROWS.max(inner.height as usize)])
            .split(inner);

    let items = [
        ("Cable Mode", s.cable_mode.display_name().to_string()),
        ("Standby", s.standby_mode.display_name().to_string()),
        ("Max Volume", format!("{}", s.max_volume)),
        (
            "Front LED",
            if s.front_led { "ON" } else { "OFF" }.to_string(),
        ),
        (
            "Startup Tone",
            if s.startup_tone { "ON" } else { "OFF" }.to_string(),
        ),
        (
            "",
            "◂/▸ cycle   Enter confirm   Esc back".to_string(),
        ),
    ];

    for (i, (label, value)) in items.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let style = if i == focus {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else if label.is_empty() {
            Style::default().fg(theme.fg_dim)
        } else {
            Style::default().fg(theme.fg)
        };

        let marker = if i == focus && !label.is_empty() {
            "▸ "
        } else {
            "  "
        };
        let arrows = if i == focus && !label.is_empty() {
            "  ◂ ▸"
        } else {
            ""
        };
        let text = if label.is_empty() {
            format!("  {value}")
        } else {
            format!("{marker}{label:<16} {value}{arrows}")
        };
        frame.render_widget(Paragraph::new(text).style(style), rows[i]);
    }
}
