use crate::{
    cache::RenderedPreviewCache,
    colors::{Colorscheme, PreviewColorscheme},
};
use color_eyre::eyre::Result;
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap};
use ratatui::Frame;
use ratatui::{
    layout::{Alignment, Rect},
    prelude::{Color, Line, Modifier, Span, Style, Stylize, Text},
};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use television_channels::entry::Entry;
use television_previewers::{
    ansi::IntoText,
    previewers::{
        Preview, PreviewContent, FILE_TOO_LARGE_MSG, PREVIEW_NOT_SUPPORTED_MSG,
    },
};
use television_utils::strings::{
    replace_non_printable, shrink_with_ellipsis, ReplaceNonPrintableConfig,
    EMPTY_STRING,
};

#[allow(dead_code)]
const FILL_CHAR_SLANTED: char = '╱';
const FILL_CHAR_EMPTY: char = ' ';

#[allow(clippy::needless_pass_by_value)]
pub fn build_preview_paragraph(
    preview_block: Block<'_>,
    inner: Rect,
    preview_content: PreviewContent,
    target_line: Option<u16>,
    preview_scroll: u16,
    colorscheme: Colorscheme,
) -> Paragraph<'_> {
    match preview_content {
        PreviewContent::AnsiText(text) => {
            build_ansi_text_paragraph(text, preview_block, preview_scroll)
        }
        PreviewContent::PlainText(content) => build_plain_text_paragraph(
            content,
            preview_block,
            target_line,
            preview_scroll,
            colorscheme.preview,
        ),
        PreviewContent::PlainTextWrapped(content) => {
            build_plain_text_wrapped_paragraph(
                content,
                preview_block,
                colorscheme.preview,
            )
        }
        PreviewContent::SyntectHighlightedText(highlighted_lines) => {
            build_syntect_highlighted_paragraph(
                highlighted_lines,
                preview_block,
                target_line,
                preview_scroll,
                colorscheme.preview,
            )
        }
        // meta
        PreviewContent::Loading => {
            build_meta_preview_paragraph(inner, "Loading...", FILL_CHAR_EMPTY)
                .block(preview_block)
                .alignment(Alignment::Left)
                .style(Style::default().add_modifier(Modifier::ITALIC))
        }
        PreviewContent::NotSupported => build_meta_preview_paragraph(
            inner,
            PREVIEW_NOT_SUPPORTED_MSG,
            FILL_CHAR_EMPTY,
        )
        .block(preview_block)
        .alignment(Alignment::Left)
        .style(Style::default().add_modifier(Modifier::ITALIC)),
        PreviewContent::FileTooLarge => build_meta_preview_paragraph(
            inner,
            FILE_TOO_LARGE_MSG,
            FILL_CHAR_EMPTY,
        )
        .block(preview_block)
        .alignment(Alignment::Left)
        .style(Style::default().add_modifier(Modifier::ITALIC)),
        PreviewContent::Empty => Paragraph::new(Text::raw(EMPTY_STRING)),
    }
}

#[allow(clippy::needless_pass_by_value)]
fn build_ansi_text_paragraph(
    text: String,
    preview_block: Block,
    preview_scroll: u16,
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
        .scroll((preview_scroll, 0))
}

