use ratatui::prelude::{Color, Line, Modifier, Span, Style, Stylize, Text};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::layout::{Alignment, Rect};
use std::sync::Arc;
use syntect::highlighting::Color as SyntectColor;
use crate::previewers::{Preview, PreviewContent, FILE_TOO_LARGE_MSG, PREVIEW_NOT_SUPPORTED_MSG};
use crate::television::Television;
use crate::utils::strings::{EMPTY_STRING, FOUR_SPACES};

//  preview
pub const DEFAULT_PREVIEW_TITLE_FG: Color = Color::Blue;
const DEFAULT_SELECTED_PREVIEW_BG: Color = Color::Rgb(50, 50, 50);
const DEFAULT_PREVIEW_CONTENT_FG: Color = Color::Rgb(150, 150, 180);
const DEFAULT_PREVIEW_GUTTER_FG: Color = Color::Rgb(70, 70, 70);
const DEFAULT_PREVIEW_GUTTER_SELECTED_FG: Color = Color::Rgb(255, 150, 150);

impl Television {
    const FILL_CHAR_SLANTED: char = '╱';
    const FILL_CHAR_EMPTY: char = ' ';

    pub fn build_preview_paragraph<'b>(
        &'b mut self,
        preview_block: Block<'b>,
        inner: Rect,
        preview: &Arc<Preview>,
        target_line: Option<u16>,
    ) -> Paragraph<'b> {
        self.maybe_init_preview_scroll(target_line, inner.height);
        match &preview.content {
            PreviewContent::PlainText(content) => {
                let mut lines = Vec::new();
                for (i, line) in content.iter().enumerate() {
                    lines.push(Line::from(vec![
                        build_line_number_span(i + 1).style(Style::default().fg(
                            // FIXME: this actually might panic in some edge cases
                            if matches!(
                                target_line,
                                Some(l) if l == u16::try_from(i).unwrap() + 1
                            )
                            {
                                DEFAULT_PREVIEW_GUTTER_SELECTED_FG
                            } else {
                                DEFAULT_PREVIEW_GUTTER_FG
                            },
                        )),
                        Span::styled(" │ ",
                            Style::default().fg(DEFAULT_PREVIEW_GUTTER_FG).dim()),
                        Span::styled(
                            line.to_string(),
                            Style::default().fg(DEFAULT_PREVIEW_CONTENT_FG).bg(
                                if matches!(target_line, Some(l) if l == u16::try_from(i).unwrap() + 1) {
                                    DEFAULT_SELECTED_PREVIEW_BG
                                } else {
                                    Color::Reset
                                },
                            ),
                        ),
                    ]));
                }
                let text = Text::from(lines);
                Paragraph::new(text)
                    .block(preview_block)
                    .scroll((self.preview_scroll.unwrap_or(0), 0))
            }
            PreviewContent::PlainTextWrapped(content) => {
                let mut lines = Vec::new();
                for line in content.lines() {
                    lines.push(Line::styled(
                        line.to_string(),
                        Style::default().fg(DEFAULT_PREVIEW_CONTENT_FG),
                    ));
                }
                let text = Text::from(lines);
                Paragraph::new(text)
                    .block(preview_block)
                    .wrap(Wrap { trim: true })
            }
            PreviewContent::HighlightedText(highlighted_lines) => {
                compute_paragraph_from_highlighted_lines(
                    highlighted_lines,
                    target_line.map(|l| l as usize),
                    self.preview_scroll.unwrap_or(0),
                    self.preview_pane_height,
                )
                .block(preview_block)
                .alignment(Alignment::Left)
                .scroll((self.preview_scroll.unwrap_or(0), 0))
            }
            // meta
            PreviewContent::Loading => self
                .build_meta_preview_paragraph(
                    inner,
                    "Loading...",
                    Self::FILL_CHAR_EMPTY,
                )
                .block(preview_block)
                .alignment(Alignment::Left)
                .style(Style::default().add_modifier(Modifier::ITALIC)),
            PreviewContent::NotSupported => self
                .build_meta_preview_paragraph(
                    inner,
                    PREVIEW_NOT_SUPPORTED_MSG,
                    Self::FILL_CHAR_SLANTED,
                )
                .block(preview_block)
                .alignment(Alignment::Left)
                .style(Style::default().add_modifier(Modifier::ITALIC)),
            PreviewContent::FileTooLarge => self
                .build_meta_preview_paragraph(
                    inner,
                    FILE_TOO_LARGE_MSG,
                    Self::FILL_CHAR_SLANTED,
                )
                .block(preview_block)
                .alignment(Alignment::Left)
                .style(Style::default().add_modifier(Modifier::ITALIC)),
            _ => Paragraph::new(Text::raw(EMPTY_STRING)),
        }
    }

    pub fn maybe_init_preview_scroll(
        &mut self,
        target_line: Option<u16>,
        height: u16,
    ) {
        if self.preview_scroll.is_none() {
            self.preview_scroll =
                Some(target_line.unwrap_or(0).saturating_sub(height / 3));
        }
    }

    pub fn build_meta_preview_paragraph<'a>(
        &mut self,
        inner: Rect,
        message: &str,
        fill_char: char,
    ) -> Paragraph<'a> {
        if let Some(paragraph) = self.meta_paragraph_cache.get(message) {
            return paragraph.clone();
        }
        let message_len = message.len();
        let fill_char_str = fill_char.to_string();
        let fill_line = fill_char_str.repeat(inner.width as usize);

        // Build the paragraph content with slanted lines and center the custom message
        let mut lines = Vec::new();

        // Calculate the vertical center
        let vertical_center = inner.height as usize / 2;
        let horizontal_padding = (inner.width as usize - message_len) / 2 - 4;

        // Fill the paragraph with slanted lines and insert the centered custom message
        for i in 0..inner.height {
            if i as usize == vertical_center {
                // Center the message horizontally in the middle line
                let line = format!(
                    "{}  {}  {}",
                    fill_char_str.repeat(horizontal_padding),
                    message,
                    fill_char_str.repeat(
                        inner.width as usize
                            - horizontal_padding
                            - message_len
                    )
                );
                lines.push(Line::from(line));
            } else if i as usize + 1 == vertical_center
                || (i as usize).saturating_sub(1) == vertical_center
            {
                let line = format!(
                    "{}  {}  {}",
                    fill_char_str.repeat(horizontal_padding),
                    " ".repeat(message_len),
                    fill_char_str.repeat(
                        inner.width as usize
                            - horizontal_padding
                            - message_len
                    )
                );
                lines.push(Line::from(line));
            } else {
                lines.push(Line::from(fill_line.clone()));
            }
        }

        // Create a paragraph with the generated content
        let p = Paragraph::new(Text::from(lines));
        self.meta_paragraph_cache
            .insert(message.to_string(), p.clone());
        p
    }
}

