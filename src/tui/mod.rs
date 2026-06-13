pub mod panels;

use std::io;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::{AppMode, PanelFocus, RuneApp};
use crate::git;
use crate::git::diff;
use crate::keybind::{Action, map_key};

type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

pub fn run(repo_path: &Path) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let repo = git::open_repo(repo_path)?;
    let path = repo
        .workdir()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| repo_path.to_path_buf());

    let mut app = RuneApp::new(path);
    refresh_state(&repo, &mut app)?;

    let res = run_app(&mut terminal, &repo, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_app(
    terminal: &mut CrosstermTerminal,
    repo: &git2::Repository,
    app: &mut RuneApp,
) -> Result<()> {
    let mut dialog: Option<crate::tui::panels::dialog::DialogState> = None;

    loop {
        terminal.draw(|f| draw(f, app, &dialog))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let event = event::read()?;

        if dialog.is_some() {
            let should_close = {
                let ds = dialog.as_mut().unwrap();
                handle_dialog_event(event, ds, repo, app)?
            };
            if should_close {
                dialog = None;
            }
            continue;
        }

        if app.filter_active {
            handle_filter_event(event, app)?;
            continue;
        }

        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let action = map_key(key);
                if let Some(action) = action
                    && handle_action(action, repo, app, &mut dialog)?
                {
                    break;
                }
            }
            Event::Mouse(mouse) => {
                handle_mouse(mouse, app);
            }
            _ => {}
        }
    }

    Ok(())
}

fn draw(f: &mut Frame, app: &RuneApp, dialog: &Option<crate::tui::panels::dialog::DialogState>) {
    if app.show_help {
        draw_help(f);
        return;
    }

    if let Some(dialog_state) = dialog {
        let area = f.area();
        let dialog_area = centered_rect(50, 10, area);

        f.render_widget(Clear, dialog_area);

        match dialog_state.dialog_type {
            crate::tui::panels::dialog::DialogType::CommitMessage => {
                crate::tui::panels::dialog::render_commit_dialog(f, dialog_area, dialog_state);
            }
            crate::tui::panels::dialog::DialogType::BranchPicker => {
                crate::tui::panels::dialog::render_branch_dialog(f, dialog_area, dialog_state);
            }
        }
        return;
    }

    match app.mode {
        AppMode::Tree | AppMode::Diff => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(f.area());

            crate::tui::panels::tree::render(f, chunks[0], app);
            crate::tui::panels::diff::render(f, chunks[1], app);

            let status_line = format!(
                " {} | ? help | q quit | Tab focus | s stage | c commit | l log | / filter",
                app.current_branch,
            );
            let status_bar = Paragraph::new(status_line)
                .style(Style::default().fg(Color::Gray).bg(Color::Black))
                .block(
                    Block::default()
                        .borders(Borders::TOP)
                        .border_style(Style::default().fg(Color::DarkGray)),
                );
            let status_area = Rect {
                x: 0,
                y: f.area().height.saturating_sub(1),
                width: f.area().width,
                height: 1,
            };
            f.render_widget(status_bar, status_area);
        }
        AppMode::CommitLog => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(f.area());

            crate::tui::panels::commitlog::render(f, chunks[0], app);
            crate::tui::panels::diff::render(f, chunks[1], app);
        }
        AppMode::Help => {
            draw_help(f);
        }
    }
}

fn draw_help(f: &mut Frame) {
    let help_text = vec![
        " Help ",
        "",
        " Navigation:",
        "  ↑/↓ or j/k    — Move selection",
        "  Tab           — Cycle panel focus",
        "  Enter         — View diff / select",
        "  Backspace     — Go back",
        "",
        " Git Operations:",
        "  s             — Stage/unstage file",
        "  S             — Stage all files",
        "  u             — Unstage all files",
        "  c             — Commit (enter message)",
        "  l             — Toggle commit log",
        "  b             — Switch branch",
        "  r             — Refresh",
        "",
        " Search & Display:",
        "  /             — Filter file tree",
        "  ?             — Toggle this help",
        "  PageUp/Down   — Scroll diff",
        "",
        " General:",
        "  q or Esc      — Quit",
        "",
        " Mouse: click to select, scroll to navigate",
    ]
    .join("\n");

    let para = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().fg(Color::White));

    f.render_widget(para, f.area());
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height * (100 - percent_y)) / 200),
            Constraint::Length((r.height * percent_y) / 100),
            Constraint::Length((r.height * (100 - percent_y)) / 200),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width * (100 - percent_x)) / 200),
            Constraint::Length((r.width * percent_x) / 100),
            Constraint::Length((r.width * (100 - percent_x)) / 200),
        ])
        .split(popup_layout[1])[1]
}

