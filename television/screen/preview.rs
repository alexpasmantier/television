use crate::preview::PreviewState;
use crate::preview::{
    ansi::IntoText, PreviewContent, LOADING_MSG, TIMEOUT_MSG,
};
use crate::screen::colors::{Colorscheme, PreviewColorscheme};
use crate::utils::strings::{
    replace_non_printable, shrink_with_ellipsis, ReplaceNonPrintableConfig,
    EMPTY_STRING,
};
use anyhow::Result;
use devicons::FileIcon;
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap};
use ratatui::Frame;
use ratatui::{
    layout::{Alignment, Rect},
    prelude::{Color, Line, Modifier, Span, Style, Stylize, Text},
};
use std::str::FromStr;

#[allow(dead_code)]
const FILL_CHAR_SLANTED: char = '╱';
const FILL_CHAR_EMPTY: char = ' ';

#[allow(clippy::too_many_arguments)]
pub fn draw_preview_content_block(
    f: &mut Frame,
    rect: Rect,
    preview_state: &PreviewState,
    use_nerd_font_icons: bool,
    colorscheme: &Colorscheme,
) -> Result<()> {
    let inner = draw_content_outer_block(
        f,
        rect,
        colorscheme,
        preview_state.preview.icon,
        &preview_state.preview.title,
        use_nerd_font_icons,
    )?;
    // render the preview content
    let rp = build_preview_paragraph(
        inner,
        &preview_state.preview.content,
        preview_state.target_line,
        preview_state.scroll,
    );
    f.render_widget(rp, inner);

    Ok(())
}

pub fn build_preview_paragraph<'a>(
    inner: Rect,
    preview_content: &'a PreviewContent,
    target_line: Option<u16>,
    preview_scroll: u16,
) -> Paragraph<'a> {
    let preview_block =
        Block::default().style(Style::default()).padding(Padding {
            top: 0,
            right: 1,
            bottom: 0,
            left: 1,
        });

    match preview_content {
        PreviewContent::AnsiText(text) => {
            build_ansi_text_paragraph(text, preview_block, preview_scroll)
        }
        // meta
        PreviewContent::Loading => {
            build_meta_preview_paragraph(inner, LOADING_MSG, FILL_CHAR_EMPTY)
                .block(preview_block)
                .alignment(Alignment::Left)
                .style(Style::default().add_modifier(Modifier::ITALIC))
        }
        PreviewContent::Timeout => {
            build_meta_preview_paragraph(inner, TIMEOUT_MSG, FILL_CHAR_EMPTY)
                .block(preview_block)
                .alignment(Alignment::Left)
                .style(Style::default().add_modifier(Modifier::ITALIC))
        }
        PreviewContent::Empty => Paragraph::new(Text::raw(EMPTY_STRING)),
    }
}

const ANSI_BEFORE_CONTEXT_SIZE: u16 = 10;
const ANSI_CONTEXT_SIZE: usize = 150;

fn build_ansi_text_paragraph<'a>(
    text: &'a str,
    preview_block: Block<'a>,
    preview_scroll: u16,
) -> Paragraph<'a> {
    let lines = text.lines();
    let skip =
        preview_scroll.saturating_sub(ANSI_BEFORE_CONTEXT_SIZE) as usize;
    let context = lines
        .skip(skip)
        .take(ANSI_CONTEXT_SIZE)
        .collect::<Vec<_>>()
        .join("\n");

    let mut text = "\n".repeat(skip);
    text.push_str(
        &replace_non_printable(
            context.as_bytes(),
            &ReplaceNonPrintableConfig {
                replace_line_feed: false,
                replace_control_characters: false,
                ..Default::default()
            },
        )
        .0,
    );

    Paragraph::new(text.into_text().unwrap())
        .block(preview_block)
        .scroll((preview_scroll, 0))
}

fn build_plain_text_paragraph<'a>(
    text: &'a [String],
    preview_block: Block<'a>,
    target_line: Option<u16>,
    preview_scroll: u16,
    colorscheme: PreviewColorscheme,
) -> Paragraph<'a> {
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

fn build_plain_text_wrapped_paragraph<'a>(
    text: &'a str,
    preview_block: Block<'a>,
    colorscheme: PreviewColorscheme,
) -> Paragraph<'a> {
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

fn draw_content_outer_block(
    f: &mut Frame,
    rect: Rect,
    colorscheme: &Colorscheme,
    icon: Option<FileIcon>,
    title: &str,
    use_nerd_font_icons: bool,
) -> Result<Rect> {
    let mut preview_title_spans = vec![Span::from(" ")];
    // optional icon
    if icon.is_some() && use_nerd_font_icons {
        let icon = icon.as_ref().unwrap();
        preview_title_spans.push(Span::styled(
            {
                let mut icon_str = String::from(icon.icon);
                icon_str.push(' ');
                icon_str
            },
            Style::default().fg(Color::from_str(icon.color)?),
        ));
    }
    // preview title
    preview_title_spans.push(Span::styled(
        shrink_with_ellipsis(
            &replace_non_printable(
                title.as_bytes(),
                &ReplaceNonPrintableConfig::default(),
            )
            .0,
            rect.width.saturating_sub(4) as usize,
        ),
        Style::default().fg(colorscheme.preview.title_fg).bold(),
    ));
    preview_title_spans.push(Span::from(" "));

    // build the preview block
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

    let inner = preview_outer_block.inner(rect);
    f.render_widget(preview_outer_block, rect);
    Ok(inner)
}

fn build_line_number_span<'a>(line_number: usize) -> Span<'a> {
    Span::from(format!("{line_number:5} "))
}
