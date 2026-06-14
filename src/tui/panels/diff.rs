use std::sync::OnceLock;

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::app::{DiffLineKind, RuneApp};
use crate::git::diff::get_side_by_side;
use crate::theme;

fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme_set() -> &'static ThemeSet {
    static TS: OnceLock<ThemeSet> = OnceLock::new();
    TS.get_or_init(ThemeSet::load_defaults)
}

fn line_spans(file_ext: &str, content: &str) -> Vec<Span<'static>> {
    let ss = syntax_set();
    let ts = theme_set();

    let syntax = ss
        .find_syntax_by_extension(file_ext)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let theme = &ts.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);

    let mut out = Vec::new();
    for line in LinesWithEndings::from(content) {
        let ranges = highlighter.highlight_line(line, ss).unwrap_or_default();
        for (style, text) in ranges {
            let fg = style.foreground;
            let r = (fg.r as f32 / 255.0) as u8;
            let g = (fg.g as f32 / 255.0) as u8;
            let b = (fg.b as f32 / 255.0) as u8;
            let color = Color::Rgb(r, g, b);
            let mut ratatui_style = Style::default().fg(color);
            if style.font_style.contains(FontStyle::BOLD) {
                ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
            }
            if style.font_style.contains(FontStyle::ITALIC) {
                ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
            }
            out.push(Span::styled(text.to_string(), ratatui_style));
        }
    }
    out
}

pub fn render(f: &mut Frame, area: Rect, app: &RuneApp) {
    let block = Block::default()
        .title(" Diff ")
        .borders(Borders::ALL)
        .border_style(match app.focus {
            crate::app::PanelFocus::Diff => theme::focused_border(),
            _ => theme::unfocused_border(),
        });

    if area.width < 40 {
        let inner = block.inner(area);
        f.render_widget(block, area);
        let msg = Paragraph::new("Window too narrow")
            .block(Block::default())
            .alignment(Alignment::Center);
        f.render_widget(msg, inner);
        return;
    }

    if let Some(ref diff) = app.diff {
        if diff.is_empty() {
            let inner = block.inner(area);
            f.render_widget(block, area);
            let msg = Paragraph::new("No changes")
                .block(Block::default())
                .alignment(Alignment::Center);
            f.render_widget(msg, inner);
            return;
        }

        let file_ext = diff
            .new_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let inner_area = block.inner(area);
        let col_width = ((inner_area.width.saturating_sub(2)) / 2).max(20) as usize;

        let mut lines: Vec<Line> = Vec::new();

        for hunk in &diff.hunks {
            let header_text = hunk.header.trim();
            lines.push(Line::from(Span::styled(
                format!(" {} ", header_text),
                Style::default().fg(theme::DIFF_HEADER).bg(theme::SURFACE),
            )));

            let pairs = get_side_by_side(hunk);

            for &(old_line, new_line) in &pairs {
                let left_spans = side_spans(old_line, true, file_ext, col_width, app.diff_h_scroll);
                let right_spans =
                    side_spans(new_line, false, file_ext, col_width, app.diff_h_scroll);

                let mut row = Vec::new();
                row.extend(left_spans);
                row.push(Span::raw("│"));
                row.extend(right_spans);
                lines.push(Line::from(row));
            }
        }

        let inner = block.inner(area);
        let scroll_offset =
            (app.diff_scroll as usize).min(lines.len().saturating_sub(inner.height as usize));

        let visible: Vec<Line> = lines
            .iter()
            .skip(scroll_offset)
            .take(inner.height as usize)
            .cloned()
            .collect();

        let para = Paragraph::new(visible).block(block);

        f.render_widget(para, area);
    } else {
        let inner = block.inner(area);
        f.render_widget(block, area);
        let msg = Paragraph::new("Select a file to view diff")
            .block(Block::default())
            .alignment(Alignment::Center);
        f.render_widget(msg, inner);
    }
}

fn prefix_and_kind(
    line: Option<&crate::app::DiffLine>,
    is_left: bool,
) -> (&'static str, DiffLineKind) {
    match line {
        Some(l) => {
            let p = if is_left {
                match l.kind {
                    DiffLineKind::Delete => "-",
                    DiffLineKind::Add => " ",
                    DiffLineKind::Context => " ",
                }
            } else {
                match l.kind {
                    DiffLineKind::Add => "+",
                    DiffLineKind::Delete => " ",
                    DiffLineKind::Context => " ",
                }
            };
            (p, l.kind.clone())
        }
        None => ("", DiffLineKind::Context),
    }
}

fn diff_prefix_style(kind: &DiffLineKind) -> Style {
    match kind {
        DiffLineKind::Add => Style::default().fg(theme::GREEN),
        DiffLineKind::Delete => Style::default().fg(theme::RED),
        DiffLineKind::Context => Style::default().fg(theme::TEXT),
    }
}

fn side_spans(
    line: Option<&crate::app::DiffLine>,
    is_left: bool,
    file_ext: &str,
    col_width: usize,
    h_scroll: u16,
) -> Vec<Span<'static>> {
    let (prefix, kind) = prefix_and_kind(line, is_left);
    let content = match line {
        Some(l) => l.content.trim_end_matches('\n').to_string(),
        None => String::new(),
    };

    let lineno_str = match line {
        Some(l) => {
            let n = if is_left { l.old_lineno } else { l.new_lineno };
            match n {
                Some(n) => format!("{:>4}", n),
                None => "    ".to_string(),
            }
        }
        None => "    ".to_string(),
    };

    let prefix_style = diff_prefix_style(&kind);
    let mut spans = vec![
        Span::styled(prefix.to_string(), prefix_style),
        Span::raw(" "),
        Span::styled(lineno_str, Style::default().fg(theme::TEXT_DIM)),
        Span::raw(" "),
    ];

    let avail_width = col_width.saturating_sub(7);
    if !content.is_empty() && avail_width > 0 {
        let h = h_scroll as usize;
        let chars: Vec<char> = content.chars().collect();
        let start = h.min(chars.len());
        let end = (start + avail_width).min(chars.len());
        let visible: String = chars[start..end].iter().collect();

        if !visible.is_empty() {
            if kind == DiffLineKind::Context && !file_ext.is_empty() {
                let mut hl_spans = line_spans(file_ext, &visible);
                spans.append(&mut hl_spans);
            } else if !file_ext.is_empty() {
                let mut hl_spans = line_spans(file_ext, &visible);
                let bg_color = match kind {
                    DiffLineKind::Add => Color::Rgb(22, 42, 28),
                    DiffLineKind::Delete => Color::Rgb(42, 22, 22),
                    _ => Color::Rgb(0, 0, 0),
                };
                for span in &mut hl_spans {
                    span.style = span.style.bg(bg_color);
                }
                spans.append(&mut hl_spans);
            } else {
                let base_style = diff_prefix_style(&kind);
                let bg_color = match kind {
                    DiffLineKind::Add => Color::Rgb(22, 42, 28),
                    DiffLineKind::Delete => Color::Rgb(42, 22, 22),
                    _ => Color::Rgb(0, 0, 0),
                };
                spans.push(Span::styled(visible, base_style.bg(bg_color)));
            }
        }
    }

    let fixed_len: usize = col_width;
    let current: usize = spans.iter().map(|s| s.content.len()).sum();
    if current < fixed_len {
        spans.push(Span::raw(" ".repeat(fixed_len - current)));
    }

    spans
}
