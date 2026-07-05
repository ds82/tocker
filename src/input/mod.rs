use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::Mode;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    Top,
    Bottom,
    HalfPageUp,
    HalfPageDown,
    ToggleStartStop,
    Restart,
    Delete,
    OpenLogs,
    Refresh,
    EnterCommand,
    EnterFilter,
    OpenMenu,
    ToggleFollow,
    Confirm,
    Cancel,
    Enter,
    Escape,
    Char(char),
    Backspace,
    HistoryPrev,
    HistoryNext,
    Exec,
    Visual,
    None,
}

pub fn map_key(mode: &Mode, key: KeyEvent) -> Action {
    match mode {
        Mode::Normal => map_normal(key),
        Mode::Filter(_) => map_filter_input(key),
        Mode::Command(_) => map_command_input(key),
        Mode::Log { .. } => map_log(key),
        Mode::Confirm(_) => map_confirm(key),
        Mode::Menu { .. } => map_menu(key),
        Mode::Visual { .. } => map_visual(key),
    }
}

fn map_normal(key: KeyEvent) -> Action {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => Action::Quit,
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => Action::MoveDown,
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => Action::MoveUp,
        (KeyCode::Char('g'), _) | (KeyCode::Home, _) => Action::Top,
        (KeyCode::Char('G'), _) | (KeyCode::End, _) => Action::Bottom,
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => Action::HalfPageDown,
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => Action::HalfPageUp,
        (KeyCode::Char('s'), _) => Action::ToggleStartStop,
        (KeyCode::Char('r'), _) => Action::Restart,
        (KeyCode::Char('d'), _) => Action::Delete,
        (KeyCode::Char('l'), _) | (KeyCode::Enter, _) => Action::OpenLogs,
        (KeyCode::Char('e'), _) => Action::Exec,
        (KeyCode::Char('v'), _) => Action::Visual,
        (KeyCode::Char('R'), _) => Action::Refresh,
        (KeyCode::Char(':'), _) => Action::EnterCommand,
        (KeyCode::Char('/'), _) => Action::EnterFilter,
        (KeyCode::Tab, _) => Action::OpenMenu,
        _ => Action::None,
    }
}

fn map_visual(key: KeyEvent) -> Action {
    match (key.code, key.modifiers) {
        (KeyCode::Esc, _) => Action::Escape,
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => Action::MoveDown,
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => Action::MoveUp,
        (KeyCode::Char('g'), _) | (KeyCode::Home, _) => Action::Top,
        (KeyCode::Char('G'), _) | (KeyCode::End, _) => Action::Bottom,
        (KeyCode::Char('d'), _) => Action::Delete,
        (KeyCode::Char('s'), _) => Action::ToggleStartStop,
        _ => Action::None,
    }
}

fn map_menu(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Tab => Action::Escape,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Enter => Action::Enter,
        _ => Action::None,
    }
}

fn map_log(key: KeyEvent) -> Action {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => Action::Quit,
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => Action::MoveDown,
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => Action::MoveUp,
        (KeyCode::Char('G'), _) | (KeyCode::End, _) => Action::Bottom,
        (KeyCode::Char('f'), _) => Action::ToggleFollow,
        _ => Action::None,
    }
}

fn map_command_input(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::Escape,
        KeyCode::Enter => Action::Enter,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Up => Action::HistoryPrev,
        KeyCode::Down => Action::HistoryNext,
        KeyCode::Char(c) => Action::Char(c),
        _ => Action::None,
    }
}

fn map_filter_input(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::Escape,
        KeyCode::Enter => Action::Enter,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Char(c) => Action::Char(c),
        _ => Action::None,
    }
}

fn map_confirm(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => Action::Confirm,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::Cancel,
        _ => Action::None,
    }
}
