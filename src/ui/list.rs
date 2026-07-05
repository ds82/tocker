use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::{App, Mode, Section};
use crate::docker::types::ContainerState;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    match app.section {
        Section::Containers => render_containers(f, area, app),
        other => render_placeholder(f, area, other),
    }
}

fn render_containers(f: &mut Frame, area: Rect, app: &App) {
    let visible = app.visible_containers();

    let header = Row::new(vec![
        Cell::from("NAME").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("IMAGE").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("STATUS").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("PORTS").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(Color::DarkGray))
    .height(1);

    let rows: Vec<Row> = visible
        .iter()
        .map(|c| {
            let state_style = match c.state {
                ContainerState::Running => Style::default().fg(Color::Green),
                ContainerState::Paused | ContainerState::Restarting => {
                    Style::default().fg(Color::Yellow)
                }
                _ => Style::default().fg(Color::Red),
            };

            Row::new(vec![
                Cell::from(c.name.as_str()),
                Cell::from(c.image.as_str()),
                Cell::from(Span::styled(c.status_text.as_str(), state_style)),
                Cell::from(c.ports.as_str()),
            ])
        })
        .collect();

    let title = if let Mode::Filter(ref q) = app.mode {
        if !q.is_empty() {
            format!(" Containers ({}/{}) ", visible.len(), app.containers.len())
        } else {
            format!(" Containers ({}) ", app.containers.len())
        }
    } else {
        format!(" Containers ({}) ", app.containers.len())
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(24),
            Constraint::Fill(1),
            Constraint::Length(20),
            Constraint::Length(22),
        ],
    )
    .header(header)
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(title),
    );

    let mut state = TableState::default();
    state.select(if visible.is_empty() { None } else { Some(app.selected) });

    f.render_stateful_widget(table, area, &mut state);
}

fn render_placeholder(f: &mut Frame, area: Rect, section: Section) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(format!(" {} ", section.label()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let msg = Line::from(vec![
        Span::styled(
            format!("{} — coming in Phase 2", section.label()),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    // Vertically centre the message
    if inner.height > 2 {
        let y_offset = inner.height / 2;
        let msg_area = Rect::new(inner.x, inner.y + y_offset, inner.width, 1);
        f.render_widget(
            Paragraph::new(msg).alignment(ratatui::layout::Alignment::Center),
            msg_area,
        );
    }
}
