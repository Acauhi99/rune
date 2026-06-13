use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState},
};

use crate::app::{FileStatus, RuneApp};
use crate::theme;

pub fn render(f: &mut Frame, area: Rect, app: &RuneApp) {
    let block = Block::default()
        .title(format!(" Files ({}) ", app.changed_files.len()))
        .borders(Borders::ALL)
        .border_style(match app.focus {
            crate::app::PanelFocus::Tree => theme::focused_border(),
            _ => theme::unfocused_border(),
        });

    let files = app.filtered_files();
    let items: Vec<ListItem> = files
        .iter()
        .map(|f| {
            let icon = match f.status {
                FileStatus::Added => (" +", theme::GREEN),
                FileStatus::Modified => (" ~", theme::YELLOW),
                FileStatus::Deleted => (" -", theme::RED),
                FileStatus::Renamed => (" >", theme::CYAN),
                FileStatus::Copied => (" >>", theme::MAGENTA),
                FileStatus::Untracked => (" ?", theme::GRAY),
            };

            let staged_mark = if f.staged { "*" } else { " " };
            let label = format!("{}{} {}", staged_mark, icon.0, f.path.to_string_lossy());

            let style = Style::default().fg(icon.1).add_modifier(if f.staged {
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
        .highlight_style(theme::selected_item())
        .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(list, area, &mut list_state);
}
