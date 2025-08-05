use crate::{
    channels::{entry::Entry, remote_control::CableEntry},
    config::layers::MergedConfig,
    picker::Picker,
    previewer::state::PreviewState,
    screen::{
        colors::Colorscheme, help_panel::draw_help_panel,
        input::draw_input_box, layout::Layout,
        preview::draw_preview_content_block,
        remote_control::draw_remote_control, results::draw_results_list,
        spinner::Spinner, status_bar,
    },
    television::Mode,
    utils::metadata::AppMetadata,
};
use anyhow::Result;
use ratatui::{Frame, layout::Rect};
use rustc_hash::FxHashSet;
use std::{hash::Hash, time::Instant};

#[derive(Debug, Clone, PartialEq)]
/// The state of the current television channel.
///
/// This struct is passed along to the UI thread as part of the `TvState` struct.
pub struct ChannelState {
    pub current_channel_name: String,
    pub selected_entries: FxHashSet<Entry>,
    pub total_count: u32,
    pub running: bool,
    pub current_command: String,
}

impl ChannelState {
    pub fn new(
        current_channel_name: String,
        selected_entries: FxHashSet<Entry>,
        total_count: u32,
        running: bool,
        current_command: String,
    ) -> Self {
        Self {
            current_channel_name,
            selected_entries,
            total_count,
            running,
            current_command,
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
        self.current_command.hash(state);
    }
}

#[derive(Debug, Clone)]
/// The state of the main thread `Television` struct.
///
/// This struct is passed along to the UI thread as part of the `Ctx` struct.
pub struct TvState {
    pub mode: Mode,
    pub selected_entry: Option<Entry>,
    pub results_picker: Picker<Entry>,
    pub rc_picker: Picker<CableEntry>,
    pub channel_state: ChannelState,
    pub spinner: Spinner,
    pub preview_state: PreviewState,
}

impl TvState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mode: Mode,
        selected_entry: Option<Entry>,
        results_picker: Picker<Entry>,
        rc_picker: Picker<CableEntry>,
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
    pub config: MergedConfig,
    pub colorscheme: Colorscheme,
    pub app_metadata: AppMetadata,
    pub instant: Instant,
    pub layout: Layout,
}

impl Ctx {
    pub fn new(
        tv_state: TvState,
        config: MergedConfig,
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

/// Trait implemented by every drawable UI component.
pub trait UiComponent {
    /// Draw the component inside the given area.
    fn draw(&self, f: &mut Frame<'_>, area: Rect);
}

/// Wrapper around the existing `status_bar` drawing logic so it can be treated as a `UiComponent`.
pub struct StatusBarComponent<'a> {
    pub ctx: &'a Ctx,
}

impl<'a> StatusBarComponent<'a> {
    pub fn new(ctx: &'a Ctx) -> Self {
        Self { ctx }
    }
}

impl UiComponent for StatusBarComponent<'_> {
    fn draw(&self, f: &mut Frame<'_>, area: Rect) {
        status_bar::draw_status_bar(f, area, self.ctx);
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
/// It will draw the results list, the input box, the preview content block, the remote control,
/// the help panel, and the status bar.
///
/// # Returns
/// A `Result` containing the layout of the current frame if the drawing was successful.
/// This layout can then be sent back to the main thread to serve for tasks where having that
/// information can be useful or lead to optimizations.
pub fn draw(ctx: &Ctx, f: &mut Frame<'_>, area: Rect) -> Result<Layout> {
    let show_remote = matches!(ctx.tv_state.mode, Mode::RemoteControl);

    let layout =
        Layout::build(area, &ctx.config, ctx.tv_state.mode, &ctx.colorscheme);

    // results list
    draw_results_list(
        f,
        layout.results,
        &ctx.tv_state.results_picker.entries,
        &ctx.tv_state.channel_state.selected_entries,
        &mut ctx.tv_state.results_picker.relative_state.clone(),
        ctx.config.input_bar_position,
        &ctx.colorscheme,
        &ctx.config.results_panel_padding,
        &ctx.config.results_panel_border_type,
    )?;

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
        ctx.config.input_bar_position,
        &ctx.config.input_bar_header,
        &ctx.config.input_bar_padding,
        &ctx.config.input_bar_border_type,
        ctx.config.input_bar_prompt.as_ref(),
    )?;

    // status bar at the bottom
    if let Some(status_bar_area) = layout.status_bar {
        let status_component = StatusBarComponent::new(ctx);
        status_component.draw(f, status_bar_area);
    }

    if let Some(preview_rect) = layout.preview_window {
        draw_preview_content_block(
            f,
            preview_rect,
            &ctx.tv_state.preview_state,
            &ctx.colorscheme,
            &ctx.config.preview_panel_border_type,
            &ctx.config.preview_panel_padding,
            ctx.config.preview_panel_scrollbar,
        )?;
    }

    // remote control
    if show_remote {
        draw_remote_control(
            f,
            layout.remote_control.unwrap(),
            &ctx.tv_state.rc_picker.entries,
            &mut ctx.tv_state.rc_picker.state.clone(),
            &mut ctx.tv_state.rc_picker.input.clone(),
            &ctx.colorscheme,
            ctx.config.remote_show_channel_descriptions,
        )?;
    }

    // floating help panel (rendered last to appear on top)
    if let Some(help_area) = layout.help_panel {
        draw_help_panel(
            f,
            help_area,
            &ctx.config,
            ctx.tv_state.mode,
            &ctx.colorscheme,
        );
    }

    Ok(layout)
}
