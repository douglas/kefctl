//! Network panel: connection status + discovered speakers.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{List, ListItem, Paragraph},
};

use crate::app::{App, ConnectionState, Focus};

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let focused = app.focus == Focus::Main;

    let block = theme.block(" Speakers ", focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(inner);

    // Connection status
    let (status_text, status_color) = match app.connection {
        ConnectionState::Connected => ("● Connected", theme.status_ok),
        ConnectionState::Connecting => ("◌ Connecting", theme.status_warn),
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
        let text = if app.discovery_in_progress {
            "  Discovering speakers..."
        } else {
            "  No speakers found. Press r to refresh."
        };
        frame.render_widget(
            Paragraph::new(text).style(Style::default().fg(theme.fg_dim)),
            chunks[1],
        );
    } else {
        let heading = if app.discovery_in_progress {
            "  Available speakers (refreshing...):"
        } else {
            "  Available speakers:"
        };
        let list_area =
            Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(chunks[1]);
        frame.render_widget(
            Paragraph::new(heading)
                .style(Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
            list_area[0],
        );

        let items = app.network_speakers.iter().map(|speaker| {
            let marker = if app.switching_to == Some(speaker.ip) {
                "◌"
            } else if app.speaker.ip == speaker.ip {
                "●"
            } else {
                "○"
            };
            let style = if app.speaker.ip == speaker.ip {
                Style::default().fg(theme.status_ok)
            } else {
                Style::default().fg(theme.fg)
            };
            ListItem::new(format!(
                "  {marker} {}  {}:{}",
                speaker.name, speaker.ip, speaker.port
            ))
            .style(style)
        });
        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ");
        frame.render_stateful_widget(list, list_area[1], &mut app.network_list_state);
    }
}
