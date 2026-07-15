use crate::{
    action::Action,
    channels::{
        action_picker::ActionEntry, entry::Entry, remote_control::CableEntry,
    },
    config::layers::MergedConfig,
    picker::Picker,
    previewer::state::PreviewState,
    screen::{
        action_picker::draw_minimal_actions_pane,
        colors::Colorscheme,
        help_panel::draw_help_pane,
        input::{SourceIndicator, draw_input_box},
        layout::{InputPosition, Layout, Orientation},
        missing_requirements_popup::draw_missing_requirements_popup,
        preview::draw_preview_content_block,
        results::{draw_minimal_picker_list, draw_results_list},
        status_bar,
    },
    television::{MissingRequirementsPopup, Mode},
    utils::metadata::AppMetadata,
};
use anyhow::Result;
use ratatui::{Frame, layout::Rect, widgets::Borders};
use rustc_hash::FxHashSet;
use std::{hash::Hash, sync::Arc, time::Instant};

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
    pub current_source_name: Option<String>,
    pub source_index: usize,
    pub source_count: usize,
}

impl ChannelState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        current_channel_name: String,
        selected_entries: FxHashSet<Entry>,
        total_count: u32,
        running: bool,
        current_command: String,
        current_source_name: Option<String>,
        source_index: usize,
        source_count: usize,
    ) -> Self {
        Self {
            current_channel_name,
            selected_entries,
            total_count,
            running,
            current_command,
            current_source_name,
            source_index,
            source_count,
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
        self.current_source_name.hash(state);
        self.source_index.hash(state);
        self.source_count.hash(state);
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
    pub ap_picker: Picker<ActionEntry>,
    pub channel_state: ChannelState,
    pub preview_state: PreviewState,
    pub missing_requirements_popup: Option<MissingRequirementsPopup>,
}

impl TvState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mode: Mode,
        selected_entry: Option<Entry>,
        results_picker: Picker<Entry>,
        rc_picker: Picker<CableEntry>,
        ap_picker: Picker<ActionEntry>,
        channel_state: ChannelState,
        preview_state: PreviewState,
        missing_requirements_popup: Option<MissingRequirementsPopup>,
    ) -> Self {
        Self {
            mode,
            selected_entry,
            results_picker,
            rc_picker,
            ap_picker,
            channel_state,
            preview_state,
            missing_requirements_popup,
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
    pub config: Arc<MergedConfig>,
    pub colorscheme: Arc<Colorscheme>,
    pub app_metadata: Arc<AppMetadata>,
    pub instant: Instant,
    pub layout: Layout,
}

impl Ctx {
    pub fn new(
        tv_state: TvState,
        config: Arc<MergedConfig>,
        colorscheme: Arc<Colorscheme>,
        app_metadata: Arc<AppMetadata>,
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
pub fn draw(ctx: Ctx, f: &mut Frame<'_>, area: Rect) -> Result<Layout> {
    let show_remote = matches!(ctx.tv_state.mode, Mode::RemoteControl);
    let show_action_picker = matches!(ctx.tv_state.mode, Mode::ActionPicker);
    let minimal = ctx.config.input_bar_minimal;

    let layout = Layout::build(area, &ctx.config, ctx.tv_state.mode);

    // the remote control takes over the main results and input areas
    if show_remote {
        let picker = &ctx.tv_state.rc_picker;
        draw_minimal_picker_list(
            f,
            layout.results,
            &picker.entries,
            &mut picker.relative_state.clone(),
            ctx.config.input_bar_position,
            &ctx.colorscheme,
            &ctx.config.results_panel_padding,
            ctx.config.remote_show_channel_descriptions,
        )?;
        draw_input_box(
            f,
            layout.input,
            picker.total_items,
            picker.total_count,
            &picker.input,
            &picker.state,
            false,
            "channels",
            &ctx.colorscheme,
            ctx.config.input_bar_position,
            &ctx.config.input_bar_header,
            &ctx.config.input_bar_padding,
            &ctx.config.input_bar_border_type,
            ctx.config.input_bar_prompt.as_ref(),
            minimal,
            // with no status bar, the picker hint stands in for the mode
            ctx.config
                .status_bar_hidden
                .then_some(("channels", ctx.colorscheme.mode.remote_control)),
            None,
        )?;
    } else {
        // results list
        let cycle_sources_key = ctx
            .config
            .input_map
            .get_key_for_action(&Action::CycleSources);
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
            ctx.tv_state.channel_state.source_index,
            ctx.tv_state.channel_state.source_count,
            ctx.tv_state.channel_state.current_source_name.as_deref(),
            cycle_sources_key,
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
            &ctx.colorscheme,
            ctx.config.input_bar_position,
            &ctx.config.input_bar_header,
            &ctx.config.input_bar_padding,
            &ctx.config.input_bar_border_type,
            ctx.config.input_bar_prompt.as_ref(),
            minimal,
            // with no status bar, the channel name moves next to the count,
            // in the same color the status bar would use
            (minimal && ctx.config.status_bar_hidden).then(|| {
                (
                    ctx.tv_state.channel_state.current_channel_name.as_str(),
                    ctx.colorscheme.results.result_fg,
                )
            }),
            Some(SourceIndicator {
                name: ctx
                    .tv_state
                    .channel_state
                    .current_source_name
                    .as_deref(),
                index: ctx.tv_state.channel_state.source_index,
                count: ctx.tv_state.channel_state.source_count,
            }),
        )?;
    }

    // status bar at the bottom
    if let Some(status_bar_area) = layout.status_bar {
        let status_component = StatusBarComponent::new(&ctx);
        status_component.draw(f, status_bar_area);
    }

    if let Some(preview_rect) = layout.preview_window {
        let cycle_previews_key = ctx
            .config
            .input_map
            .get_key_for_action(&Action::CyclePreviews);
        // when the minimal UI preset is active, draw a hairline on the side
        // of the preview that faces the results list
        let separator = if ctx.config.preview_panel_separator {
            Some(match (ctx.config.layout, ctx.config.input_bar_position) {
                // preview sits on the right
                (Orientation::Landscape, _) => Borders::LEFT,
                // preview sits at the bottom
                (Orientation::Portrait, InputPosition::Top) => Borders::TOP,
                // preview sits at the top
                (Orientation::Portrait, InputPosition::Bottom) => {
                    Borders::BOTTOM
                }
            })
        } else {
            None
        };
        draw_preview_content_block(
            f,
            preview_rect,
            ctx.tv_state.preview_state,
            &ctx.colorscheme,
            &ctx.config.preview_panel_border_type,
            &ctx.config.preview_panel_padding,
            ctx.config.preview_panel_scrollbar,
            ctx.config.preview_panel_word_wrap,
            cycle_previews_key,
            separator,
        )?;
    }

    // the actions picker borrows the preview pane, so the entry the action
    // applies to stays visible in the results list
    if show_action_picker && let Some(pane) = layout.action_picker {
        draw_minimal_actions_pane(
            f,
            pane,
            &ctx.tv_state.ap_picker.entries,
            &mut ctx.tv_state.ap_picker.relative_state.clone(),
            &ctx.tv_state.ap_picker.state,
            &ctx.tv_state.ap_picker.input,
            ctx.tv_state.ap_picker.total_items,
            ctx.tv_state.ap_picker.total_count,
            &ctx.config,
            &ctx.colorscheme,
        )?;
    }

    if let Some(popup) = &ctx.tv_state.missing_requirements_popup {
        draw_missing_requirements_popup(f, area, popup, &ctx.colorscheme);
    }

    // help panel in the borrowed preview pane
    if let Some(help_area) = layout.help_panel {
        draw_help_pane(
            f,
            help_area,
            &ctx.config,
            ctx.tv_state.mode,
            &ctx.colorscheme,
        );
    }

    Ok(layout)
}
