use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn draw(frame: &mut Frame) {
    let area = centered_rect(60, 28, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Keybindings ")
        .title_bottom(" ? or Esc to close ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let lines = vec![
        heading("Navigation"),
        binding("Tab / Shift+Tab", "Next / previous panel"),
        binding("j / k  ↓ / ↑", "Move down / up"),
        binding("h / ←", "Focus sidebar / back"),
        binding("l / → / Enter", "Focus main panel / select"),
        binding("Esc", "Back to sidebar"),
        Line::raw(""),
        heading("Playback"),
        binding("Space", "Play / pause"),
        binding("n / p", "Next / previous track"),
        binding("f / b", "Seek forward / backward 10s"),
        Line::raw(""),
        heading("Volume"),
        binding("+ / -", "Volume up / down"),
        binding("m", "Toggle mute"),
        Line::raw(""),
        heading("Panels"),
        binding("Source", "j/k navigate, Enter to switch"),
        binding("EQ / DSP", "j/k navigate, h/l adjust values"),
        binding("Settings", "j/k navigate, h/l cycle options"),
        Line::raw(""),
        heading("General"),
        binding("?", "Show this help"),
        binding("q / Ctrl+c", "Quit"),
    ];

    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}

fn heading(text: &str) -> Line<'_> {
    Line::from(Span::styled(
        format!("  {text}"),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
}

fn binding<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("    {key:<20}"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc, Style::default().fg(Color::White)),
    ])
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(area);
    Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
}
