//! Input source selector panel.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{List, ListItem},
};

use crate::app::{App, Focus};
use crate::kef_api::types::Source;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let focused = app.focus == Focus::Main;

    let items: Vec<ListItem> = Source::ALL
        .iter()
        .map(|s| {
            let marker = if *s == app.speaker.source {
                " ● "
            } else {
                "   "
            };
            let style = if *s == app.speaker.source {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };
            ListItem::new(format!("{marker}{}", s.display_name())).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(theme.block(" Source ", focused))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        )
        .highlight_symbol("▸");

    frame.render_stateful_widget(list, area, &mut app.source_list_state);
}
