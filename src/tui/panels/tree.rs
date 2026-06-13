use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState},
    Frame,
};

use crate::app::{RuneApp, FileStatus};

pub fn render(f: &mut Frame, area: Rect, app: &RuneApp) {
    let block = Block::default()
        .title(format!(" Files ({}) ", app.changed_files.len()))
        .borders(Borders::ALL)
        .border_style(match app.focus {
            crate::app::PanelFocus::Tree => Style::default().fg(Color::Cyan),
            _ => Style::default(),
        });

    let files = app.filtered_files();
    let items: Vec<ListItem> = files
        .iter()
        .map(|f| {
            let icon = match f.status {
                FileStatus::Added => (" +", Color::Green),
                FileStatus::Modified => (" ~", Color::Yellow),
                FileStatus::Deleted => (" -", Color::Red),
                FileStatus::Renamed => (" >", Color::Cyan),
                FileStatus::Copied => (" >>", Color::Magenta),
                FileStatus::Untracked => (" ?", Color::Gray),
            };

            let staged_mark = if f.staged { "*" } else { " " };
            let label = format!(
                "{}{} {}",
                staged_mark,
                icon.0,
                f.path.to_string_lossy()
            );

            let style = Style::default()
                .fg(icon.1)
                .add_modifier(if f.staged {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                });

            ListItem::new(Text::styled(label, style))
        })
        .collect();

    let mut list_state = ListState::default().with_selected(Some(app.selected_file));
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(list, area, &mut list_state);
}
