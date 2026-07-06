use std::collections::BTreeMap;

use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::{App, Section};
use crate::docker::types::{Container, ContainerState};

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

    let groups = build_container_groups(&visible);

    // Compute which display row (including headers) corresponds to table_selected
    let mut display_selected = table_selected;
    let mut containers_seen: usize = 0;
    for group in &groups {
        if group.project.is_some() && containers_seen <= table_selected {
            display_selected += 1;
        }
        containers_seen += group.containers.len();
    }

    let header = bold_header(&["NAME", "IMAGE", "STATUS", "PORTS"]);

    let mut rows: Vec<Row> = Vec::new();
    let mut container_idx: usize = 0;

    for group in &groups {
        if let Some(ref proj) = group.project {
            rows.push(
                Row::new(vec![
                    Cell::from(format!(" ▸ {proj}"))
                        .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])
                .style(Style::default().bg(Color::Reset)),
            );
        }

        for c in &group.containers {
            let state_style = match c.state {
                ContainerState::Running => Style::default().fg(app.theme.status_running),
                ContainerState::Paused | ContainerState::Restarting => {
                    Style::default().fg(app.theme.status_paused)
                }
                _ => Style::default().fg(app.theme.status_exited),
            };
            let row = Row::new(vec![
                Cell::from(c.name.as_str()),
                Cell::from(c.image.as_str()),
                Cell::from(Span::styled(c.status_text.as_str(), state_style)),
                Cell::from(c.ports.as_str()),
            ]);
            let row = if let Some((lo, hi)) = visual_range {
                if container_idx >= lo && container_idx <= hi {
                    row.style(Style::default().bg(Color::Blue))
                } else {
                    row
                }
            } else {
                row
            };
            rows.push(row);
            container_idx += 1;
        }
    }

    let title = section_title("Containers", visible.len(), app.containers.len(), app);

    let hl_style = if visual_range.is_some() {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::Blue)
    } else {
        Style::default().bg(app.theme.selected_bg).add_modifier(Modifier::BOLD)
    };

    let table = Table::new(
        rows,
        [Constraint::Length(24), Constraint::Fill(1), Constraint::Length(20), Constraint::Length(22)],
    )
    .header(header)
    .row_highlight_style(hl_style)
    .block(panel_block(title));

    render_table(f, area, table, display_selected, visible.is_empty());
}

struct ContainerGroup<'a> {
    project: Option<String>,
    containers: Vec<&'a Container>,
}

fn build_container_groups<'a>(containers: &[&'a Container]) -> Vec<ContainerGroup<'a>> {
    let mut by_project: BTreeMap<String, Vec<&Container>> = BTreeMap::new();
    let mut ungrouped: Vec<&Container> = Vec::new();

    for c in containers {
        match &c.compose_project {
            Some(proj) => by_project.entry(proj.clone()).or_default().push(c),
            None => ungrouped.push(c),
        }
    }

    let mut groups: Vec<ContainerGroup> = by_project
        .into_iter()
        .map(|(project, containers)| ContainerGroup { project: Some(project), containers })
        .collect();

    if !ungrouped.is_empty() {
        groups.push(ContainerGroup { project: None, containers: ungrouped });
    }

    groups
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

    let title = section_title("Images", visible.len(), app.images.len(), app);

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
    .row_highlight_style(highlight_style(app))
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

    let title = section_title("Volumes", visible.len(), app.volumes.len(), app);

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
    .row_highlight_style(highlight_style(app))
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

    let title = section_title("Networks", visible.len(), app.networks.len(), app);

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
    .row_highlight_style(highlight_style(app))
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

fn highlight_style(app: &App) -> Style {
    Style::default().bg(app.theme.selected_bg).add_modifier(Modifier::BOLD)
}

fn panel_block(title: String) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(title)
}

fn section_title(name: &str, visible: usize, total: usize, app: &App) -> String {
    if !app.filter.is_empty() {
        format!(" {name} ({visible}/{total}) ")
    } else {
        format!(" {name} ({total}) ")
    }
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
