use crate::{
    action::{Action, CUSTOM_ACTION_PREFIX},
    channels::prototypes::{
        ActionSpec, BinaryRequirement, ChannelPrototype, CommandSpec, Template,
    },
    cli::{ChannelCli, GlobalCli, PostProcessedCli},
    config::{
        Config, Keybindings, merge_keybindings,
        ui::{BorderType, Padding, ThemeOverrides},
    },
    keymap::InputMap,
    screen::layout::{InputPosition, Orientation},
    utils::shell::Shell,
};
use rustc_hash::FxHashMap;
use std::path::PathBuf;

/// Represents the different layers of configuration that make up the final
/// merged configuration used by the application.
pub struct ConfigLayers {
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

impl ConfigLayers {
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

    /// Update the current channel prototype and reset channel CLI options.
    pub fn update_channel(&mut self, channel: ChannelPrototype) {
        self.channel = channel;
        // Reset channel-specific CLI options to defaults
        self.channel_cli = ChannelCli::default();
    }

    /// Merges the different configuration layers into a single `MergedConfig`.
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
        let frecency_max_entries =
            self.base_config.application.frecency_max_entries;
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
        // Determine if sorting is disabled: --no-sort CLI flag OR channel config
        let no_sort = self.channel_cli.no_sort || self.channel.source.no_sort;
        let channel_name = self
            .channel_cli
            .channel
            .as_ref()
            .unwrap_or(&self.channel.metadata.name)
            .clone();

        // Global shell from base config (channel-specific shell overrides this)
        let global_shell = self.base_config.application.shell;

        // Build source command and apply global shell if no channel-specific shell
        let mut channel_source_command =
            if let Some(template) = &self.channel_cli.source_command {
                CommandSpec::from(template.clone())
            } else {
                self.channel.source.command.clone()
            };
        if channel_source_command.shell.is_none() {
            channel_source_command.shell = global_shell;
        }

        let channel_source_entry_delimiter = self
            .channel_cli
            .source_entry_delimiter
            .or(self.channel.source.entry_delimiter);
        let channel_source_ansi =
            self.channel_cli.ansi || self.channel.source.ansi;
        // Per-channel frecency setting (defaults to true, can be disabled per-channel)
        let channel_frecency = self.channel.source.frecency;
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