fn handle_action(
    action: Action,
    repo: &git2::Repository,
    app: &mut RuneApp,
    dialog: &mut Option<crate::tui::panels::dialog::DialogState>,
) -> Result<bool> {
    match action {
        Action::Quit => return Ok(true),
        Action::Help => {
            app.show_help = !app.show_help;
        }
        Action::Up => match app.mode {
            AppMode::CommitLog => {
                if app.selected_commit > 0 {
                    app.selected_commit -= 1;
                }
            }
            _ => {
                if app.selected_file > 0 {
                    app.selected_file -= 1;
                    load_diff(repo, app)?;
                }
            }
        },
        Action::Down => match app.mode {
            AppMode::CommitLog => {
                if app.selected_commit + 1 < app.commits.len() {
                    app.selected_commit += 1;
                }
            }
            _ => {
                let count = app.filtered_files().len();
                if app.selected_file + 1 < count {
                    app.selected_file += 1;
                    load_diff(repo, app)?;
                }
            }
        },
        Action::ScrollUp => {
            app.diff_scroll = app.diff_scroll.saturating_sub(5);
        }
        Action::ScrollDown if app.diff_scroll < 10000 => {
            app.diff_scroll += 5;
        }
        Action::Enter => {
            if matches!(app.mode, AppMode::CommitLog) {
                if let Some(commit) = app.commits.get(app.selected_commit) {
                    load_commit_diff(repo, app, commit.id)?;
                }
            } else {
                load_diff(repo, app)?;
                app.mode = AppMode::Diff;
                app.focus = PanelFocus::Diff;
            }
        }
        Action::Back => {
            app.mode = AppMode::Tree;
            app.focus = PanelFocus::Tree;
            app.diff = None;
        }
        Action::Tab => {
            app.focus = match app.focus {
                PanelFocus::Tree => {
                    load_diff(repo, app)?;
                    PanelFocus::Diff
                }
                PanelFocus::Diff => PanelFocus::Tree,
            };
        }
        Action::StageToggle => {
            if let Some(file) = app.selected_file_entry() {
                let path = &file.path;
                let _ = if file.staged {
                    git::unstage_file(repo, path)
                } else {
                    git::stage_file(repo, path)
                };
                refresh_state(repo, app)?;
                let _ = load_diff(repo, app);
            }
        }
        Action::StageAll => {
            let _ = git::stage_all(repo);
            refresh_state(repo, app)?;
        }
        Action::UnstageAll => {
            let _ = git::unstage_all(repo);
            refresh_state(repo, app)?;
        }
        Action::Commit => {
            *dialog = Some(crate::tui::panels::dialog::DialogState::new_commit());
        }
        Action::CommitLog => {
            if matches!(app.mode, AppMode::CommitLog) {
                app.mode = AppMode::Tree;
            } else {
                let _ = git::get_commit_history(repo, 100).map(|c| app.commits = c);
                app.mode = AppMode::CommitLog;
                app.selected_commit = 0;
            }
        }
        Action::Refresh => {
            refresh_state(repo, app)?;
        }
        Action::BranchSwitch => {
            let _ = git::list_branches(repo).map(|b| {
                let names: Vec<String> = b.into_iter().map(|bi| bi.name).collect();
                *dialog = Some(crate::tui::panels::dialog::DialogState::new_branch(names));
            });
        }
        Action::Filter => {
            app.filter_active = true;
            app.filter_text.clear();
        }
        _ => {}
    }

    Ok(false)
}

