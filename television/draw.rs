use std::{hash::Hash, time::Instant};

use anyhow::Result;
use ratatui::{layout::Rect, Frame};
use rustc_hash::FxHashSet;

use crate::{
    action::Action,
    channels::entry::Entry,
    config::Config,
    picker::Picker,
    preview::PreviewState,
    screen::{
        colors::Colorscheme, help::draw_help_bar, input::draw_input_box,
        keybindings::build_keybindings_table, layout::Layout,
        preview::draw_preview_content_block,
        remote_control::draw_remote_control, results::draw_results_list,
        spinner::Spinner,
    },
    television::Mode,
    utils::metadata::AppMetadata,
};

#[derive(Debug, Clone, PartialEq)]
/// The state of the current television channel.
///
/// This struct is passed along to the UI thread as part of the `TvState` struct.
pub struct ChannelState {
    pub current_channel_name: String,
    pub selected_entries: FxHashSet<Entry>,
    pub total_count: u32,
    pub running: bool,
}

impl ChannelState {
    pub fn new(
        current_channel_name: String,
        selected_entries: FxHashSet<Entry>,
        total_count: u32,
        running: bool,
    ) -> Self {
        Self {
            current_channel_name,
            selected_entries,
            total_count,
            running,
        }
    }
}

impl Hash for ChannelState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.current_channel_name.hash(state);
        self.selected_entries
            .iter()
            .for_each(|entry| entry.hash(state));
        self.total_count.hash(state);
        self.running.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
/// The state of the main thread `Television` struct.
///
/// This struct is passed along to the UI thread as part of the `Ctx` struct.
pub struct TvState {
    pub mode: Mode,
    pub selected_entry: Option<Entry>,
    pub results_picker: Picker,
    pub rc_picker: Picker,
    pub channel_state: ChannelState,
    pub spinner: Spinner,
    pub preview_state: PreviewState,
}

impl TvState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mode: Mode,
        selected_entry: Option<Entry>,
        results_picker: Picker,
        rc_picker: Picker,
        channel_state: ChannelState,
        spinner: Spinner,
        preview_state: PreviewState,
    ) -> Self {
        Self {
            mode,
            selected_entry,
            results_picker,
            rc_picker,
            channel_state,
            spinner,
            preview_state,
        }
    }
}

#[derive(Debug, Clone)]
/// A drawing context that holds the current state of the application.
///
/// This is used as a message passing object between the main thread
/// and the UI thread and should contain all the information needed to
/// draw a frame.
pub struct Ctx {
    pub tv_state: TvState,
    pub config: Config,
    pub colorscheme: Colorscheme,
    pub app_metadata: AppMetadata,
    pub instant: Instant,
    pub layout: Layout,
}

impl Ctx {
    pub fn new(
        tv_state: TvState,
        config: Config,
        colorscheme: Colorscheme,
        app_metadata: AppMetadata,
        instant: Instant,
        layout: Layout,
    ) -> Self {
        Self {
            tv_state,
            config,
            colorscheme,
            app_metadata,
            instant,
            layout,
        }
    }
}

impl PartialEq for Ctx {
    fn eq(&self, other: &Self) -> bool {
        self.tv_state == other.tv_state
            && self.config == other.config
            && self.colorscheme == other.colorscheme
            && self.app_metadata == other.app_metadata
    }
}

impl Eq for Ctx {}

impl Hash for Ctx {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tv_state.hash(state);
        self.config.hash(state);
        self.colorscheme.hash(state);
        self.app_metadata.hash(state);
    }
}

impl PartialOrd for Ctx {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.instant.cmp(&other.instant))
    }
}

impl Ord for Ctx {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.instant.cmp(&other.instant)
    }
}

/// Draw the current UI frame based on the given context.
///
/// This function is responsible for drawing the entire UI frame based on the given context by
/// ultimately flushing buffers down to the underlying terminal.
///
/// This function is executed by the UI thread whenever it receives a render message from the main
/// thread.
///
/// It will draw the help bar, the results list, the input box, the preview content block, and the
/// remote control.
///
/// # Returns
/// A `Result` containing the layout of the current frame if the drawing was successful.
/// This layout can then be sent back to the main thread to serve for tasks where having that
/// information can be useful or lead to optimizations.
pub fn draw(ctx: &Ctx, f: &mut Frame<'_>, area: Rect) -> Result<Layout> {
    let show_preview =
        ctx.config.ui.show_preview_panel && ctx.tv_state.preview_state.enabled;
    let show_remote = !matches!(ctx.tv_state.mode, Mode::Channel);

    let layout =
        Layout::build(area, &ctx.config.ui, show_remote, show_preview);

    // help bar (metadata, keymaps, logo)
    draw_help_bar(
        f,
        &layout.help_bar,
        &ctx.tv_state.channel_state.current_channel_name,
        build_keybindings_table(
            &ctx.config.keybindings.to_displayable(),
            ctx.tv_state.mode,
            &ctx.colorscheme,
        ),
        ctx.tv_state.mode,
        &ctx.app_metadata,
        &ctx.colorscheme,
    );

    // results list
    draw_results_list(
        f,
        layout.results,
        &ctx.tv_state.results_picker.entries,
        &ctx.tv_state.channel_state.selected_entries,
        &mut ctx.tv_state.results_picker.relative_state.clone(),
        ctx.config.ui.input_bar_position,
        ctx.config.ui.use_nerd_font_icons,
        &ctx.colorscheme,
        &ctx.config
            .keybindings
            .get(&Action::ToggleHelp)
            // just display the first keybinding
            .unwrap()
            .to_string(),
        &ctx.config
            .keybindings
            .get(&Action::TogglePreview)
            // just display the first keybinding
            .unwrap()
            .to_string(),
        // only show the preview keybinding hint if there's actually something to preview
        ctx.tv_state.preview_state.enabled,
        ctx.config.ui.no_help,
    )?;

    // input box
    draw_input_box(
        f,
        layout.input,
        ctx.tv_state.results_picker.total_items,
        ctx.tv_state.channel_state.total_count,
        &ctx.tv_state.results_picker.input,
        &ctx.tv_state.results_picker.state,
        ctx.tv_state.channel_state.running,
        &ctx.tv_state.channel_state.current_channel_name,
        &ctx.tv_state.spinner,
        &ctx.colorscheme,
        &ctx.config.ui.custom_header,
    )?;

    if layout.preview_window.is_some() {
        draw_preview_content_block(
            f,
            layout.preview_window.unwrap(),
            &ctx.tv_state.preview_state,
            ctx.config.ui.use_nerd_font_icons,
            &ctx.colorscheme,
        )?;
    }

    // remote control
    if show_remote {
        draw_remote_control(
            f,
            layout.remote_control.unwrap(),
            &ctx.tv_state.rc_picker.entries,
            ctx.config.ui.use_nerd_font_icons,
            &mut ctx.tv_state.rc_picker.state.clone(),
            &mut ctx.tv_state.rc_picker.input.clone(),
            &ctx.tv_state.mode,
            &ctx.colorscheme,
        )?;
    }

    Ok(layout)
}
