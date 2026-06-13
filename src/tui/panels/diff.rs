use std::sync::OnceLock;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::app::{DiffLineKind, RuneApp};
use crate::git::diff::get_side_by_side;

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
            crate::app::PanelFocus::Diff => Style::default().fg(Color::Cyan),
            _ => Style::default(),
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

        let mut lines: Vec<Line> = Vec::new();

        for hunk in &diff.hunks {
            let header_text = hunk.header.trim();
            lines.push(Line::from(
                Span::styled(
                    header_text,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::DIM),
                ),
            ));

            let col_width = ((area.width.saturating_sub(4)) / 2).max(20) as usize;
            let pairs = get_side_by_side(hunk);

            for &(old_line, new_line) in &pairs {
                let left_line = colored_line(old_line, true, file_ext);
                let right_line = colored_line(new_line, false, file_ext);

                let left_text = left_line
                    .spans
                    .iter()
                    .map(|s| s.content.clone())
                    .collect::<String>();
                if left_text.len() > col_width {
                    lines.push(left_line);
                    lines.push(right_line);
                } else {
                    let padded_left = format!("{:<width$}", left_text, width = col_width);
                    let spans: Vec<Span> = std::iter::once(
                        Span::styled(padded_left, Style::default().fg(Color::White)),
                    )
                    .chain(
                        std::iter::once(Span::raw("│")),
                    )
                    .chain(right_line.spans)
                    .collect();
                    lines.push(Line::from(spans));
                }
            }
        }

        let inner = block.inner(area);
        let scroll_offset = (app.diff_scroll as usize).min(
            lines.len().saturating_sub(inner.height as usize),
        );

        let visible: Vec<Line> = lines
            .iter()
            .skip(scroll_offset)
            .take(inner.height as usize)
            .cloned()
            .collect();

        let para = Paragraph::new(visible)
            .block(block)
            .scroll((app.diff_scroll, 0));

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

fn prefix_and_kind(line: Option<&crate::app::DiffLine>, is_left: bool) -> (&'static str, DiffLineKind) {
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
        DiffLineKind::Add => Style::default().fg(Color::Green),
        DiffLineKind::Delete => Style::default().fg(Color::Red),
        DiffLineKind::Context => Style::default().fg(Color::White),
    }
}

fn colored_line(
    line: Option<&crate::app::DiffLine>,
    is_left: bool,
    file_ext: &str,
) -> Line<'static> {
    let (prefix, kind) = prefix_and_kind(line, is_left);
    let content = match line {
        Some(l) => l.content.trim_end_matches('\n').to_string(),
        None => String::new(),
    };

    let prefix_style = diff_prefix_style(&kind);
    let prefix_span = Span::styled(prefix.to_string(), prefix_style);

    if kind == DiffLineKind::Context && !content.is_empty() && !file_ext.is_empty() {
        let mut spans = line_spans(file_ext, &content);
        let mut result = vec![prefix_span];
        result.append(&mut spans);
        Line::from(result)
    } else {
        let content_style = diff_prefix_style(&kind);
        let full = if content.is_empty() {
            prefix.to_string()
        } else {
            let lineno = match line {
                Some(l) => {
                    let n = if is_left { l.old_lineno } else { l.new_lineno };
                    match n {
                        Some(n) => format!("{:>4}", n),
                        None => "    ".to_string(),
                    }
                }
                None => "    ".to_string(),
            };
            format!("{}{} {}", prefix, lineno, content)
        };
        Line::from(Span::styled(full, content_style))
    }
}
