use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

use crate::app::Section;

pub fn render(f: &mut Frame, area: Rect, cursor: usize, current: Section) {
    let width: u16 = 24;
    let height: u16 = Section::ALL.len() as u16 + 2;

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width.min(area.width), height.min(area.height));

    f.render_widget(Clear, popup);

    let items: Vec<ListItem> = Section::ALL
        .iter()
        .enumerate()
        .map(|(i, &section)| {
            let active = section == current;
            let highlighted = i == cursor;

            let bullet = if active { "● " } else { "  " };
            let label = format!(" {bullet}{}", section.label());

            let style = if highlighted {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if active {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(label, style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" Go to "),
    );

    f.render_widget(list, popup);
}
