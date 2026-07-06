use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App, scroll: usize, follow: bool) {
    let chunks = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    let container_name = app
        .selected_container()
        .map(|c| c.name.as_str())
        .unwrap_or("unknown");

    let follow_indicator = if follow {
        Span::styled(" [follow] ", Style::default().fg(Color::Green))
    } else {
        Span::styled(" [paused] ", Style::default().fg(Color::Yellow))
    };

    let title = Line::from(vec![
        Span::raw(format!(" logs: {container_name}")),
        follow_indicator,
    ]);

    let view_height = chunks[0].height as usize;
    let total = app.log_lines.len();

    // Compute the window of lines to display
    let start = if total <= view_height {
        0
    } else {
        scroll
            .saturating_sub(view_height.saturating_sub(1))
            .min(total - view_height)
    };
    let end = (start + view_height).min(total);

    let lines: Vec<Line> = app.log_lines[start..end]
        .iter()
        .map(|l| Line::raw(l.as_str()))
        .collect();

    let log_block = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(title),
    );

    f.render_widget(log_block, chunks[0]);

    // Status bar
    let hints = Span::raw("  j/k scroll  f follow  G tail  q close");
    let status_line = Line::from(vec![
        Span::styled(
            " [log] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        hints,
    ]);
    let status = Paragraph::new(status_line).style(Style::default().bg(Color::DarkGray));
    f.render_widget(status, chunks[1]);
}
