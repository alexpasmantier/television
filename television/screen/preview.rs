use crate::{
    config::ui::{BorderType, Padding},
    event::Key,
    previewer::{PreviewContent, state::PreviewState},
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
use ratatui_image::{Resize, StatefulImage, picker::{Picker, ProtocolType}, protocol::StatefulProtocol};
use std::sync::OnceLock;
use parking_lot::Mutex;

/// Global picker instance, initialized once on first use.
static IMAGE_PICKER: OnceLock<Mutex<Picker>> = OnceLock::new();

/// Build the image picker from ioctl font-size and env-based protocol
/// detection.  No escape-sequence queries, so nothing can leak.
fn build_picker() -> Picker {
    // 1. Get actual font size from ioctl (TIOCGWINSZ)
    let font_size = font_size_from_ioctl().unwrap_or((16, 32));

    // 2. Detect protocol from environment variables
    let proto = detect_protocol_from_env();

    tracing::info!(
        "Image picker: font_size={:?}, detected_proto={:?}",
        font_size,
        proto
    );

    #[allow(deprecated)]
    let mut picker = Picker::from_fontsize(font_size);

    if let Some(proto) = proto {
        picker.set_protocol_type(proto);
    }

    tracing::info!(
        "Image picker: final protocol={:?}, font_size={:?}",
        picker.protocol_type(),
        picker.font_size()
    );
    picker
}

/// Get the terminal cell size in pixels via TIOCGWINSZ ioctl.
/// Returns (width, height) per cell.
#[cfg(unix)]
fn font_size_from_ioctl() -> Option<(u16, u16)> {
    let winsize = rustix::termios::tcgetwinsize(std::io::stdout()).ok()?;
    let (x, y, cols, rows) = (
        winsize.ws_xpixel,
        winsize.ws_ypixel,
        winsize.ws_col,
        winsize.ws_row,
    );
    if x == 0 || y == 0 || cols == 0 || rows == 0 {
        return None;
    }
    Some((x / cols, y / rows))
}

#[cfg(not(unix))]
fn font_size_from_ioctl() -> Option<(u16, u16)> {
    None
}

/// Detect the best image protocol from environment variables.
fn detect_protocol_from_env() -> Option<ProtocolType> {
    // Kitty: check KITTY_WINDOW_ID (works both in and outside tmux)
    if std::env::var("KITTY_WINDOW_ID").is_ok_and(|s| !s.is_empty()) {
        return Some(ProtocolType::Kitty);
    }
    // iTerm2 / WezTerm / mintty
    if std::env::var("TERM_PROGRAM").is_ok_and(|tp| {
        tp.contains("iTerm")
            || tp.contains("WezTerm")
            || tp.contains("mintty")
    }) {
        return Some(ProtocolType::Iterm2);
    }
    if std::env::var("LC_TERMINAL").is_ok_and(|lc| lc.contains("iTerm")) {
        return Some(ProtocolType::Iterm2);
    }
    None
}

fn get_picker() -> &'static Mutex<Picker> {
    IMAGE_PICKER.get_or_init(|| Mutex::new(build_picker()))
}

/// Per-preview image protocol state, cached so we don't re-encode every frame.
struct ImageState {
    protocol: StatefulProtocol,
    entry_raw: String,
}

use std::cell::RefCell;

thread_local! {
    static CACHED_IMAGE_STATE: RefCell<Option<ImageState>> = const { RefCell::new(None) };
}

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
    );

    match preview_state.preview.content {
        PreviewContent::Image(ref dyn_image) => {
            f.render_widget(Clear, inner);

            CACHED_IMAGE_STATE.with(|cell| {
                let mut cached = cell.borrow_mut();

                // Re-create protocol state if the entry changed
                let needs_update = cached
                    .as_ref()
                    .is_none_or(|s| s.entry_raw != preview_state.preview.entry_raw);

                if needs_update {
                    let picker = get_picker();
                    let protocol = picker.lock().new_resize_protocol((**dyn_image).clone());
                    *cached = Some(ImageState {
                        protocol,
                        entry_raw: preview_state.preview.entry_raw.clone(),
                    });
                }

                if let Some(state) = cached.as_mut() {
                    let image_widget = StatefulImage::default()
                        .resize(Resize::Fit(Some(image::imageops::FilterType::CatmullRom)));
                    f.render_stateful_widget(image_widget, inner, &mut state.protocol);
                }
            });
        }
        PreviewContent::Text(text) => {
            let total_lines =
                preview_state.preview.total_lines.saturating_sub(1) as usize;
            let scroll = preview_state.scroll;

            // render the preview content
            let rp = build_preview_paragraph(
                text,
                preview_state.preview.target_line,
                colorscheme.preview.highlight_bg,
                word_wrap,
            );
            f.render_widget(Clear, inner);
            f.render_widget(rp, inner);

            // render scrollbar if enabled
            if scrollbar {
                let scrollbar_widget =
                    Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .style(
                            Style::default()
                                .fg(colorscheme.general.border_fg),
                        );

                let mut scrollbar_state =
                    ScrollbarState::new(total_lines).position(scroll as usize);

                // Create a separate area for the scrollbar that accounts for text padding
                let scrollbar_rect = Rect {
                    x: inner.x + inner.width,
                    y: inner.y,
                    width: 1, // Scrollbar width
                    height: inner.height,
                };

                scrollbar_widget.render(
                    scrollbar_rect,
                    f.buffer_mut(),
                    &mut scrollbar_state,
                );
            }
        }
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

    let mut block = Block::default();
    block = block.title_top(
        Line::from(preview_title_spans)
            .alignment(Alignment::Center)
            .style(Style::default().fg(colorscheme.preview.title_fg)),
    );

    // preview footer
    if let Some(preview_footer) = preview_footer {
        let footer_line = Line::from(vec![
            Span::from(SPACE),
            Span::from(preview_footer),
            Span::from(SPACE),
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