fn build_line_number_span<'a>(line_number: usize) -> Span<'a> {
    Span::from(format!("{line_number:5} "))
}

fn compute_paragraph_from_highlighted_lines(
    highlighted_lines: &[Vec<(syntect::highlighting::Style, String)>],
    line_specifier: Option<usize>,
    scroll: u16,
    preview_pane_height: u16,
) -> Paragraph<'static> {
    let preview_lines: Vec<Line> = highlighted_lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            if i < scroll as usize
                || i >= (scroll + preview_pane_height) as usize
            {
                return Line::from(Span::raw(EMPTY_STRING));
            }
            let line_number =
                build_line_number_span(i + 1).style(Style::default().fg(
                    if line_specifier.is_some()
                        && i == line_specifier.unwrap() - 1
                    {
                        DEFAULT_PREVIEW_GUTTER_SELECTED_FG
                    } else {
                        DEFAULT_PREVIEW_GUTTER_FG
                    },
                ));
            Line::from_iter(
                std::iter::once(line_number)
                    .chain(std::iter::once(Span::styled(
                        " │ ",
                        Style::default().fg(DEFAULT_PREVIEW_GUTTER_FG).dim(),
                    )))
                    .chain(l.iter().cloned().map(|sr| {
                        convert_syn_region_to_span(
                            &(sr.0, sr.1.replace('\t', FOUR_SPACES)),
                            if line_specifier.is_some()
                                && i == line_specifier.unwrap() - 1
                            {
                                Some(SyntectColor {
                                    r: 50,
                                    g: 50,
                                    b: 50,
                                    a: 255,
                                })
                            } else {
                                None
                            },
                        )
                    })),
            )
        })
        .collect();

    Paragraph::new(preview_lines)
}

fn convert_syn_region_to_span<'a>(
    syn_region: &(syntect::highlighting::Style, String),
    background: Option<syntect::highlighting::Color>,
) -> Span<'a> {
    let mut style = Style::default()
        .fg(convert_syn_color_to_ratatui_color(syn_region.0.foreground));
    if let Some(background) = background {
        style = style.bg(convert_syn_color_to_ratatui_color(background));
    }
    style = match syn_region.0.font_style {
        syntect::highlighting::FontStyle::BOLD => style.bold(),
        syntect::highlighting::FontStyle::ITALIC => style.italic(),
        syntect::highlighting::FontStyle::UNDERLINE => style.underlined(),
        _ => style,
    };
    Span::styled(syn_region.1.clone(), style)
}

fn convert_syn_color_to_ratatui_color(
    color: syntect::highlighting::Color,
) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(color.r, color.g, color.b)
}