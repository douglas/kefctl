use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::{App, Focus, Panel};

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = Panel::ALL
        .iter()
        .map(|p| ListItem::new(format!("  {}", p.label())))
        .collect();

    let border_color = if app.focus == Focus::Sidebar {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(" kefctl ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut app.sidebar_state);
}
