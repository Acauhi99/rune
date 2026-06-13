use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph},
};

use crate::app::RuneApp;

pub fn render(f: &mut Frame, area: Rect, app: &RuneApp) {
    let block = Block::default()
        .title(format!(" Commit Log ({}) ", app.commits.len()))
        .borders(Borders::ALL)
        .border_style(if matches!(app.mode, crate::app::AppMode::CommitLog) {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });

    let items: Vec<ListItem> = app
        .commits
        .iter()
        .map(|c| {
            let hash = &c.id.to_string()[..7];
            let lines = vec![
                Line::from(Span::styled(
                    format!(" {}", hash),
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(Span::styled(
                    format!(" {}", c.message),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    format!(" {}", c.author),
                    Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
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
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(list, area, &mut list_state);
}
