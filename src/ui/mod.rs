//! Top-level UI layout: sidebar + main panel, footer, notification overlay.

mod eq;
mod help;
mod network;
mod sidebar;
mod settings;
mod source;
pub mod status;
pub mod theme;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, ConnectionState, Panel};

pub(crate) const HINT_ADJUST: &str = "◂/▸ adjust   Enter confirm   Esc back";
pub(crate) const HINT_CYCLE: &str = "◂/▸ cycle   Enter confirm   Esc back";

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
        help::draw(frame, app);
    }

    // Notification overlay (on top of footer)
    if let Some(ref msg) = app.notification {
        let notif = Paragraph::new(msg.as_str())
            .style(Style::default().fg(app.theme.status_warn));
        #[allow(clippy::cast_possible_truncation)]
        let msg_width = msg.len() as u16 + 2;
        let notif_area = ratatui::layout::Rect {
            x: 1,
            y: footer_area.y,
            width: footer_area.width.saturating_sub(2).min(msg_width),
            height: 1,
        };
        frame.render_widget(notif, notif_area);
    }
}

fn draw_footer(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let theme = &app.theme;

    let (conn_text, conn_color) = match app.connection {
        ConnectionState::Connected => ("●", theme.status_ok),
        ConnectionState::Disconnected if app.demo => ("◆ demo", theme.status_demo),
        ConnectionState::Disconnected => ("○ disconnected", theme.status_error),
    };

    let mut spans = vec![
        Span::styled(
            " ? ",
            Style::default()
                .fg(theme.fg)
                .bg(theme.badge_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Help", Style::default().fg(theme.fg_dim)),
    ];

    // Keybinding hints — responsive based on width
    let width = area.width as usize;
    let badges: &[(&str, &str)] = &[
        ("Space", "play/pause"),
        ("+/-", "vol"),
        ("m", "mute"),
    ];

    if width > 50 {
        for (key, desc) in badges {
            spans.push(Span::styled(" │ ", Style::default().fg(theme.fg_dim)));
            spans.push(Span::styled(
                format!(" {key} "),
                Style::default()
                    .fg(theme.fg)
                    .bg(theme.badge_bg)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!(" {desc}"),
                Style::default().fg(theme.fg_dim),
            ));
        }
    }

    // Right side: connection + speaker name + panel
    let right_text = format!(
        " {conn_text} {} │ {}",
        app.speaker.name,
        app.panel.label()
    );
    let right_len = right_text.len();
    let left_len: usize = spans.iter().map(Span::width).sum();
    let padding = width.saturating_sub(left_len + right_len);

    spans.push(Span::raw(" ".repeat(padding)));
    spans.push(Span::styled(
        conn_text,
        Style::default().fg(conn_color),
    ));
    spans.push(Span::styled(
        format!(" {} │ ", app.speaker.name),
        Style::default().fg(theme.fg_dim),
    ));
    spans.push(Span::styled(
        app.panel.label(),
        Style::default().fg(theme.accent),
    ));

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use crate::app::{App, Panel};

    fn render_app(app: &mut App, width: u16, height: u16) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| super::draw(frame, app)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn buffer_text(buf: &ratatui::buffer::Buffer) -> String {
        let mut text = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                text.push_str(buf.cell((x, y)).unwrap().symbol());
            }
            text.push('\n');
        }
        text
    }

    #[test]
    fn status_panel_renders() {
        let mut app = App::new_demo();
        app.select_panel(Panel::Status);
        let buf = render_app(&mut app, 80, 24);
        let text = buffer_text(&buf);
        assert!(text.contains("Living Room LSX II"));
        assert!(text.contains("Now Playing"));
    }

    #[test]
    fn source_panel_renders() {
        let mut app = App::new_demo();
        app.select_panel(Panel::Source);
        let buf = render_app(&mut app, 80, 24);
        let text = buffer_text(&buf);
        assert!(text.contains("Wi-Fi"));
        assert!(text.contains("Bluetooth"));
    }

    #[test]
    fn eq_panel_renders() {
        let mut app = App::new_demo();
        app.select_panel(Panel::Eq);
        let buf = render_app(&mut app, 80, 24);
        let text = buffer_text(&buf);
        assert!(text.contains("Treble"));
        assert!(text.contains("Bass Extension"));
    }

    #[test]
    fn settings_panel_renders() {
        let mut app = App::new_demo();
        app.select_panel(Panel::Settings);
        let buf = render_app(&mut app, 80, 24);
        let text = buffer_text(&buf);
        assert!(text.contains("Cable Mode"));
        assert!(text.contains("Standby"));
    }

    #[test]
    fn network_panel_renders() {
        let mut app = App::new_demo();
        app.select_panel(Panel::Network);
        let buf = render_app(&mut app, 80, 24);
        let text = buffer_text(&buf);
        assert!(text.contains("Connected") || text.contains("Disconnected"));
    }

    #[test]
    fn help_overlay_renders() {
        let mut app = App::new_demo();
        app.show_help = true;
        let buf = render_app(&mut app, 80, 30);
        let text = buffer_text(&buf);
        assert!(text.contains("Keybindings"));
    }

    #[test]
    fn footer_renders_demo_badge() {
        let mut app = App::new_demo();
        // Demo badge shows when disconnected in demo mode
        app.connection = crate::app::ConnectionState::Disconnected;
        let buf = render_app(&mut app, 80, 24);
        let text = buffer_text(&buf);
        assert!(text.contains("demo"));
    }

    #[test]
    fn small_terminal_no_panic() {
        let mut app = App::new_demo();
        // Should not panic at small size
        let _buf = render_app(&mut app, 40, 12);
    }

    // -- insta snapshot tests --

    #[test]
    fn snapshot_status_panel() {
        let mut app = App::new_demo();
        app.select_panel(Panel::Status);
        let buf = render_app(&mut app, 80, 24);
        insta::assert_snapshot!(buffer_text(&buf));
    }

    #[test]
    fn snapshot_source_panel() {
        let mut app = App::new_demo();
        app.select_panel(Panel::Source);
        let buf = render_app(&mut app, 80, 24);
        insta::assert_snapshot!(buffer_text(&buf));
    }

    #[test]
    fn snapshot_eq_panel() {
        let mut app = App::new_demo();
        app.select_panel(Panel::Eq);
        let buf = render_app(&mut app, 80, 24);
        insta::assert_snapshot!(buffer_text(&buf));
    }

    #[test]
    fn snapshot_settings_panel() {
        let mut app = App::new_demo();
        app.select_panel(Panel::Settings);
        let buf = render_app(&mut app, 80, 24);
        insta::assert_snapshot!(buffer_text(&buf));
    }

    #[test]
    fn snapshot_network_panel() {
        let mut app = App::new_demo();
        app.select_panel(Panel::Network);
        let buf = render_app(&mut app, 80, 24);
        insta::assert_snapshot!(buffer_text(&buf));
    }

    #[test]
    fn snapshot_help_overlay() {
        let mut app = App::new_demo();
        app.show_help = true;
        let buf = render_app(&mut app, 80, 30);
        insta::assert_snapshot!(buffer_text(&buf));
    }
}
