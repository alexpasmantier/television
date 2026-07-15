use crate::{
    config::ui::{BorderType, Padding},
    event::Key,
    previewer::state::PreviewState,
    screen::colors::Colorscheme,
    utils::strings::{
        ReplaceNonPrintableConfig, SPACE, replace_non_printable_bulk,
        shrink_with_ellipsis,
    },
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    prelude::{Color, Line, Span, Style, Text},
    widgets::{
        Block, Borders, Clear, Padding as RatatuiPadding, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
    },
};

#[allow(clippy::too_many_arguments)]
pub fn draw_preview_content_block(
    f: &mut Frame,
    rect: Rect,
    preview_state: PreviewState,
    colorscheme: &Colorscheme,
    border_type: &BorderType,
    padding: &Padding,
    scrollbar: bool,
    word_wrap: bool,
    cycle_key: Option<Key>,
    separator: Option<Borders>,
) -> Result<()> {
    let inner = draw_content_outer_block(
        f,
        rect,
        colorscheme,
        *border_type,
        *padding,
        &preview_state.preview.title,
        preview_state.preview.footer,
        preview_state.preview.preview_index,
        preview_state.preview.preview_count,
        cycle_key,
        separator,
    );
    let total_lines =
        preview_state.preview.total_lines.saturating_sub(1) as usize;
    let scroll = preview_state.scroll;

    // render the preview content
    let rp = build_preview_paragraph(
        preview_state.preview.content,
        preview_state.preview.target_line,
        colorscheme.preview.highlight_bg,
        word_wrap,
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
    mut text: Text<'static>,
    target_line: Option<u16>,
    highlight_bg: Color,
    word_wrap: bool,
) -> Paragraph<'static> {
    // Highlight the target line
    if let Some(target_line) = target_line
        && let Some(line) =
            text.lines.get_mut((target_line.saturating_sub(1)) as usize)
    {
        for span in &mut line.spans {
            span.style = span.style.bg(highlight_bg);
        }
    }

    let preview_block =
        Block::default()
            .style(Style::default())
            .padding(RatatuiPadding {
                top: 0,
                right: 1,
                bottom: 0,
                left: 1,
            });

    let paragraph = Paragraph::new(text).block(preview_block);

    if word_wrap {
        paragraph.wrap(ratatui::widgets::Wrap { trim: true })
    } else {
        paragraph
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_content_outer_block(
    f: &mut Frame,
    rect: Rect,
    colorscheme: &Colorscheme,
    border_type: BorderType,
    padding: Padding,
    preview_title: &str,
    preview_footer: Option<String>,
    preview_index: usize,
    preview_count: usize,
    cycle_key: Option<Key>,
    separator: Option<Borders>,
) -> Rect {
    let (indicator, key_hint) = if preview_count > 1 {
        let dots: String = (0..preview_count)
            .map(|i| if i == preview_index { "●" } else { "○" })
            .collect::<Vec<_>>()
            .join(" ");
        let hint = cycle_key.map(|k| format!(" {}", k)).unwrap_or_default();
        (format!(" ⟨ {} ⟩", dots), hint)
    } else {
        (String::new(), String::new())
    };
    let indicator_len = u16::try_from(indicator.chars().count()).unwrap_or(0);
    let key_hint_len = u16::try_from(key_hint.chars().count()).unwrap_or(0);

    let mut preview_title_spans = vec![Span::from(SPACE)];
    // preview header
    preview_title_spans.push(Span::styled(
        shrink_with_ellipsis(
            &replace_non_printable_bulk(
                preview_title,
                &ReplaceNonPrintableConfig::default(),
            )
            .0,
            rect.width.saturating_sub(4 + indicator_len + key_hint_len)
                as usize,
        ),
        Style::default().fg(colorscheme.preview.title_fg).bold(),
    ));

    if preview_count > 1 {
        preview_title_spans.push(Span::styled(
            indicator,
            Style::default().fg(colorscheme.input.results_count_fg),
        ));
        if !key_hint.is_empty() {
            preview_title_spans.push(Span::styled(
                key_hint,
                Style::default().fg(colorscheme.general.border_fg),
            ));
        }
    }
    preview_title_spans.push(Span::from(SPACE));

    // without a border to anchor them, titles read better left-aligned
    let title_alignment = if border_type.to_ratatui_border_type().is_some() {
        Alignment::Center
    } else {
        Alignment::Left
    };

    // ratatui draws titles on the border row: with a horizontal hairline the
    // title embeds into the line, so lead with a line segment instead of a
    // bare space (`─ title ───` rather than ` title ───`)
    let borderless = border_type.to_ratatui_border_type().is_none();
    let embeds_into = |side: Borders| {
        borderless && separator.is_some_and(|s| s.contains(side))
    };
    let hairline_style = Style::default().fg(colorscheme.general.border_fg);

    if embeds_into(Borders::TOP) {
        preview_title_spans.insert(0, Span::styled("─", hairline_style));
    }

    let mut block = Block::default().title_top(
        Line::from(preview_title_spans)
            .alignment(title_alignment)
            .style(Style::default().fg(colorscheme.preview.title_fg)),
    );

    // preview footer
    if let Some(preview_footer) = preview_footer {
        let mut footer_spans = vec![
            Span::from(SPACE),
            Span::from(preview_footer),
            Span::from(SPACE),
        ];
        if embeds_into(Borders::BOTTOM) {
            footer_spans.insert(0, Span::styled("─", hairline_style));
        }
        let footer_line = Line::from(footer_spans)
            .alignment(title_alignment)
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
    } else if let Some(separator) = separator {
        // borderless preview (minimal UI): a thin hairline on the side
        // facing the results provides just enough separation
        preview_outer_block = preview_outer_block
            .borders(separator)
            .border_set(crate::screen::constants::HAIRLINE_BORDER_SET)
            .border_style(hairline_style);
    }

    let inner = preview_outer_block.inner(rect);
    f.render_widget(preview_outer_block, rect);
    inner
}
