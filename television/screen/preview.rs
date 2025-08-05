use crate::{
    config::ui::{BorderType, Padding},
    previewer::{Preview, state::PreviewState},
    screen::colors::Colorscheme,
    utils::strings::{
        EMPTY_STRING, ReplaceNonPrintableConfig, replace_non_printable,
        shrink_with_ellipsis,
    },
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    prelude::{Color, Line, Span, Style, Stylize, Text},
    widgets::{
        Block, Borders, Clear, Padding as RatatuiPadding, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
    },
};

#[allow(clippy::too_many_arguments)]
pub fn draw_preview_content_block(
    f: &mut Frame,
    rect: Rect,
    preview_state: &PreviewState,
    colorscheme: &Colorscheme,
    border_type: &BorderType,
    padding: &Padding,
    scrollbar: bool,
) -> Result<()> {
    let inner = draw_content_outer_block(
        f,
        rect,
        colorscheme,
        *border_type,
        *padding,
        &preview_state.preview,
    );
    let total_lines =
        preview_state.preview.total_lines.saturating_sub(1) as usize;
    let scroll = preview_state.scroll;

    // render the preview content
    let rp = build_preview_paragraph(
        preview_state.preview.content.clone(),
        preview_state.preview.line_number,
        colorscheme.preview.highlight_bg,
    );
    f.render_widget(Clear, inner);
    f.render_widget(rp, inner);

    // render scrollbar if enabled
    if scrollbar {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(colorscheme.general.border_fg));

        let mut scrollbar_state =
            ScrollbarState::new(total_lines).position(scroll as usize);

        // Create a separate area for the scrollbar that accounts for text padding
        let scrollbar_rect = Rect {
            x: inner.x + inner.width,
            y: inner.y,
            width: 1, // Scrollbar width
            height: inner.height,
        };

        scrollbar.render(scrollbar_rect, f.buffer_mut(), &mut scrollbar_state);
    }

    Ok(())
}

pub fn build_preview_paragraph(
    content: Text<'static>,
    line_number: Option<u16>,
    highlight_bg: Color,
) -> Paragraph<'static> {
    let preview_block =
        Block::default()
            .style(Style::default())
            .padding(RatatuiPadding {
                top: 0,
                right: 1,
                bottom: 0,
                left: 1,
            });

    build_ansi_text_paragraph(
        content,
        preview_block,
        line_number,
        highlight_bg,
    )
}

fn build_ansi_text_paragraph<'a>(
    mut text: Text<'a>,
    preview_block: Block<'a>,
    target_line: Option<u16>,
    highlight_bg: Color,
) -> Paragraph<'a> {
    if let Some(target_line) = target_line {
        // Highlight the target line
        if let Some(line) = text.lines.get_mut((target_line - 1) as usize) {
            for span in &mut line.spans {
                span.style = span.style.bg(highlight_bg);
            }
        }
    }
    Paragraph::new(text).block(preview_block)
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
    border_type: BorderType,
    padding: Padding,
    preview: &Preview,
) -> Rect {
    let mut preview_title_spans = vec![Span::from(" ")];
    // preview header
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

    let mut block = Block::default();
    block = block.title_top(
        Line::from(preview_title_spans)
            .alignment(Alignment::Center)
            .style(Style::default().fg(colorscheme.preview.title_fg)),
    );

    // preview footer
    if !preview.footer.is_empty() {
        let footer_line = Line::from(vec![
            Span::from(" "),
            Span::from(preview.footer.as_str()),
            Span::from(" "),
        ])
        .alignment(Alignment::Center)
        .style(Style::default().fg(colorscheme.preview.title_fg));
        block = block.title_bottom(footer_line);
    }

    let mut preview_outer_block = block
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(RatatuiPadding::from(padding));
    if let Some(border_type) = border_type.to_ratatui_border_type() {
        preview_outer_block = preview_outer_block
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(colorscheme.general.border_fg));
    }

    let inner = preview_outer_block.inner(rect);
    f.render_widget(preview_outer_block, rect);
    inner
}
