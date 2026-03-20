mod sidebar;
pub mod status;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{App, ConnectionState, Panel};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::horizontal([Constraint::Length(17), Constraint::Min(1)])
        .split(frame.area());

    sidebar::draw(frame, app, chunks[0]);

    match app.panel {
        Panel::Status => status::draw(frame, app, chunks[1]),
        Panel::Source => draw_placeholder(frame, chunks[1], "Source", "Source selector — Phase 7"),
        Panel::Eq => draw_placeholder(frame, chunks[1], "EQ / DSP", "EQ controls — Phase 7"),
        Panel::Settings => {
            draw_placeholder(frame, chunks[1], "Settings", "Settings panel — Phase 7")
        }
        Panel::Network => {
            draw_placeholder(frame, chunks[1], "Network", "Network panel — Phase 7")
        }
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

fn draw_placeholder(frame: &mut Frame, area: ratatui::layout::Rect, title: &str, msg: &str) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let content = Paragraph::new(msg)
        .style(Style::default().fg(Color::DarkGray))
        .block(block);
    frame.render_widget(content, area);
}
