use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::theme;

pub enum DialogType {
    CommitMessage,
    BranchPicker,
}

pub struct DialogState {
    pub dialog_type: DialogType,
    pub input: String,
    pub _cursor_position: usize,
    pub branches: Vec<String>,
    pub selected_branch: usize,
}

impl DialogState {
    pub fn new_commit() -> Self {
        Self {
            dialog_type: DialogType::CommitMessage,
            input: String::new(),
            _cursor_position: 0,
            branches: Vec::new(),
            selected_branch: 0,
        }
    }

    pub fn new_branch(branches: Vec<String>) -> Self {
        Self {
            dialog_type: DialogType::BranchPicker,
            input: String::new(),
            _cursor_position: 0,
            branches,
            selected_branch: 0,
        }
    }
}

pub fn render_commit_dialog(f: &mut Frame, area: Rect, dialog: &DialogState) {
    let block = Block::default()
        .title(" Commit ")
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(area);

    let prompt = Paragraph::new("Enter commit message:")
        .style(Style::default().fg(theme::TEXT))
        .block(Block::default())
        .alignment(Alignment::Left);

    let input_area = Rect {
        x: inner.x,
        y: inner.y + 1,
        width: inner.width,
        height: 1,
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .style(Style::default().bg(theme::BG));

    let input_para = Paragraph::new(dialog.input.as_str())
        .style(Style::default().fg(theme::TEXT))
        .block(input_block);

    let hint = Paragraph::new("Enter to confirm, Esc to cancel")
        .style(Style::default().fg(theme::TEXT_DIM))
        .block(Block::default())
        .alignment(Alignment::Left);

    let hint_area = Rect {
        x: inner.x,
        y: inner.y + 3,
        width: inner.width,
        height: 1,
    };

    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(prompt, inner);
    f.render_widget(input_para, input_area);
    f.render_widget(hint, hint_area);
}

pub fn render_branch_dialog(f: &mut Frame, area: Rect, dialog: &DialogState) {
    let block = Block::default()
        .title(" Switch Branch ")
        .borders(Borders::ALL)
        .border_style(theme::focused_border())
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(area);

    let items: Vec<String> = dialog
        .branches
        .iter()
        .enumerate()
        .map(|(i, b)| {
            if i == dialog.selected_branch {
                format!("> {}", b)
            } else {
                format!("  {}", b)
            }
        })
        .collect();

    let text = items.join("\n");
    let para = Paragraph::new(text)
        .style(Style::default().fg(theme::TEXT))
        .block(Block::default());

    let hint = Paragraph::new("↑↓ to navigate, Enter to select, Esc to cancel")
        .style(Style::default().fg(theme::TEXT_DIM))
        .block(Block::default())
        .alignment(Alignment::Left);

    let hint_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(1),
        width: inner.width,
        height: 1,
    };

    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(para, inner);
    f.render_widget(hint, hint_area);
}
