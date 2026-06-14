use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

#[allow(dead_code)]
pub enum Action {
    Quit,
    Up,
    Down,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    Enter,
    Back,
    Tab,
    StageToggle,
    StageAll,
    UnstageAll,
    Commit,
    CommitLog,
    Refresh,
    BranchSwitch,
    Filter,
    Help,
    Mouse(MouseEvent),
}

pub fn map_key(key: KeyEvent) -> Option<Action> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(Action::Quit);
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
        KeyCode::PageUp => Some(Action::ScrollUp),
        KeyCode::PageDown => Some(Action::ScrollDown),
        KeyCode::Enter => Some(Action::Enter),
        KeyCode::Backspace => Some(Action::Back),
        KeyCode::Tab => Some(Action::Tab),
        KeyCode::Char('s') => Some(Action::StageToggle),
        KeyCode::Char('S') => Some(Action::StageAll),
        KeyCode::Char('u') => Some(Action::UnstageAll),
        KeyCode::Char('c') => Some(Action::Commit),
        KeyCode::Char('l') => Some(Action::CommitLog),
        KeyCode::Char('r') => Some(Action::Refresh),
        KeyCode::Char('b') => Some(Action::BranchSwitch),
        KeyCode::Char('/') => Some(Action::Filter),
        KeyCode::Char('?') => Some(Action::Help),
        KeyCode::Left | KeyCode::Char('h') => Some(Action::ScrollLeft),
        KeyCode::Right => Some(Action::ScrollRight),
        KeyCode::Home => Some(Action::Up),
        KeyCode::End => Some(Action::Down),
        _ => None,
    }
}
