//! Settings editor panel: standby, max volume, LED, startup tone, cable mode, wake-up source, app analytics.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::Paragraph,
};

use crate::app::{App, Focus};

const SETTINGS_ROWS: usize = 9;

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

    // Adjustable rows (focus 0-6)
    let adjustable = [
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
        ("Cable Mode", s.cable_mode.display_name().to_string()),
        ("Wake-Up Source", s.wake_up_source.display_name().to_string()),
        (
            "App Analytics",
            if s.app_analytics { "ON" } else { "OFF" }.to_string(),
        ),
    ];

    for (i, (label, value)) in adjustable.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let style = if i == focus {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg)
        };
        let marker = if i == focus { "▸ " } else { "  " };
        let arrows = if i == focus { "  ◂ ▸" } else { "" };
        let text = format!("{marker}{label:<16} {value}{arrows}");
        frame.render_widget(Paragraph::new(text).style(style), rows[i]);
    }

    // Hint row below adjustable rows
    let hint_row = adjustable.len() + 1; // +1 for blank separator
    let sep = adjustable.len();
    if sep < rows.len() {
        frame.render_widget(Paragraph::new(""), rows[sep]);
    }
    if hint_row < rows.len() {
        frame.render_widget(
            Paragraph::new(format!("  {}", super::HINT_CYCLE))
                .style(Style::default().fg(theme.fg_dim)),
            rows[hint_row],
        );
    }
}
