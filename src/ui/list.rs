use ratatui::{
    layout::{Alignment, Constraint, Rect},
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
        Section::Images => render_images(f, area, app),
        Section::Volumes => render_volumes(f, area, app),
        Section::Networks => render_networks(f, area, app),
    }
}

// ── Containers ────────────────────────────────────────────────────────────────

fn render_containers(f: &mut Frame, area: Rect, app: &App) {
    let visible = app.visible_containers();
    let visual_range = app.visual_range();
    let table_selected = app.visual_cursor().unwrap_or(app.selected);

    let header = bold_header(&["NAME", "IMAGE", "STATUS", "PORTS"]);

    let rows: Vec<Row> = visible
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let state_style = match c.state {
                ContainerState::Running => Style::default().fg(Color::Green),
                ContainerState::Paused | ContainerState::Restarting => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::Red),
            };
            let row = Row::new(vec![
                Cell::from(c.name.as_str()),
                Cell::from(c.image.as_str()),
                Cell::from(Span::styled(c.status_text.as_str(), state_style)),
                Cell::from(c.ports.as_str()),
            ]);
            // Apply visual range background for rows inside the selection
            if let Some((lo, hi)) = visual_range {
                if i >= lo && i <= hi {
                    return row.style(Style::default().bg(Color::Blue));
                }
            }
            row
        })
        .collect();

    let title = section_title("Containers", visible.len(), app.containers.len(), &app.mode);

    let hl_style = if visual_range.is_some() {
        // In visual mode: cursor row stands out from the range with bright yellow
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::Blue)
    } else {
        highlight_style()
    };

    let table = Table::new(
        rows,
        [Constraint::Length(24), Constraint::Fill(1), Constraint::Length(20), Constraint::Length(22)],
    )
    .header(header)
    .row_highlight_style(hl_style)
    .block(panel_block(title));

    render_table(f, area, table, table_selected, visible.is_empty());
}

// ── Images ────────────────────────────────────────────────────────────────────

fn render_images(f: &mut Frame, area: Rect, app: &App) {
    let visible = app.visible_images();

    let header = bold_header(&["REPOSITORY", "TAG", "ID", "SIZE", "CREATED"]);

    let rows: Vec<Row> = visible
        .iter()
        .map(|img| {
            Row::new(vec![
                Cell::from(img.repository.as_str()),
                Cell::from(img.tag.as_str()),
                Cell::from(img.id.as_str()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(img.size.as_str()),
                Cell::from(img.created.as_str()).style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let title = section_title("Images", visible.len(), app.images.len(), &app.mode);

    let table = Table::new(
        rows,
        [
            Constraint::Fill(1),
            Constraint::Length(16),
            Constraint::Length(14),
            Constraint::Length(9),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .row_highlight_style(highlight_style())
    .block(panel_block(title));

    render_table(f, area, table, app.selected, visible.is_empty());
}

// ── Volumes ───────────────────────────────────────────────────────────────────

fn render_volumes(f: &mut Frame, area: Rect, app: &App) {
    let visible = app.visible_volumes();

    let header = bold_header(&["NAME", "DRIVER", "SCOPE", "MOUNTPOINT"]);

    let rows: Vec<Row> = visible
        .iter()
        .map(|v| {
            Row::new(vec![
                Cell::from(v.name.as_str()),
                Cell::from(v.driver.as_str()),
                Cell::from(v.scope.as_str()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(v.mountpoint.as_str()).style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let title = section_title("Volumes", visible.len(), app.volumes.len(), &app.mode);

    let table = Table::new(
        rows,
        [
            Constraint::Length(32),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .row_highlight_style(highlight_style())
    .block(panel_block(title));

    render_table(f, area, table, app.selected, visible.is_empty());
}

// ── Networks ──────────────────────────────────────────────────────────────────

fn render_networks(f: &mut Frame, area: Rect, app: &App) {
    let visible = app.visible_networks();

    let header = bold_header(&["NAME", "DRIVER", "SCOPE", "SUBNET"]);

    let rows: Vec<Row> = visible
        .iter()
        .map(|n| {
            Row::new(vec![
                Cell::from(n.name.as_str()),
                Cell::from(n.driver.as_str()),
                Cell::from(n.scope.as_str()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(n.subnet.as_str()),
            ])
        })
        .collect();

    let title = section_title("Networks", visible.len(), app.networks.len(), &app.mode);

    let table = Table::new(
        rows,
        [
            Constraint::Length(24),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .row_highlight_style(highlight_style())
    .block(panel_block(title));

    render_table(f, area, table, app.selected, visible.is_empty());
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn bold_header<'a>(cols: &[&'a str]) -> Row<'a> {
    Row::new(
        cols.iter()
            .map(|&c| Cell::from(c).style(Style::default().add_modifier(Modifier::BOLD)))
            .collect::<Vec<_>>(),
    )
    .style(Style::default().fg(Color::DarkGray))
    .height(1)
}

fn highlight_style() -> Style {
    Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
}

fn panel_block(title: String) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(title)
}

fn section_title(name: &str, visible: usize, total: usize, mode: &Mode) -> String {
    if let Mode::Filter(ref q) = mode {
        if !q.is_empty() {
            return format!(" {name} ({visible}/{total}) ");
        }
    }
    format!(" {name} ({total}) ")
}

fn render_table(f: &mut Frame, area: Rect, table: Table, selected: usize, empty: bool) {
    let mut state = TableState::default();
    state.select(if empty { None } else { Some(selected) });
    f.render_stateful_widget(table, area, &mut state);
}

#[allow(dead_code)]
fn render_empty_hint(f: &mut Frame, area: Rect, section: Section) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(format!(" {} ", section.label()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height > 2 {
        let msg = Line::from(Span::styled(
            format!("no {} found", section.label().to_lowercase()),
            Style::default().fg(Color::DarkGray),
        ));
        let msg_area = Rect::new(inner.x, inner.y + inner.height / 2, inner.width, 1);
        f.render_widget(Paragraph::new(msg).alignment(Alignment::Center), msg_area);
    }
}
