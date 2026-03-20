mod eq;
mod network;
mod sidebar;
mod settings;
mod source;
pub mod status;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::Paragraph,
};

use crate::app::{App, ConnectionState, Panel};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::horizontal([Constraint::Length(17), Constraint::Min(1)])
        .split(frame.area());

    sidebar::draw(frame, app, chunks[0]);

    match app.panel {
        Panel::Status => status::draw(frame, app, chunks[1]),
        Panel::Source => source::draw(frame, app, chunks[1]),
        Panel::Eq => eq::draw(frame, app, chunks[1]),
        Panel::Settings => settings::draw(frame, app, chunks[1]),
        Panel::Network => network::draw(frame, app, chunks[1]),
    }

    // Connection status in bottom-right if disconnected
    if app.connection == ConnectionState::Disconnected && !app.demo {
        let status = Paragraph::new("⚠ Disconnected")
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        let area = frame.area();
        let status_area = ratatui::layout::Rect {
            x: area.width.saturating_sub(16),
            y: area.height.saturating_sub(1),
            width: 16,
            height: 1,
        };
        frame.render_widget(status, status_area);
    }

    // Notification overlay
    if let Some(ref msg) = app.notification {
        let notif = Paragraph::new(Text::raw(msg.as_str()))
            .style(Style::default().fg(Color::Yellow));
        let area = frame.area();
        let notif_area = ratatui::layout::Rect {
            x: 1,
            y: area.height.saturating_sub(1),
            width: area.width.saturating_sub(2).min(msg.len() as u16 + 2),
            height: 1,
        };
        frame.render_widget(notif, notif_area);
    }
}
