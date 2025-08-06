use std::path::PathBuf;

use rustc_hash::FxHashMap;

use crate::{
    channels::prototypes::{
        ActionSpec, BinaryRequirement, ChannelPrototype, CommandSpec, Template,
    },
    cli::{ChannelCli, GlobalCli, PostProcessedCli},
    config::{
        Config,
        ui::{BorderType, Padding, ThemeOverrides},
    },
    keymap::InputMap,
    screen::layout::{InputPosition, Orientation},
};

pub struct LayeredConfig {
    /// The base configuration that is loaded from the config file.
    base_config: Config,
    /// The channel prototype that is currently loaded.
    channel: ChannelPrototype,
    /// The CLI configuration for the channel.
    ///
    /// This contains the channel-specific CLI options that were processed
    /// for the current channel.
    channel_cli: ChannelCli,
    /// The global CLI configuration options that will persist across channels.
    global_cli: GlobalCli,
}

impl LayeredConfig {
    pub fn new(
        base_config: Config,
        channel: ChannelPrototype,
        cli: PostProcessedCli,
    ) -> Self {
        Self {
            base_config,
            channel,
            channel_cli: cli.channel,
            global_cli: cli.global,
        }
    }

    pub fn update_channel(&mut self, channel: ChannelPrototype) {
        self.channel = channel;
        // Reset channel CLI options to defaults
        self.channel_cli = ChannelCli::default();
    }

