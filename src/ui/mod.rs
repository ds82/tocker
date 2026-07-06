use ratatui::{
    layout::{Constraint, Layout},
    Frame,
};

use crate::app::{App, Mode};

mod command;
mod help;
mod inspect;
mod list;
mod logs;
mod menu;

pub fn render(f: &mut Frame, app: &App) {
    if let Mode::Log { scroll, follow } = app.mode {
        logs::render(f, f.area(), app, scroll, follow);
        return;
    }

    let area = f.area();

    let vertical = Layout::vertical([
        Constraint::Length(1), // title / mode bar
        Constraint::Fill(1),   // main list
        Constraint::Length(1), // status / command bar
    ])
    .split(area);

    command::render_title(f, vertical[0], app);
    list::render(f, vertical[1], app);
    command::render_statusbar(f, vertical[2], app);

    if let Mode::Menu { cursor } = app.mode {
        menu::render(f, area, cursor, app.section);
    }

    if let Mode::Help { scroll } = app.mode {
        help::render(f, area, scroll);
    }

    if let Mode::Inspect { scroll } = app.mode {
        inspect::render(f, area, app, scroll);
    }
}
