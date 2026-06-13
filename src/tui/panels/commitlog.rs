use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph},
};

use crate::app::RuneApp;
use crate::theme;

pub fn render(f: &mut Frame, area: Rect, app: &RuneApp) {
    let block = Block::default()
        .title(format!(" Commit Log ({}) ", app.commits.len()))
        .borders(Borders::ALL)
        .border_style(if matches!(app.mode, crate::app::AppMode::CommitLog) {
            theme::focused_border()
        } else {
            theme::unfocused_border()
        });

    let items: Vec<ListItem> = app
        .commits
        .iter()
        .map(|c| {
            let hash = &c.id.to_string()[..7];
            let lines = vec![
                Line::from(Span::styled(
                    format!(" {}", hash),
                    Style::default().fg(theme::YELLOW),
                )),
                Line::from(Span::styled(
                    format!(" {}", c.message),
                    Style::default().fg(theme::TEXT),
                )),
                Line::from(Span::styled(
                    format!(" {}", c.author),
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                )),
            ];
            ListItem::new(lines)
        })
        .collect();

    if items.is_empty() {
        let inner = block.inner(area);
        f.render_widget(block, area);
        let msg = Paragraph::new("No commits found").block(Block::default());
        f.render_widget(msg, inner);
        return;
    }

    let mut list_state = ListState::default().with_selected(Some(app.selected_commit));
    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selected_item())
        .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(list, area, &mut list_state);
}
