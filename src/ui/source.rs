use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::App;
use crate::kef_api::types::Source;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
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
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("{marker}{}", s.display_name())).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Source ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        )
        .highlight_symbol("▸");

    frame.render_stateful_widget(list, area, &mut app.source_list_state);
}
