use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, Mode, PendingAction};

pub fn render_title(f: &mut Frame, area: Rect, app: &App) {
    let mode_label = match &app.mode {
        Mode::Normal => {
            Span::styled(" [normal] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        }
        Mode::Command(_) => {
            Span::styled(" [command] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        }
        Mode::Filter(_) => {
            Span::styled(" [filter] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        }
        Mode::Log { .. } => {
            Span::styled(" [log] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        }
        Mode::Confirm(_) => {
            Span::styled(" [confirm] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        }
        Mode::Menu { .. } => {
            Span::styled(" [menu] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        }
        Mode::Visual { .. } => {
            Span::styled(" [visual] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        }
    };

    let section = Span::styled(
        app.section.label(),
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    );

    let right = Span::styled(" tocker ", Style::default().fg(Color::DarkGray));

    let line = Line::from(vec![mode_label, section, right]);
    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(Color::DarkGray)),
        area,
    );
}

pub fn render_statusbar(f: &mut Frame, area: Rect, app: &App) {
    let content = match &app.mode {
        Mode::Command(input) => Line::from(vec![
            Span::styled(":", Style::default().fg(Color::Yellow)),
            Span::raw(input.as_str()),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),

        Mode::Filter(input) => Line::from(vec![
            Span::styled("/", Style::default().fg(Color::Cyan)),
            Span::raw(input.as_str()),
            Span::styled("█", Style::default().fg(Color::Cyan)),
        ]),

        Mode::Confirm(pending) => {
            let label = match pending {
                PendingAction::Remove { display, .. } => {
                    format!("remove {display}? [y] confirm  [n/Esc] cancel")
                }
                PendingAction::BulkRemoveContainers { count, .. } => {
                    format!("remove {count} containers? [y] confirm  [n/Esc] cancel")
                }
            };
            Line::from(vec![
                Span::styled(" ! ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(label),
            ])
        }

        Mode::Menu { .. } => Line::from(vec![
            Span::raw("  "),
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(" navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" select  "),
            Span::styled("Esc/Tab", Style::default().fg(Color::DarkGray)),
            Span::raw(" close"),
        ]),

        Mode::Visual { anchor, cursor } => {
            let lo = anchor.min(cursor);
            let hi = anchor.max(cursor);
            let count = hi - lo + 1;
            Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{count} selected"), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("j/k", Style::default().fg(Color::Magenta)),
                Span::raw(" extend  "),
                Span::styled("d", Style::default().fg(Color::Red)),
                Span::raw(" delete  "),
                Span::styled("s", Style::default().fg(Color::Green)),
                Span::raw(" start/stop  "),
                Span::styled("Esc", Style::default().fg(Color::DarkGray)),
                Span::raw(" cancel"),
            ])
        }

        _ => {
            if let Some(ref msg) = app.status {
                Line::from(Span::styled(
                    format!(" {msg}"),
                    Style::default().fg(Color::Yellow),
                ))
            } else {
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("e", Style::default().fg(Color::Green)),
                    Span::raw(" exec  "),
                    Span::styled("v", Style::default().fg(Color::Magenta)),
                    Span::raw(" visual  "),
                    Span::styled("s", Style::default().fg(Color::Green)),
                    Span::raw(" start/stop  "),
                    Span::styled("r", Style::default().fg(Color::Green)),
                    Span::raw(" restart  "),
                    Span::styled("d", Style::default().fg(Color::Red)),
                    Span::raw(" delete  "),
                    Span::styled("l", Style::default().fg(Color::Cyan)),
                    Span::raw(" logs  "),
                    Span::styled("/", Style::default().fg(Color::Cyan)),
                    Span::raw(" filter  "),
                    Span::styled("Tab", Style::default().fg(Color::Yellow)),
                    Span::raw(" sections  "),
                    Span::styled("q", Style::default().fg(Color::DarkGray)),
                    Span::raw(" quit"),
                ])
            }
        }
    };

    f.render_widget(
        Paragraph::new(content).style(Style::default().bg(Color::DarkGray)),
        area,
    );
}