#[allow(clippy::needless_pass_by_value)]
fn build_plain_text_paragraph(
    text: Vec<String>,
    preview_block: Block<'_>,
    target_line: Option<u16>,
    preview_scroll: u16,
    colorscheme: PreviewColorscheme,
) -> Paragraph<'_> {
    let mut lines = Vec::new();
    for (i, line) in text.iter().enumerate() {
        lines.push(Line::from(vec![
            build_line_number_span(i + 1).style(Style::default().fg(
                if matches!(
                        target_line,
                        Some(l) if l == u16::try_from(i).unwrap_or(0) + 1
                    )
                {
                    colorscheme.gutter_selected_fg
                } else {
                    colorscheme.gutter_fg
                },
            )),
            Span::styled(" │ ",
                         Style::default().fg(colorscheme.gutter_fg).dim()),
            Span::styled(
                line.to_string(),
                Style::default().fg(colorscheme.content_fg).bg(
                    if matches!(target_line, Some(l) if l == u16::try_from(i).unwrap() + 1) {
                        colorscheme.highlight_bg
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
        .scroll((preview_scroll, 0))
}

#[allow(clippy::needless_pass_by_value)]
fn build_plain_text_wrapped_paragraph(
    text: String,
    preview_block: Block<'_>,
    colorscheme: PreviewColorscheme,
) -> Paragraph<'_> {
    let mut lines = Vec::new();
    for line in text.lines() {
        lines.push(Line::styled(
            line.to_string(),
            Style::default().fg(colorscheme.content_fg),
        ));
    }
    let text = Text::from(lines);
    Paragraph::new(text)
        .block(preview_block)
        .wrap(Wrap { trim: true })
}

#[allow(clippy::needless_pass_by_value)]
fn build_syntect_highlighted_paragraph(
    highlighted_lines: Vec<Vec<(syntect::highlighting::Style, String)>>,
    preview_block: Block,
    target_line: Option<u16>,
    preview_scroll: u16,
    colorscheme: PreviewColorscheme,
) -> Paragraph {
    compute_paragraph_from_highlighted_lines(
        &highlighted_lines,
        target_line.map(|l| l as usize),
        colorscheme,
    )
    .block(preview_block)
    .alignment(Alignment::Left)
    .scroll((preview_scroll, 0))
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
                    inner.width as usize - horizontal_padding - message_len
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
                    inner.width as usize - horizontal_padding - message_len
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

#[allow(clippy::too_many_arguments)]
pub fn draw_preview_content_block(
    f: &mut Frame,
    rect: Rect,
    entry: &Entry,
    preview: &Arc<Preview>,
    rendered_preview_cache: &Arc<Mutex<RenderedPreviewCache<'static>>>,
    preview_scroll: u16,
    use_nerd_font_icons: bool,
    colorscheme: &Colorscheme,
) -> Result<()> {
    let mut preview_title_spans = vec![Span::from(" ")];
    if preview.icon.is_some() && use_nerd_font_icons {
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
        Style::default().fg(colorscheme.preview.title_fg).bold(),
    ));
    preview_title_spans.push(Span::from(" "));
    let preview_outer_block = Block::default()
        .title_top(
            Line::from(preview_title_spans)
                .alignment(Alignment::Center)
                .style(Style::default().fg(colorscheme.preview.title_fg)),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(Padding::new(0, 1, 1, 0));

    let preview_inner_block =
        Block::default().style(Style::default()).padding(Padding {
            top: 0,
            right: 1,
            bottom: 0,
            left: 1,
        });
    let inner = preview_outer_block.inner(rect);
    f.render_widget(preview_outer_block, rect);

    let target_line = entry.line_number.map(|l| u16::try_from(l).unwrap_or(0));
    let cache_key = compute_cache_key(entry);

    // Check if the rendered preview content is already in the cache
    if let Some(preview_paragraph) =
        rendered_preview_cache.lock().unwrap().get(&cache_key)
    {
        let p = preview_paragraph.as_ref().clone();
        f.render_widget(p.scroll((preview_scroll, 0)), inner);
        return Ok(());
    }
    // If not, render the preview content and cache it if not empty
    let c_scheme = colorscheme.clone();
    let rp = build_preview_paragraph(
        preview_inner_block,
        inner,
        preview.content.clone(),
        target_line,
        preview_scroll,
        c_scheme,
    );
    if !preview.stale {
        rendered_preview_cache
            .lock()
            .unwrap()
            .insert(cache_key, &Arc::new(rp.clone()));
    }
    f.render_widget(
        Arc::new(rp).as_ref().clone().scroll((preview_scroll, 0)),
        inner,
    );
    Ok(())
}

fn build_line_number_span<'a>(line_number: usize) -> Span<'a> {
    Span::from(format!("{line_number:5} "))
}

fn compute_paragraph_from_highlighted_lines(
    highlighted_lines: &[Vec<(syntect::highlighting::Style, String)>],
    line_specifier: Option<usize>,
    colorscheme: PreviewColorscheme,
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
                        colorscheme.gutter_selected_fg
                    } else {
                        colorscheme.gutter_fg
                    },
                ));
            Line::from_iter(
                std::iter::once(line_number)
                    .chain(std::iter::once(Span::styled(
                        " │ ",
                        Style::default().fg(colorscheme.gutter_fg).dim(),
                    )))
                    .chain(l.iter().cloned().map(|sr| {
                        convert_syn_region_to_span(
                            &(sr.0, sr.1),
                            if line_specifier.is_some()
                                && i == line_specifier
                                    .unwrap()
                                    .saturating_sub(1)
                            {
                                Some(colorscheme.highlight_bg)
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
    background: Option<Color>,
) -> Span<'a> {
    let mut style = Style::default()
        .fg(convert_syn_color_to_ratatui_color(syn_region.0.foreground));
    if let Some(background) = background {
        style = style.bg(background);
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
