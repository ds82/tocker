use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App, scroll: usize) {
    let lines_data = app.inspect_lines();
    if lines_data.is_empty() {
        return;
    }

    let key_width = lines_data.iter().map(|(k, _)| k.len()).max().unwrap_or(8) + 2;
    let content_width = lines_data
        .iter()
        .map(|(_, v)| v.len())
        .max()
        .unwrap_or(20);
    let width = (key_width + content_width + 4).max(40).min(area.width as usize - 4) as u16;
    let height = (lines_data.len() + 2).min(area.height as usize - 4) as u16;

    let popup = centered_rect(width, height, area);

    let lines: Vec<Line> = lines_data
        .into_iter()
        .map(|(k, v)| {
            Line::from(vec![
                Span::styled(
                    format!("  {k:<key_width$}"),
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
                ),
                Span::raw(v),
            ])
        })
        .collect();

    let section_label = app.section.label();

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(format!(" Inspect · {section_label}  K or Esc close ")),
            )
            .scroll((scroll as u16, 0)),
        popup,
    );
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
