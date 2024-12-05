use tv::television::Television;

use crate::colors::{
    BORDER_COLOR, DEFAULT_PREVIEW_CONTENT_FG, DEFAULT_PREVIEW_GUTTER_FG,
    DEFAULT_PREVIEW_GUTTER_SELECTED_FG, DEFAULT_PREVIEW_TITLE_FG,
    DEFAULT_SELECTED_PREVIEW_BG,
};
use ansi_to_tui::IntoText;
use color_eyre::eyre::Result;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Stylize, Text};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap};
use ratatui::Frame;
use std::str::FromStr;
use std::sync::Arc;
use syntect::highlighting::Color as SyntectColor;
use television_channels::channels::OnAir;
use television_channels::entry::Entry;
use television_previewers::previewers::{
    Preview, PreviewContent, FILE_TOO_LARGE_MSG, PREVIEW_NOT_SUPPORTED_MSG,
};
use television_utils::strings::{
    replace_non_printable, shrink_with_ellipsis, ReplaceNonPrintableConfig,
    EMPTY_STRING,
};

impl Television {
    pub fn draw_preview_title_block(
        &self,
        f: &mut Frame,
        rect: Rect,
        preview: &Arc<Preview>,
    ) -> Result<()> {
        let mut preview_title_spans = Vec::new();
        if preview.icon.is_some() && self.config.ui.use_nerd_font_icons {
            let icon = preview.icon.as_ref().unwrap();
            preview_title_spans.push(Span::styled(
                {
                    let mut icon_str = String::from(icon.icon);
                    icon_str.push(' ');
                    icon_str
                },
                Style::default().fg(Color::from_str(icon.color)?),
            ));
        }
        preview_title_spans.push(Span::styled(
            shrink_with_ellipsis(
                &replace_non_printable(
                    preview.title.as_bytes(),
                    &ReplaceNonPrintableConfig::default(),
                )
                .0,
                rect.width.saturating_sub(4) as usize,
            ),
            Style::default().fg(DEFAULT_PREVIEW_TITLE_FG).bold(),
        ));
        let preview_title = Paragraph::new(Line::from(preview_title_spans))
            .block(
                Block::default()
                    .padding(Padding::horizontal(1))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(BORDER_COLOR)),
            )
            .alignment(Alignment::Left);
        f.render_widget(preview_title, rect);
        Ok(())
    }

    pub fn draw_preview_content_block(
        &mut self,
        f: &mut Frame,
        rect: Rect,
        entry: &Entry,
        preview: &Arc<Preview>,
    ) {
        let preview_outer_block = Block::default()
            .title_top(Line::from(" Preview ").alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER_COLOR))
            .style(Style::default())
            .padding(Padding::right(1));

        let preview_inner_block =
            Block::default().style(Style::default()).padding(Padding {
                top: 0,
                right: 1,
                bottom: 0,
                left: 1,
            });
        let inner = preview_outer_block.inner(rect);
        f.render_widget(preview_outer_block, rect);

        let target_line =
            entry.line_number.map(|l| u16::try_from(l).unwrap_or(0));
        let cache_key = compute_cache_key(entry);

        self.maybe_init_preview_scroll(target_line, inner.height);

        // Check if the rendered preview content is already in the cache
        if let Some(preview_paragraph) =
            self.rendered_preview_cache.lock().unwrap().get(&cache_key)
        {
            let p = preview_paragraph.as_ref().clone();
            f.render_widget(
                p.scroll((self.preview_scroll.unwrap_or(0), 0)),
                inner,
            );
            return;
        }
        // If not, render the preview content and cache it if not empty
        let rp = Self::build_preview_paragraph(
            preview_inner_block,
            inner,
            preview.content.clone(),
            target_line,
            self.preview_scroll,
        );
        if !preview.stale {
            self.rendered_preview_cache
                .lock()
                .unwrap()
                .insert(cache_key, &Arc::new(rp.clone()));
        }
        f.render_widget(
            Arc::new(rp)
                .as_ref()
                .clone()
                .scroll((self.preview_scroll.unwrap_or(0), 0)),
            inner,
        );
    }

    #[allow(dead_code)]
    const FILL_CHAR_SLANTED: char = '╱';
    const FILL_CHAR_EMPTY: char = ' ';

    // FIXME: I broke the previewer (srolling is not working as intended)
    // and it looks like the previewer displays the wrong previews
    pub fn build_preview_paragraph(
        preview_block: Block,
        inner: Rect,
        preview_content: PreviewContent,
        target_line: Option<u16>,
        preview_scroll: Option<u16>,
    ) -> Paragraph {
        match preview_content {
            PreviewContent::AnsiText(text) => Self::build_ansi_text_paragraph(
                text,
                preview_block,
                preview_scroll,
            ),
            PreviewContent::PlainText(content) => {
                Self::build_plain_text_paragraph(
                    content,
                    preview_block,
                    target_line,
                    preview_scroll,
                )
            }
            PreviewContent::PlainTextWrapped(content) => {
                Self::build_plain_text_wrapped_paragraph(
                    content,
                    preview_block,
                )
            }
            PreviewContent::SyntectHighlightedText(highlighted_lines) => {
                Self::build_syntect_highlighted_paragraph(
                    highlighted_lines,
                    preview_block,
                    target_line,
                    preview_scroll,
                )
            }
            // meta
            PreviewContent::Loading => Self::build_meta_preview_paragraph(
                inner,
                "Loading...",
                Self::FILL_CHAR_EMPTY,
            )
            .block(preview_block)
            .alignment(Alignment::Left)
            .style(Style::default().add_modifier(Modifier::ITALIC)),
            PreviewContent::NotSupported => {
                Self::build_meta_preview_paragraph(
                    inner,
                    PREVIEW_NOT_SUPPORTED_MSG,
                    Self::FILL_CHAR_EMPTY,
                )
                .block(preview_block)
                .alignment(Alignment::Left)
                .style(Style::default().add_modifier(Modifier::ITALIC))
            }
            PreviewContent::FileTooLarge => {
                Self::build_meta_preview_paragraph(
                    inner,
                    FILE_TOO_LARGE_MSG,
                    Self::FILL_CHAR_EMPTY,
                )
                .block(preview_block)
                .alignment(Alignment::Left)
                .style(Style::default().add_modifier(Modifier::ITALIC))
            }
            PreviewContent::Empty => Paragraph::new(Text::raw(EMPTY_STRING)),
        }
    }

    fn build_ansi_text_paragraph(
        text: String,
        preview_block: Block,
        preview_scroll: Option<u16>,
    ) -> Paragraph {
        let text = replace_non_printable(
            text.as_bytes(),
            &ReplaceNonPrintableConfig {
                replace_line_feed: false,
                replace_control_characters: false,
                ..Default::default()
            },
        )
        .0
        .into_text()
        .unwrap();
        Paragraph::new(text)
            .block(preview_block)
            .scroll((preview_scroll.unwrap_or(0), 0))
    }

    fn build_plain_text_paragraph(
        text: Vec<String>,
        preview_block: Block,
        target_line: Option<u16>,
        preview_scroll: Option<u16>,
    ) -> Paragraph {
        let mut lines = Vec::new();
        for (i, line) in text.iter().enumerate() {
            lines.push(Line::from(vec![
                build_line_number_span(i + 1).style(Style::default().fg(
                    if matches!(
                        target_line,
                        Some(l) if l == u16::try_from(i).unwrap_or(0) + 1
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
            .scroll((preview_scroll.unwrap_or(0), 0))
    }

    fn build_plain_text_wrapped_paragraph(
        text: String,
        preview_block: Block,
    ) -> Paragraph {
        let mut lines = Vec::new();
        for line in text.lines() {
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

    fn build_syntect_highlighted_paragraph(
        highlighted_lines: Vec<Vec<(syntect::highlighting::Style, String)>>,
        preview_block: Block,
        target_line: Option<u16>,
        preview_scroll: Option<u16>,
    ) -> Paragraph {
        compute_paragraph_from_highlighted_lines(
            &highlighted_lines,
            target_line.map(|l| l as usize),
        )
        .block(preview_block)
        .alignment(Alignment::Left)
        .scroll((preview_scroll.unwrap_or(0), 0))
    }

    pub fn maybe_init_preview_scroll(
        &mut self,
        target_line: Option<u16>,
        height: u16,
    ) {
        if self.preview_scroll.is_none() && !self.channel.running() {
            self.preview_scroll =
                Some(target_line.unwrap_or(0).saturating_sub(height / 3));
        }
    }

    pub fn build_meta_preview_paragraph<'a>(
        inner: Rect,
        message: &str,
        fill_char: char,
    ) -> Paragraph<'a> {
        let message_len = message.len();
        if message_len + 8 > inner.width as usize {
            return Paragraph::new(Text::from(EMPTY_STRING));
        }
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
        Paragraph::new(Text::from(lines))
    }
}

fn build_line_number_span<'a>(line_number: usize) -> Span<'a> {
    Span::from(format!("{line_number:5} "))
}

fn compute_paragraph_from_highlighted_lines(
    highlighted_lines: &[Vec<(syntect::highlighting::Style, String)>],
    line_specifier: Option<usize>,
) -> Paragraph<'static> {
    let preview_lines: Vec<Line> = highlighted_lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let line_number =
                build_line_number_span(i + 1).style(Style::default().fg(
                    if line_specifier.is_some()
                        && i == line_specifier.unwrap().saturating_sub(1)
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
                            &(sr.0, sr.1),
                            if line_specifier.is_some()
                                && i == line_specifier
                                    .unwrap()
                                    .saturating_sub(1)
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

pub fn convert_syn_region_to_span<'a>(
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
) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

fn compute_cache_key(entry: &Entry) -> String {
    let mut cache_key = entry.name.clone();
    if let Some(line_number) = entry.line_number {
        cache_key.push_str(&line_number.to_string());
    }
    cache_key
}
