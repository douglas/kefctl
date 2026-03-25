//! Floating keybindings overlay (press `?`).

use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;
use crate::ui::theme::Theme;

pub fn draw(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = centered_rect(60, 22, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Keybindings ")
        .title_bottom(" ? or Esc to close ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));

    let lines = vec![
        heading("Navigation", theme),
        binding("Tab / Shift+Tab", "Next / previous panel", theme),
        binding("j / k  ↓ / ↑", "Move down / up", theme),
        binding("h / ←", "Focus sidebar / back", theme),
        binding("l / → / Enter", "Focus main panel / select", theme),
        binding("Esc", "Back to sidebar", theme),
        Line::raw(""),
        heading("Volume", theme),
        binding("+ / -", "Volume up / down", theme),
        binding("m", "Toggle mute", theme),
        Line::raw(""),
        heading("Panels", theme),
        binding("Status", "e to edit speaker name", theme),
        binding("Source", "j/k navigate, Enter to switch", theme),
        binding("EQ / DSP", "j/k navigate, ◂/▸ adjust values", theme),
        binding("Settings", "j/k navigate, h/l cycle options", theme),
        Line::raw(""),
        heading("General", theme),
        binding("?", "Show this help", theme),
        binding("q / Ctrl+c", "Quit", theme),
    ];

    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}

fn heading<'a>(text: &'a str, theme: &Theme) -> Line<'a> {
    Line::from(Span::styled(
        format!("  {text}"),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))
}

fn binding<'a>(key: &'a str, desc: &'a str, theme: &Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("    {key:<20}"),
            Style::default()
                .fg(theme.accent_secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc, Style::default().fg(theme.fg)),
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
