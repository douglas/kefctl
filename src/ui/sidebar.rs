//! Sidebar panel navigation list.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{List, ListItem},
};

use crate::app::{App, Focus, Panel};

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let focused = app.focus == Focus::Sidebar;

    let items: Vec<ListItem> = Panel::ALL
        .iter()
        .map(|p| ListItem::new(format!("  {}", p.label())))
        .collect();

    let list = List::new(items)
        .block(theme.block(" kefctl ", focused))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut app.sidebar_state);
}
