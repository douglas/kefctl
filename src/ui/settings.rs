//! Settings editor panel: standby, max volume, LED, startup tone, cable mode (display-only).

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::Paragraph,
};

use crate::app::{App, Focus};

const SETTINGS_ROWS: usize = 7;

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

    // Adjustable rows (focus 0-3)
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

    // Display-only info below adjustable rows
    let info_start = adjustable.len();
    if info_start < rows.len() {
        frame.render_widget(Paragraph::new(""), rows[info_start]);
    }
    if info_start + 1 < rows.len() {
        let cable = format!("  Cable Mode       {}", s.cable_mode.display_name());
        frame.render_widget(
            Paragraph::new(cable).style(Style::default().fg(theme.fg_dim)),
            rows[info_start + 1],
        );
    }
    if info_start + 2 < rows.len() {
        frame.render_widget(
            Paragraph::new(format!("  {}", super::HINT_CYCLE))
                .style(Style::default().fg(theme.fg_dim)),
            rows[info_start + 2],
        );
    }
}
