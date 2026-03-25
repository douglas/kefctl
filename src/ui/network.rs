//! Network panel: connection status + discovered speakers.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::Paragraph,
};

use crate::app::{App, ConnectionState, Focus};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let focused = app.focus == Focus::Main;

    let block = theme.block(" Network ", focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(inner);

    // Connection status
    let (status_text, status_color) = match app.connection {
        ConnectionState::Connected => ("● Connected", theme.status_ok),
        ConnectionState::Disconnected => ("○ Disconnected", theme.status_error),
    };
    let conn_info = format!(
        "  Status: {}\n  IP: {}\n  Name: {}",
        status_text, app.speaker.ip, app.speaker.name
    );
    frame.render_widget(
        Paragraph::new(conn_info).style(Style::default().fg(status_color)),
        chunks[0],
    );

    // Discovered speakers list
    if app.network_speakers.is_empty() {
        frame.render_widget(
            Paragraph::new("  No speakers discovered.")
                .style(Style::default().fg(theme.fg_dim)),
            chunks[1],
        );
    } else {
        let rows = Layout::vertical(
            vec![Constraint::Length(1); app.network_speakers.len() + 1],
        )
        .split(chunks[1]);

        frame.render_widget(
            Paragraph::new("  Discovered speakers:")
                .style(Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
            rows[0],
        );

        for (i, speaker) in app.network_speakers.iter().enumerate() {
            if i + 1 >= rows.len() {
                break;
            }
            let text = format!("    {} — {}:{}", speaker.name, speaker.ip, speaker.port);
            frame.render_widget(
                Paragraph::new(text).style(Style::default().fg(theme.fg)),
                rows[i + 1],
            );
        }
    }
}
