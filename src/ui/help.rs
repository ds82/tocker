use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, scroll: usize) {
    let lines = help_lines();
    let popup = centered_rect(62, (lines.len() + 2).min(area.height as usize - 2) as u16, area);

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title(" Help  j/k scroll  ? or Esc close "),
            )
            .scroll((scroll as u16, 0)),
        popup,
    );
}

fn key(k: &'static str) -> Span<'static> {
    Span::styled(k, Style::default().fg(Color::Cyan))
}

fn desc(d: &'static str) -> Span<'static> {
    Span::raw(d)
}

fn section(title: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        title,
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    ))
}

fn help_lines() -> Vec<Line<'static>> {
    vec![
        section("  Normal mode"),
        Line::from(vec![key("  j / k"), desc("           move down / up")]),
        Line::from(vec![key("  g / G"), desc("           top / bottom")]),
        Line::from(vec![key("  Ctrl-d / Ctrl-u"), desc("   half page down / up")]),
        Line::from(vec![key("  Tab"), desc("              cycle sections")]),
        Line::from(vec![key("  s"), desc("                start / stop")]),
        Line::from(vec![key("  r"), desc("                restart")]),
        Line::from(vec![key("  d"), desc("                delete (confirm)")]),
        Line::from(vec![key("  l  Enter"), desc("         logs")]),
        Line::from(vec![key("  e"), desc("                exec into container")]),
        Line::from(vec![key("  v"), desc("                visual (multi-select)")]),
        Line::from(vec![key("  y"), desc("                yank mode (y/i/d)")]),
        Line::from(vec![key("  K"), desc("                inspect selected")]),
        Line::from(vec![key("  /"), desc("                filter list")]),
        Line::from(vec![key("  :"), desc("                command mode")]),
        Line::from(vec![key("  R"), desc("                force refresh")]),
        Line::from(vec![key("  Esc"), desc("             clear active filter")]),
        Line::from(vec![key("  q"), desc("                quit")]),
        Line::from(vec![key("  ?"), desc("                this help")]),
        Line::from(""),
        section("  Visual mode  [v]"),
        Line::from(vec![key("  j / k"), desc("           extend selection")]),
        Line::from(vec![key("  d"), desc("                delete all selected")]),
        Line::from(vec![key("  s"), desc("                start/stop all selected")]),
        Line::from(vec![key("  Esc"), desc("             cancel")]),
        Line::from(""),
        section("  Log mode  [l / Enter]"),
        Line::from(vec![key("  j / k"), desc("           scroll")]),
        Line::from(vec![key("  G"), desc("                jump to latest")]),
        Line::from(vec![key("  f"), desc("                toggle follow mode")]),
        Line::from(vec![key("  q  Esc"), desc("          close")]),
        Line::from(""),
        section("  Exec mode  [e]"),
        Line::from(vec![key("  Enter"), desc("           run command")]),
        Line::from(vec![key("  Tab"), desc("              toggle interactive / one-shot")]),
        Line::from(vec![key("  ↑ / ↓"), desc("           command history")]),
        Line::from(vec![key("  Esc"), desc("             cancel")]),
        Line::from(""),
        section("  Yank mode  [y …]"),
        Line::from(vec![key("  y"), desc("                copy name")]),
        Line::from(vec![key("  i"), desc("                copy secondary (image / ID / mountpoint / subnet)")]),
        Line::from(vec![key("  d"), desc("                copy ID")]),
        Line::from(vec![key("  Esc"), desc("             cancel")]),
        Line::from(""),
        section("  Filter mode  [/]"),
        Line::from(vec![key("  Enter"), desc("           lock filter, return to Normal")]),
        Line::from(vec![key("  Esc"), desc("             cancel edit")]),
        Line::from(""),
        section("  Command mode  [:]"),
        Line::from(vec![key("  Enter"), desc("           execute")]),
        Line::from(vec![key("  ↑ / ↓"), desc("           command history")]),
        Line::from(vec![key("  Esc"), desc("             cancel")]),
        Line::from(""),
        section("  Clipboard  (OSC 52 — works over SSH)"),
        Line::from(vec![desc("  For tmux: "), key("set -g set-clipboard on"), desc(" in ~/.tmux.conf")]),
    ]
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