        // Build preview command and apply global shell if no channel-specific shell
        let mut channel_preview_command = self
            .channel_cli
            .preview_command
            .as_ref()
            .map(|t| CommandSpec::from(t.clone()))
            .or(self.channel.preview.as_ref().map(|p| p.command.clone()));
        if let Some(ref mut cmd) = channel_preview_command
            && cmd.shell.is_none()
        {
            cmd.shell = global_shell;
        }
        let channel_preview_offset = self
            .channel_cli
            .preview_offset
            .clone()
            .map(|t| vec![t])
            .or_else(|| {
                if let Some(preview) = &self.channel.preview {
                    preview.offset.clone()
                } else {
                    None
                }
            });
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
        let input_bar_position = self.channel_cli.input_position.unwrap_or(
            self.channel
                .ui
                .as_ref()
                .and_then(|ui| ui.input_bar.as_ref())
                .map_or(self.base_config.ui.input_bar.position, |ib| {
                    ib.position
                }),
        );

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
        let mut input_bar_padding = self
            .channel_cli
            .input_padding
            .or_else(|| {
                Some(self.channel.ui.as_ref()?.input_bar.as_ref()?.padding)
            })
            .unwrap_or(self.base_config.ui.input_bar.padding);
        let status_bar_disabled = self.global_cli.no_status_bar;
        let mut status_bar_hidden = if status_bar_disabled {
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
        let mut results_panel_padding = self
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
        let mut preview_panel_padding = self
            .channel_cli
            .preview_padding
            .or_else(|| {
                Some(self.channel.ui.as_ref()?.preview_panel.as_ref()?.padding)
            })
            .unwrap_or(self.base_config.ui.preview_panel.padding);

        // If the CLI sets word-wrap to true, we respect that
        let preview_panel_word_wrap = if self.channel_cli.preview_word_wrap {
            true
        // Otherwise, we check the channel UI config
        } else if let Some(ui) = self.channel.ui.as_ref()
            && let Some(panel) = ui.preview_panel.as_ref()
        {
            panel.word_wrap
        // And if no config is present, we check the base UI config
        } else {
            self.base_config.ui.preview_panel.word_wrap
        };

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

        // ------------------------------------------------------------------
        // Minimal UI preset. The static parts (no borders, no prompt, no
        // header, no scrollbar) are the built-in defaults over in config::ui;
        // what's left here are the dynamic bits: paddings that depend on the
        // input position or on another field staying borderless, and the
        // non-fullscreen (--inline / --height) extras. Guarded so explicit
        // CLI flags and non-default channel/config values always win.
        // ------------------------------------------------------------------
        let fullscreen = !inline && height.is_none();
        // hide the preview automatically when the viewport is too small
        // to fit a useful pane; an explicit --show-preview or
        // --preview-size signals the user wants it regardless
        let preview_panel_auto_hide = !self.channel_cli.show_preview
            && self.channel_cli.preview_size.is_none();
        if !fullscreen {
            let channel_sets_status_bar = self
                .channel
                .ui
                .as_ref()
                .is_some_and(|ui| ui.status_bar.is_some());

            // the status bar doesn't earn its row in a small viewport; the
            // channel name moves next to the result count instead
            if !self.channel_cli.show_status_bar && !channel_sets_status_bar {
                status_bar_hidden = true;
            }
        }
        // 1-column left margin so the query doesn't sit flush against
        // the terminal edge, plus a blank line between the input and
        // the results (only for a borderless input bar: a border already
        // provides both)
        if input_bar_border_type == BorderType::None
            && self.channel_cli.input_padding.is_none()
            && input_bar_padding == Padding::default()
        {
            input_bar_padding = match input_bar_position {
                InputPosition::Top => Padding::new(0, 1, 1, 0),
                InputPosition::Bottom => Padding::new(1, 0, 1, 0),
            };
        }
        if results_panel_border_type == BorderType::None
            && self.channel_cli.results_padding.is_none()
            && results_panel_padding == Padding::default()
        {
            // 1-column left margin, aligning the entries with the query
            results_panel_padding = Padding::new(0, 0, 1, 0);
        }
        // a borderless preview still needs a hint of separation from the
        // results list: a thin hairline on the side facing them
        let preview_panel_separator =
            preview_panel_border_type == BorderType::None;
        // breathing room between the preview title and its content
        if preview_panel_separator
            && self.channel_cli.preview_padding.is_none()
            && preview_panel_padding == Padding::default()
        {
            preview_panel_padding = Padding::new(1, 0, 0, 0);
        }
        // minimal input decoration (dimmed compact count, undecorated query)
        // follows the input bar itself being borderless
        let input_bar_minimal = input_bar_border_type == BorderType::None;

        // Do we have any channel-specific keybindings?
        let mut channel_keybindings = Keybindings::default();
        if let Some(channel_bindings) = &self.channel.keybindings {
            channel_keybindings = channel_bindings.bindings.clone();
        }
        if let Some(cli_bindings) = &self.channel_cli.keybindings {
            channel_keybindings =
                merge_keybindings(channel_keybindings, cli_bindings);
        }

        // Validate that all external actions referenced in channel keybindings exist
        if let Some(channel_bindings) = &self.channel.keybindings {
            for (_, actions) in channel_bindings.bindings.iter() {
                for action in actions.as_slice() {
                    if let Action::ExternalAction(custom_with_prefix) = action
                        && !channel_actions.contains_key(
                            custom_with_prefix
                                .trim_start_matches(CUSTOM_ACTION_PREFIX),
                        )
                    {
                        eprintln!(
                            "Action '{}' referenced in keybinding not found in actions section.",
                            custom_with_prefix
                        );
                        std::process::exit(1);
                    }
                }
            }
        }

        let input_map = InputMap::new(
            self.base_config.keybindings.clone(),
            channel_keybindings,
        );

        MergedConfig {
            // General
            data_dir,
            config_file,
            cable_dir,
            tick_rate,
            default_channel,
            history_size,
            global_history,
            frecency_max_entries,
            working_directory,
            autocomplete_prompt,
            shell: global_shell,
            // matcher configuration
            exact_match,
            select_1,
            take_1,
            take_1_fast,
            input,
            no_sort,

            // Bindings
            input_map,

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
            input_bar_minimal,
            // status bar
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
            preview_panel_word_wrap,
            preview_panel_hidden,
            preview_panel_disabled,
            preview_panel_separator,
            preview_panel_auto_hide,
            fullscreen,
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
            // frecency
            channel_frecency,
            // stdin
            is_stdin: self.channel.metadata.name == "stdin",
        }
    }
}

/// The final merged configuration used by the application, combining
/// settings from the base config, channel prototype, and CLI options.
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
    pub frecency_max_entries: usize,
    pub working_directory: Option<PathBuf>,
    pub autocomplete_prompt: Option<String>,
    /// Global shell for command execution (from base config).
    /// Already applied to `channel_source_command` and `channel_preview_command`.
    pub shell: Option<Shell>,
    // matcher configuration
    pub exact_match: bool,
    pub select_1: bool,
    pub take_1: bool,
    pub take_1_fast: bool,
    pub input: Option<String>,
    pub no_sort: bool,

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
    /// Show the channel name next to the result count (minimal UI preset).
    pub input_bar_minimal: bool,
    // status bar
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
    pub preview_panel_word_wrap: bool,
    pub preview_panel_hidden: bool,
    pub preview_panel_disabled: bool,
    /// Draw a single separator line between results and preview
    /// (minimal UI preset, only when no preview border is configured).
    pub preview_panel_separator: bool,
    /// Hide the preview automatically when the viewport is too small to fit
    /// a useful pane next to (or below) the results.
    pub preview_panel_auto_hide: bool,
    /// Whether tv runs in the whole terminal screen (as opposed to the
    /// --inline / --height viewports).
    pub fullscreen: bool,
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
    pub channel_preview_offset: Option<Vec<Template>>,
    pub channel_preview_cached: bool,
    pub channel_actions: FxHashMap<String, ActionSpec>,
    /// Whether frecency is enabled for the current channel (per-channel override)
    pub channel_frecency: bool,
    /// Whether the current channel reads from stdin directly
    pub is_stdin: bool,
}

