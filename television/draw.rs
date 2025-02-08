use std::{hash::Hash, time::Instant};

use anyhow::Result;
use ratatui::{layout::Rect, Frame};
use rustc_hash::FxHashSet;
use tokio::sync::mpsc::Sender;

use crate::{
    action::Action,
    channels::entry::{Entry, PreviewType, ENTRY_PLACEHOLDER},
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
    television::{Message, Mode},
    utils::metadata::AppMetadata,
};

#[derive(Debug, Clone, PartialEq)]
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
pub struct TvState {
    pub mode: Mode,
    pub selected_entry: Option<Entry>,
    pub results_area_height: u16,
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
        results_area_height: u16,
        results_picker: Picker,
        rc_picker: Picker,
        channel_state: ChannelState,
        spinner: Spinner,
        preview_state: PreviewState,
    ) -> Self {
        Self {
            mode,
            selected_entry,
            results_area_height,
            results_picker,
            rc_picker,
            channel_state,
            spinner,
            preview_state,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ctx {
    pub tv_state: TvState,
    pub config: Config,
    pub colorscheme: Colorscheme,
    pub app_metadata: AppMetadata,
    pub tv_tx_handle: Sender<Message>,
    pub instant: Instant,
}

impl Ctx {
    pub fn new(
        tv_state: TvState,
        config: Config,
        colorscheme: Colorscheme,
        app_metadata: AppMetadata,
        tv_tx_handle: Sender<Message>,
        instant: Instant,
    ) -> Self {
        Self {
            tv_state,
            config,
            colorscheme,
            app_metadata,
            tv_tx_handle,
            instant,
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

pub fn draw(ctx: &Ctx, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    let selected_entry = ctx
        .tv_state
        .selected_entry
        .clone()
        .unwrap_or(ENTRY_PLACEHOLDER);

    let show_preview = ctx.config.ui.show_preview_panel
        && !matches!(selected_entry.preview_type, PreviewType::None);
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

    if layout.results.height.saturating_sub(2)
        != ctx.tv_state.results_area_height
    {
        ctx.tv_tx_handle.try_send(Message::ResultListHeightChanged(
            layout.results.height.saturating_sub(2),
        ))?;
    }

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
            .get(&ctx.tv_state.mode)
            .unwrap()
            .get(&Action::ToggleHelp)
            // just display the first keybinding
            .unwrap()
            .to_string(),
        &ctx.config
            .keybindings
            .get(&ctx.tv_state.mode)
            .unwrap()
            .get(&Action::TogglePreview)
            // just display the first keybinding
            .unwrap()
            .to_string(),
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
    )?;

    if show_preview {
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

    Ok(())
}