fn handle_dialog_event(
    event: Event,
    dialog_state: &mut crate::tui::panels::dialog::DialogState,
    repo: &git2::Repository,
    app: &mut RuneApp,
) -> Result<bool> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Esc => {
                return Ok(true);
            }
            KeyCode::Enter => match dialog_state.dialog_type {
                crate::tui::panels::dialog::DialogType::CommitMessage => {
                    let msg = dialog_state.input.trim().to_string();
                    if !msg.is_empty() {
                        let _ = git::create_commit(repo, &msg);
                        refresh_state(repo, app)?;
                    }
                    return Ok(true);
                }
                crate::tui::panels::dialog::DialogType::BranchPicker => {
                    if let Some(name) = dialog_state.branches.get(dialog_state.selected_branch) {
                        let _ = git::switch_branch(repo, name);
                        refresh_state(repo, app)?;
                    }
                    return Ok(true);
                }
            },
            KeyCode::Up | KeyCode::Char('k')
                if matches!(
                    dialog_state.dialog_type,
                    crate::tui::panels::dialog::DialogType::BranchPicker
                ) && dialog_state.selected_branch > 0 =>
            {
                dialog_state.selected_branch -= 1;
            }
            KeyCode::Down | KeyCode::Char('j')
                if matches!(
                    dialog_state.dialog_type,
                    crate::tui::panels::dialog::DialogType::BranchPicker
                ) && dialog_state.selected_branch + 1 < dialog_state.branches.len() =>
            {
                dialog_state.selected_branch += 1;
            }
            KeyCode::Char(c) => {
                if matches!(
                    dialog_state.dialog_type,
                    crate::tui::panels::dialog::DialogType::CommitMessage
                ) {
                    dialog_state.input.push(c);
                }
            }
            KeyCode::Backspace => {
                dialog_state.input.pop();
            }
            _ => {}
        },
        _ => {}
    }

    Ok(false)
}

fn handle_filter_event(event: Event, app: &mut RuneApp) -> Result<bool> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Esc => {
                app.filter_active = false;
                app.filter_text.clear();
            }
            KeyCode::Enter => {
                app.filter_active = false;
            }
            KeyCode::Char(c) if !c.is_control() => {
                app.filter_text.push(c);
                app.selected_file = 0;
            }
            KeyCode::Backspace => {
                app.filter_text.pop();
                app.selected_file = 0;
            }
            _ => {}
        },
        _ => {}
    }
    Ok(false)
}

fn handle_mouse(event: crossterm::event::MouseEvent, app: &mut RuneApp) {
    match event.kind {
        MouseEventKind::ScrollDown => match app.focus {
            PanelFocus::Tree => {
                let count = app.filtered_files().len();
                if app.selected_file + 1 < count {
                    app.selected_file += 1;
                }
            }
            PanelFocus::Diff => {
                if app.diff_scroll < 10000 {
                    app.diff_scroll += 3;
                }
            }
        },
        MouseEventKind::ScrollUp => match app.focus {
            PanelFocus::Tree => {
                if app.selected_file > 0 {
                    app.selected_file -= 1;
                }
            }
            PanelFocus::Diff => {
                app.diff_scroll = app.diff_scroll.saturating_sub(3);
            }
        },
        _ => {}
    }
}

fn refresh_state(repo: &git2::Repository, app: &mut RuneApp) -> Result<()> {
    app.changed_files = git::list_changed_files(repo)?;
    app.current_branch = git::get_current_branch(repo);
    app.branches = git::list_branches(repo)?;
    if app.selected_file >= app.changed_files.len() {
        app.selected_file = app.changed_files.len().saturating_sub(1);
    }
    Ok(())
}

fn load_diff(repo: &git2::Repository, app: &mut RuneApp) -> Result<()> {
    let path = app.selected_file_entry().map(|f| f.path.clone());

    if let Some(path) = path {
        app.diff_scroll = 0;

        let staged_diff = diff::get_staged_diff(repo, &path).ok();
        let workdir_diff = diff::get_workdir_diff(repo, &path).ok();

        app.diff = match (staged_diff, workdir_diff) {
            (Some(s), Some(_w)) if !s.is_empty() => Some(s),
            (_, Some(w)) if !w.is_empty() => Some(w),
            (Some(s), _) if !s.is_empty() => Some(s),
            _ => None,
        };
    } else {
        app.diff = None;
    }
    Ok(())
}

fn load_commit_diff(
    repo: &git2::Repository,
    app: &mut RuneApp,
    commit_id: git2::Oid,
) -> Result<()> {
    let _files = git::get_commit_files(repo, commit_id)?;
    let diffs = diff::get_commit_diff(repo, commit_id)?;
    app.diff = diffs.into_iter().next();
    Ok(())
}