impl MergedConfig {
    /// An empty input bar header means "no header line at all".
    pub fn input_bar_header_hidden(&self) -> bool {
        self.input_bar_header.as_deref().is_some_and(str::is_empty)
    }

    /// Number of vertical cells the results block chrome (borders + padding)
    /// takes away from the results area.
    pub fn results_panel_chrome_height(&self) -> u16 {
        let borders = if self.results_panel_border_type == BorderType::None {
            0
        } else {
            2
        };
        borders
            + self.results_panel_padding.top
            + self.results_panel_padding.bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::prototypes::UiSpec;

    fn merge_layers(
        config: Config,
        prototype: ChannelPrototype,
        channel_cli: ChannelCli,
        global_cli: GlobalCli,
    ) -> MergedConfig {
        ConfigLayers::new(
            config,
            prototype,
            PostProcessedCli {
                channel: channel_cli,
                global: global_cli,
            },
        )
        .merge()
    }

    #[test]
    fn minimal_preset_applies_to_inline_and_height() {
        for global_cli in [
            GlobalCli {
                inline: true,
                ..Default::default()
            },
            GlobalCli {
                height: Some(20),
                ..Default::default()
            },
        ] {
            let merged = merge_layers(
                Config::default(),
                ChannelPrototype::new("test", "echo 1"),
                ChannelCli::default(),
                global_cli,
            );
            assert!(merged.status_bar_hidden);
            assert_eq!(merged.input_bar_border_type, BorderType::None);
            assert_eq!(merged.results_panel_border_type, BorderType::None);
            assert_eq!(merged.preview_panel_border_type, BorderType::None);
            assert_eq!(merged.input_bar_header.as_deref(), Some(""));
            assert!(merged.input_bar_header_hidden());
            assert_eq!(merged.input_bar_prompt.as_deref(), Some(""));
            assert_eq!(merged.input_bar_padding, Padding::new(0, 1, 1, 0));
            assert_eq!(merged.results_panel_padding, Padding::new(0, 0, 1, 0));
            assert_eq!(merged.preview_panel_padding, Padding::new(1, 0, 0, 0));
            assert!(merged.preview_panel_separator);
            assert!(merged.preview_panel_auto_hide);
            assert!(!merged.preview_panel_scrollbar);
            assert!(merged.input_bar_minimal);
        }
    }

    #[test]
    fn minimal_preset_applies_fullscreen_but_keeps_status_bar() {
        let merged = merge_layers(
            Config::default(),
            ChannelPrototype::new("test", "echo 1"),
            ChannelCli::default(),
            GlobalCli::default(),
        );
        assert!(merged.fullscreen);
        // same borderless chrome as the small viewports...
        assert_eq!(merged.input_bar_border_type, BorderType::None);
        assert_eq!(merged.results_panel_border_type, BorderType::None);
        assert_eq!(merged.preview_panel_border_type, BorderType::None);
        assert!(merged.input_bar_header_hidden());
        assert_eq!(merged.input_bar_prompt.as_deref(), Some(""));
        assert!(merged.preview_panel_separator);
        assert!(merged.input_bar_minimal);
        // ...but the status bar stays
        assert!(!merged.status_bar_hidden);
        assert!(merged.preview_panel_auto_hide);
    }

    #[test]
    fn minimal_preset_respects_explicit_config_and_channel() {
        let mut config = Config::default();
        config.ui.input_bar.border_type = BorderType::Thick;
        let mut prototype = ChannelPrototype::new("test", "echo 1");
        prototype.ui = Some(UiSpec {
            status_bar: Some(crate::config::ui::StatusBarConfig::default()),
            ..Default::default()
        });
        let merged = merge_layers(
            config,
            prototype,
            ChannelCli::default(),
            GlobalCli {
                height: Some(20),
                ..Default::default()
            },
        );
        // non-default config file value survives
        assert_eq!(merged.input_bar_border_type, BorderType::Thick);
        // channel [ui.status_bar] presence keeps the status bar visible
        assert!(!merged.status_bar_hidden);
        // untouched fields still get the preset
        assert_eq!(merged.results_panel_border_type, BorderType::None);
        assert_eq!(merged.preview_panel_border_type, BorderType::None);
    }

    #[test]
    fn minimal_preset_ignores_unrelated_channel_ui_fields() {
        // A channel tweaking e.g. the preview size (like the stock files
        // channel) should still get the minimal borderless preview.
        let mut prototype = ChannelPrototype::new("test", "echo 1");
        prototype.ui = Some(UiSpec {
            preview_panel: Some(crate::config::ui::PreviewPanelConfig {
                size: 60,
                ..Default::default()
            }),
            ..Default::default()
        });
        let merged = merge_layers(
            Config::default(),
            prototype,
            ChannelCli::default(),
            GlobalCli {
                inline: true,
                ..Default::default()
            },
        );
        assert_eq!(merged.preview_panel_size, 60);
        assert_eq!(merged.preview_panel_border_type, BorderType::None);
        assert!(merged.preview_panel_separator);
        // but a channel explicitly picking a non-default border keeps it
        let mut prototype = ChannelPrototype::new("test", "echo 1");
        prototype.ui = Some(UiSpec {
            preview_panel: Some(crate::config::ui::PreviewPanelConfig {
                border_type: BorderType::Thick,
                ..Default::default()
            }),
            ..Default::default()
        });
        let merged = merge_layers(
            Config::default(),
            prototype,
            ChannelCli::default(),
            GlobalCli {
                inline: true,
                ..Default::default()
            },
        );
        assert_eq!(merged.preview_panel_border_type, BorderType::Thick);
        assert!(!merged.preview_panel_separator);
    }

    #[test]
    fn minimal_preset_cli_flags_win() {
        let channel_cli = ChannelCli {
            show_status_bar: true,
            preview_border: Some(BorderType::Rounded),
            input_header: Some(String::from("Custom")),
            ..Default::default()
        };
        let merged = merge_layers(
            Config::default(),
            ChannelPrototype::new("test", "echo 1"),
            channel_cli,
            GlobalCli {
                inline: true,
                ..Default::default()
            },
        );
        assert!(!merged.status_bar_hidden);
        assert_eq!(merged.preview_panel_border_type, BorderType::Rounded);
        assert!(!merged.preview_panel_separator);
        assert_eq!(merged.input_bar_header.as_deref(), Some("Custom"));
        // fields the CLI didn't touch still get the preset
        assert_eq!(merged.results_panel_border_type, BorderType::None);
    }
}