    pub fn merge(&self) -> MergedConfig {
        // CLI-only fields
        let config_file = self.global_cli.config_file.clone();
        let working_directory = self.global_cli.workdir.clone();
        let autocomplete_prompt = self.channel_cli.autocomplete_prompt.clone();
        let input = self.channel_cli.input.clone();
        let exact_match = self.channel_cli.exact;
        let select_1 = self.channel_cli.select_1;
        let take_1 = self.channel_cli.take_1;
        let take_1_fast = self.channel_cli.take_1_fast;
        let inline = self.global_cli.inline;
        let height = self.global_cli.height;
        let width = self.global_cli.width;

        // base config only fields
        let data_dir = self.base_config.application.data_dir.clone();
        let default_channel =
            self.base_config.application.default_channel.clone();
        let history_size = self.base_config.application.history_size;
        let theme = self.base_config.ui.theme.clone();
        let shell_integration_commands =
            self.base_config.shell_integration.commands.clone();
        let shell_integration_fallback_channel =
            self.base_config.shell_integration.fallback_channel.clone();

        // channel only fields
        let channel_description = self.channel.metadata.description.clone();
        let channel_requirements = self.channel.metadata.requirements.clone();
        let channel_actions = self.channel.actions.clone();

        // CLI > base config fields
        let cable_dir = self
            .global_cli
            .cable_dir
            .as_ref()
            .unwrap_or(&self.base_config.application.cable_dir)
            .clone();
        let tick_rate = self
            .global_cli
            .tick_rate
            .unwrap_or(self.base_config.application.tick_rate);

        // CLI > channel fields
        let watch = self
            .channel_cli
            .watch_interval
            .unwrap_or(self.channel.watch);
        let channel_name = self
            .channel_cli
            .channel
            .as_ref()
            .unwrap_or(&self.channel.metadata.name)
            .clone();
        let channel_source_command =
            if let Some(template) = &self.channel_cli.source_command {
                CommandSpec::from_template(template.clone())
            } else {
                self.channel.source.command.clone()
            };
        let channel_source_entry_delimiter = self
            .channel_cli
            .source_entry_delimiter
            .or(self.channel.source.entry_delimiter);
        let channel_source_ansi =
            self.channel_cli.ansi || self.channel.source.ansi;
        let channel_source_display = self
            .channel_cli
            .source_display
            .as_ref()
            .or(self.channel.source.display.as_ref())
            .cloned();
        let channel_source_output = self
            .channel_cli
            .source_output
            .as_ref()
            .or(self.channel.source.output.as_ref())
            .cloned();
        let channel_preview_command = self
            .channel_cli
            .preview_command
            .as_ref()
            .map(|t| CommandSpec::from_template(t.clone()))
            .or(self.channel.preview.as_ref().map(|p| p.command.clone()));
        let channel_preview_offset =
            self.channel_cli.preview_offset.clone().or(
                if let Some(preview) = &self.channel.preview {
                    preview.offset.clone()
                } else {
                    None
                },
            );
        let channel_preview_cached = self.channel_cli.cache_preview
            || self.channel.preview.as_ref().is_some_and(|p| p.cached);

        // Channel > base config fields
        let remote_show_channel_descriptions = self
            .channel
            .ui
            .as_ref()
            .and_then(|ui| ui.remote_control.as_ref())
            .is_none_or(|rc| {
                rc.show_channel_descriptions
                    && self
                        .base_config
                        .ui
                        .remote_control
                        .show_channel_descriptions
            });
        let remote_sort_alphabetically = self
            .channel
            .ui
            .as_ref()
            .and_then(|ui| ui.remote_control.as_ref())
            .is_none_or(|rc| {
                rc.sort_alphabetically
                    && self.base_config.ui.remote_control.sort_alphabetically
            });
        let theme_overrides = self
            .channel
            .ui
            .as_ref()
            .map(|ui| ui.theme_overrides.clone())
            .unwrap_or_default()
            .merge(self.base_config.ui.theme_overrides.clone());

        let help_panel_show_categories = self
            .channel
            .ui
            .as_ref()
            .and_then(|ui| ui.help_panel.as_ref())
            .is_none_or(|hp| hp.show_categories)
            && self.base_config.ui.help_panel.show_categories;
        let status_bar_separator_open = self
            .channel
            .ui
            .as_ref()
            .and_then(|ui| ui.status_bar.as_ref())
            .map_or_else(
                || self.base_config.ui.status_bar.separator_open.clone(),
                |sb| sb.separator_open.clone(),
            );
        let status_bar_separator_close = self
            .channel
            .ui
            .as_ref()
            .and_then(|ui| ui.status_bar.as_ref())
            .map_or_else(
                || self.base_config.ui.status_bar.separator_close.clone(),
                |sb| sb.separator_close.clone(),
            );
        let input_bar_position = self
            .channel
            .ui
            .as_ref()
            .and_then(|ui| ui.input_bar.as_ref())
            .map_or(self.base_config.ui.input_bar.position, |ib| ib.position);

        // CLI > channel > base config fields
        let ui_scale = self.channel_cli.ui_scale.unwrap_or(
            self.channel
                .ui
                .as_ref()
                .and_then(|ui| ui.ui_scale)
                .unwrap_or(self.base_config.ui.ui_scale),
        );
        let layout = self.channel_cli.layout.unwrap_or(
            self.channel
                .ui
                .as_ref()
                .and_then(|ui| ui.orientation)
                .unwrap_or(self.base_config.ui.orientation),
        );
        let input_bar_header = self
            .channel_cli
            .input_header
            .clone()
            .or_else(|| {
                self.channel.ui.as_ref()?.input_bar.as_ref()?.header.clone()
            })
            .or_else(|| self.base_config.ui.input_bar.header.clone());
        let input_bar_prompt = self
            .channel_cli
            .input_prompt
            .clone()
            .or_else(|| {
                self.channel.ui.as_ref()?.input_bar.as_ref()?.prompt.clone()
            })
            .or_else(|| self.base_config.ui.input_bar.prompt.clone());
        let input_bar_border_type = self
            .channel_cli
            .input_border
            .or_else(|| {
                Some(self.channel.ui.as_ref()?.input_bar.as_ref()?.border_type)
            })
            .unwrap_or(self.base_config.ui.input_bar.border_type);
        let input_bar_padding = self
            .channel_cli
            .input_padding
            .or_else(|| {
                Some(self.channel.ui.as_ref()?.input_bar.as_ref()?.padding)
            })
            .unwrap_or(self.base_config.ui.input_bar.padding);
        let status_bar_disabled = self.global_cli.no_status_bar;
        let status_bar_hidden = if status_bar_disabled {
            true // --no-status-bar always wins
        } else if self.channel_cli.show_status_bar {
            false // --show-status-bar forces visible
        } else {
            self.channel_cli.hide_status_bar
                || self
                    .channel
                    .ui
                    .as_ref()
                    .and_then(|ui| ui.status_bar.as_ref())
                    .is_some_and(|sb| sb.hidden)
                || self.base_config.ui.status_bar.hidden
        };
        let results_panel_border_type = self
            .channel_cli
            .results_border
            .or_else(|| {
                Some(
                    self.channel
                        .ui
                        .as_ref()?
                        .results_panel
                        .as_ref()?
                        .border_type,
                )
            })
            .unwrap_or(self.base_config.ui.results_panel.border_type);
        let results_panel_padding = self
            .channel_cli
            .results_padding
            .or_else(|| {
                Some(self.channel.ui.as_ref()?.results_panel.as_ref()?.padding)
            })
            .unwrap_or(self.base_config.ui.results_panel.padding);
        let preview_panel_size = self
            .channel_cli
            .preview_size
            .or_else(|| {
                Some(self.channel.ui.as_ref()?.preview_panel.as_ref()?.size)
            })
            .unwrap_or(self.base_config.ui.preview_panel.size);
        let preview_panel_header = self
            .channel_cli
            .preview_header
            .clone()
            .or_else(|| {
                Some(
                    self.channel
                        .ui
                        .as_ref()?
                        .preview_panel
                        .as_ref()?
                        .header
                        .as_ref()?
                        .clone(),
                )
            })
            .or_else(|| self.base_config.ui.preview_panel.header.clone());
        let preview_panel_footer = self
            .channel_cli
            .preview_footer
            .clone()
            .or_else(|| {
                self.channel
                    .ui
                    .as_ref()?
                    .preview_panel
                    .as_ref()?
                    .footer
                    .clone()
            })
            .or_else(|| self.base_config.ui.preview_panel.footer.clone());
        let preview_panel_scrollbar = !self.channel_cli.hide_preview_scrollbar
            && self
                .channel
                .ui
                .as_ref()
                .and_then(|ui| ui.preview_panel.as_ref())
                .map_or(self.base_config.ui.preview_panel.scrollbar, |pp| {
                    pp.scrollbar
                });
        let preview_panel_border_type = self
            .channel_cli
            .preview_border
            .or_else(|| {
                Some(
                    self.channel
                        .ui
                        .as_ref()?
                        .preview_panel
                        .as_ref()?
                        .border_type,
                )
            })
            .unwrap_or(self.base_config.ui.preview_panel.border_type);
        let preview_panel_padding = self
            .channel_cli
            .preview_padding
            .or_else(|| {
                Some(self.channel.ui.as_ref()?.preview_panel.as_ref()?.padding)
            })
            .unwrap_or(self.base_config.ui.preview_panel.padding);
        let preview_panel_disabled =
            self.global_cli.no_preview || self.channel_cli.no_preview;
        let preview_panel_hidden = if preview_panel_disabled {
            true // --no-preview always wins
        } else if self.channel_cli.show_preview {
            false // --show-preview forces visible
        } else {
            self.channel_cli.hide_preview
                || self
                    .channel
                    .ui
                    .as_ref()
                    .and_then(|ui| ui.preview_panel.as_ref())
                    .is_some_and(|pp| pp.hidden)
                || self.base_config.ui.preview_panel.hidden
        };
        let help_panel_hidden = if self.channel_cli.show_help_panel {
            false
        } else {
            self.channel_cli.hide_help_panel
                || self
                    .channel
                    .ui
                    .as_ref()
                    .and_then(|ui| ui.help_panel.as_ref())
                    .is_some_and(|hp| hp.hidden)
                || self.base_config.ui.help_panel.hidden
        };
        let help_panel_disabled = self.global_cli.no_help_panel
            || self.channel.ui.as_ref().is_some_and(|ui| {
                ui.help_panel.as_ref().is_some_and(|hp| hp.disabled)
            })
            || self.base_config.ui.help_panel.disabled;
        let remote_disabled = self.global_cli.no_remote
            || self
                .channel
                .ui
                .as_ref()
                .and_then(|ui| ui.remote_control.as_ref())
                .is_some_and(|rc| rc.disabled);
        let global_history = self.global_cli.global_history
            || self.channel.history.global_mode.unwrap_or_default()
            || self.base_config.application.global_history;
        let keybindings = {
            let mut merged_bindings = self.base_config.keybindings.clone();
            // Merge channel-specific keybindings
            if let Some(channel_bindings) = &self.channel.keybindings {
                merged_bindings
                    .extend(channel_bindings.bindings.inner.clone());
            }
            // Merge CLI keybindings
            if let Some(cli_bindings) = &self.channel_cli.keybindings {
                merged_bindings.extend(cli_bindings.inner.clone());
            }
            merged_bindings
        };
        let event_bindings = self.base_config.events.clone();

        MergedConfig {
            // General
            data_dir,
            config_file,
            cable_dir,
            tick_rate,
            default_channel,
            history_size,
            global_history,
            working_directory,
            autocomplete_prompt,
            // matcher configuration
            exact_match,
            select_1,
            take_1,
            take_1_fast,
            input,

            // Bindings
            input_map: InputMap::from((&keybindings, &event_bindings)),

            // UI
            ui_scale,
            layout,
            theme,
            inline,
            height,
            width,
            // input bar
            input_bar_position,
            input_bar_header,
            input_bar_prompt,
            input_bar_border_type,
            input_bar_padding,
            // status bar
            status_bar_separator_open,
            status_bar_separator_close,
            status_bar_hidden,
            status_bar_disabled,
            // results panel
            results_panel_border_type,
            results_panel_padding,
            // preview panel
            preview_panel_size,
            preview_panel_header,
            preview_panel_footer,
            preview_panel_scrollbar,
            preview_panel_border_type,
            preview_panel_padding,
            preview_panel_hidden,
            preview_panel_disabled,
            // help panel
            help_panel_show_categories,
            help_panel_hidden,
            help_panel_disabled,
            // remote control
            remote_show_channel_descriptions,
            remote_sort_alphabetically,
            remote_disabled,
            // theme overrides
            theme_overrides,

            // Shell integration
            shell_integration_commands,
            shell_integration_fallback_channel,

            // Channel-specific fields
            watch,

            // metadata
            channel_name,
            channel_description,
            channel_requirements,
            // source
            channel_source_command,
            channel_source_entry_delimiter,
            channel_source_ansi,
            channel_source_display,
            channel_source_output,
            // preview
            channel_preview_command,
            channel_preview_offset,
            channel_preview_cached,
            // actions
            channel_actions,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct MergedConfig {
    // General
    pub data_dir: PathBuf,
    pub config_file: Option<PathBuf>,
    pub cable_dir: PathBuf,
    pub tick_rate: u64,
    pub default_channel: String,
    pub history_size: usize,
    pub global_history: bool,
    pub working_directory: Option<PathBuf>,
    pub autocomplete_prompt: Option<String>,
    // matcher configuration
    pub exact_match: bool,
    pub select_1: bool,
    pub take_1: bool,
    pub take_1_fast: bool,
    pub input: Option<String>,

    // Bindings
    pub input_map: InputMap,

    // UI
    pub ui_scale: u16,
    pub layout: Orientation,
    pub theme: String,
    pub inline: bool,
    pub height: Option<u16>,
    pub width: Option<u16>,
    // input bar
    pub input_bar_position: InputPosition,
    pub input_bar_header: Option<String>,
    pub input_bar_prompt: Option<String>,
    pub input_bar_border_type: BorderType,
    pub input_bar_padding: Padding,
    // status bar
    pub status_bar_separator_open: String,
    pub status_bar_separator_close: String,
    pub status_bar_hidden: bool,
    pub status_bar_disabled: bool,
    // results panel
    pub results_panel_border_type: BorderType,
    pub results_panel_padding: Padding,
    // preview panel
    pub preview_panel_size: u16,
    pub preview_panel_header: Option<Template>,
    pub preview_panel_footer: Option<Template>,
    pub preview_panel_scrollbar: bool,
    pub preview_panel_border_type: BorderType,
    pub preview_panel_padding: Padding,
    pub preview_panel_hidden: bool,
    pub preview_panel_disabled: bool,
    // help panel
    pub help_panel_show_categories: bool,
    pub help_panel_hidden: bool,
    pub help_panel_disabled: bool,
    // remote control
    pub remote_show_channel_descriptions: bool,
    pub remote_sort_alphabetically: bool,
    pub remote_disabled: bool,
    // theme overrides
    pub theme_overrides: ThemeOverrides,

    // Shell integration
    pub shell_integration_commands: FxHashMap<String, String>,
    pub shell_integration_fallback_channel: String,

    // Channel-specific fields
    pub watch: f64,
    // metadata
    pub channel_name: String,
    pub channel_description: Option<String>,
    pub channel_requirements: Vec<BinaryRequirement>,
    // source
    pub channel_source_command: CommandSpec,
    pub channel_source_entry_delimiter: Option<char>,
    pub channel_source_ansi: bool,
    pub channel_source_display: Option<Template>,
    pub channel_source_output: Option<Template>,
    // preview
    pub channel_preview_command: Option<CommandSpec>,
    pub channel_preview_offset: Option<Template>,
    pub channel_preview_cached: bool,
    pub channel_actions: FxHashMap<String, ActionSpec>,
}
