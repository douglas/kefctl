use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::Paragraph,
};

use crate::app::{App, Focus};

const EQ_ROWS: usize = 8;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let focused = app.focus == Focus::Main;

    let block = theme.block(" EQ / DSP ", focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let eq = &app.speaker.eq_profile;
    let focus = app.eq_focus;

    let rows = Layout::vertical(
        vec![Constraint::Length(1); EQ_ROWS.max(inner.height as usize)],
    )
    .split(inner);

    let items = [
        ("Profile", eq.name.clone()),
        ("Treble", format!("{:+.1} dB", eq.treble)),
        ("Bass Extension", eq.bass_extension.display_name().to_string()),
        (
            "Desk Mode",
            if eq.desk_mode {
                format!("ON  {:+.1} dB", eq.desk_db)
            } else {
                "OFF".to_string()
            },
        ),
        (
            "Wall Mode",
            if eq.wall_mode {
                format!("ON  {:+.1} dB", eq.wall_db)
            } else {
                "OFF".to_string()
            },
        ),
        (
            "Sub Out",
            if eq.sub_out {
                format!(
                    "ON  gain {:+.1} dB  pol {}  xover {} Hz",
                    eq.sub_gain,
                    eq.sub_polarity.display_name(),
                    eq.sub_crossover
                )
            } else {
                "OFF".to_string()
            },
        ),
        (
            "Phase Correction",
            if eq.phase_correction { "ON" } else { "OFF" }.to_string(),
        ),
        (
            "",
            "◂/▸ adjust   Enter confirm   Esc back".to_string(),
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
        let text = if label.is_empty() {
            format!("  {value}")
        } else {
            format!("{marker}{label:<18} {value}")
        };
        frame.render_widget(Paragraph::new(text).style(style), rows[i]);
    }
}
