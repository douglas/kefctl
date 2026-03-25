//! EQ/DSP panel — adjustable EQ settings via `kef:eqProfile/v2`.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::Paragraph,
};

use crate::app::{App, Focus};

const EQ_ROWS: usize = 10;

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

    let sub_info = if eq.subwoofer_out {
        format!(
            "ON  gain {:+.0} dB  pol {}  LP {} Hz",
            eq.subwoofer_gain, eq.subwoofer_polarity, eq.sub_out_lp_freq
        )
    } else {
        "OFF".to_string()
    };

    let items = [
        ("Treble", format!("{:+.1} dB", eq.treble_amount)),
        (
            "Bass Extension",
            eq.bass_extension.display_name().to_string(),
        ),
        (
            "Desk Mode",
            if eq.desk_mode {
                format!("ON  {:+.1} dB", eq.desk_mode_setting)
            } else {
                "OFF".to_string()
            },
        ),
        (
            "Wall Mode",
            if eq.wall_mode {
                format!("ON  {:+.1} dB", eq.wall_mode_setting)
            } else {
                "OFF".to_string()
            },
        ),
        ("Sub Out", sub_info),
        (
            "Phase Correction",
            if eq.phase_correction { "ON" } else { "OFF" }.to_string(),
        ),
        ("Balance", format!("{}", eq.balance)),
        ("", String::new()),
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
            String::new()
        } else {
            format!("{marker}{label:<18} {value}{arrows}")
        };
        frame.render_widget(Paragraph::new(text).style(style), rows[i]);
    }

    // Hint text below the rows
    let hint_row = items.len();
    if hint_row < rows.len() {
        frame.render_widget(
            Paragraph::new(format!("  {}", super::HINT_CYCLE))
                .style(Style::default().fg(theme.fg_dim)),
            rows[hint_row],
        );
    }
}
