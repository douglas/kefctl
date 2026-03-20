mod eq;
mod help;
mod network;
mod sidebar;
mod settings;
mod source;
pub mod status;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use crate::app::{App, ConnectionState, Panel};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let outer = Layout::vertical([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let main_area = outer[0];
    let footer_area = outer[1];

    let chunks = Layout::horizontal([Constraint::Length(17), Constraint::Min(1)])
        .split(main_area);

    sidebar::draw(frame, app, chunks[0]);

    match app.panel {
        Panel::Status => status::draw(frame, app, chunks[1]),
        Panel::Source => source::draw(frame, app, chunks[1]),
        Panel::Eq => eq::draw(frame, app, chunks[1]),
        Panel::Settings => settings::draw(frame, app, chunks[1]),
        Panel::Network => network::draw(frame, app, chunks[1]),
    }

    // Footer status bar
    draw_footer(frame, app, footer_area);

    // Help overlay (on top of everything)
    if app.show_help {
        help::draw(frame);
    }

    // Notification overlay (on top of footer)
    if let Some(ref msg) = app.notification {
        let notif = Paragraph::new(Text::raw(msg.as_str()))
            .style(Style::default().fg(Color::Yellow));
        let notif_area = ratatui::layout::Rect {
            x: 1,
            y: footer_area.y,
            width: footer_area
                .width
                .saturating_sub(2)
                .min(msg.len() as u16 + 2),
            height: 1,
        };
        frame.render_widget(notif, notif_area);
    }
}

fn draw_footer(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let (conn_text, conn_color) = match app.connection {
        ConnectionState::Connected => ("●", Color::Green),
        ConnectionState::Connecting => ("◌", Color::Yellow),
        ConnectionState::Disconnected if app.demo => ("◆ demo", Color::Magenta),
        ConnectionState::Disconnected => ("○ disconnected", Color::Red),
    };

    let line = Line::from(vec![
        Span::styled(
            " ? ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Help  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            conn_text,
            Style::default().fg(conn_color),
        ),
        Span::styled(
            format!(" {}  ", app.speaker.name),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            app.panel.label(),
            Style::default().fg(Color::Cyan),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
