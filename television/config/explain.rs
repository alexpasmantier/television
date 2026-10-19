//! `tv explain-config`: shows the effective configuration for a channel and
//! which layer (CLI flags, channel definition, config file, built-in
//! defaults) each value comes from.
//!
//! The effective values are always taken from [`ConfigLayers::merge`], i.e.
//! exactly what tv would run with. Each setting's source is the
//! highest-precedence layer providing that exact value; when no layer does
//! (e.g. paddings computed by the minimal UI preset), the value is reported as
//! [`Layer::Derived`].
use crate::{
    channels::prototypes::{CommandSpec, SourceCommand},
    config::{
        Config, default_config_from_file,
        layers::{ConfigLayers, MergedConfig},
        ui::ThemeOverrides,
    },
};
use colored::Colorize;
use serde::Serialize;
use serde_json::Value;
use std::{fmt::Write as _, path::PathBuf};

/// A configuration layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layer {
    /// Command line flags.
    Cli,
    /// The channel definition (its cable file).
    Channel,
    /// The user's configuration file.
    ConfigFile,
    /// tv's built-in defaults.
    Default,
    /// Computed by tv from other settings rather than taken verbatim from a
    /// single layer (e.g. the minimal UI preset).
    Derived,
}

/// Override precedence between layers, highest first.
pub const PRECEDENCE: [Layer; 4] = [
    Layer::Cli,
    Layer::Channel,
    Layer::ConfigFile,
    Layer::Default,
];

impl Layer {
    pub fn label(self) -> &'static str {
        match self {
            Layer::Cli => "cli",
            Layer::Channel => "channel",
            Layer::ConfigFile => "config file",
            Layer::Default => "default",
            Layer::Derived => "derived",
        }
    }

    fn colorize(self, s: &str) -> String {
        match self {
            Layer::Cli => s.yellow().to_string(),
            Layer::Channel => s.cyan().to_string(),
            Layer::ConfigFile => s.green().to_string(),
            Layer::Default => s.dimmed().to_string(),
            Layer::Derived => s.magenta().to_string(),
        }
    }
}

/// The value a single layer provides for a setting.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LayerValue {
    pub layer: Layer,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Setting {
    pub key: &'static str,
    /// The effective value.
    pub value: Value,
    /// The layer the effective value comes from.
    pub source: Layer,
    /// Every layer that provides a value for this setting, highest
    /// precedence first.
    pub layers: Vec<LayerValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Explanation {
    pub channel: String,
    /// The configuration file that was loaded, if any.
    pub config_file: Option<PathBuf>,
    pub precedence: Vec<Layer>,
    pub settings: Vec<Setting>,
}

impl Explanation {
    pub fn setting(&self, key: &str) -> Option<&Setting> {
        self.settings.iter().find(|s| s.key == key)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self)
            .expect("explanation should always serialize")
    }

    /// Human readable table, one setting per line.
    pub fn render(&self, color: bool) -> String {
        let paint = |layer: Layer, s: &str| {
            if color {
                layer.colorize(s)
            } else {
                s.to_string()
            }
        };
        let dim = |s: &str| {
            if color {
                s.dimmed().to_string()
            } else {
                s.to_string()
            }
        };

        let mut out = String::new();
        let _ = writeln!(
            out,
            "Effective configuration for channel '{}'",
            self.channel
        );
        let _ = writeln!(
            out,
            "Config file: {}",
            self.config_file.as_ref().map_or_else(
                || String::from("none (built-in defaults)"),
                |p| p.display().to_string()
            )
        );
        let _ = writeln!(
            out,
            "Precedence:  {}\n",
            self.precedence
                .iter()
                .map(|l| paint(*l, l.label()))
                .collect::<Vec<_>>()
                .join(" > ")
        );

        let key_width = self
            .settings
            .iter()
            .map(|s| s.key.len())
            .max()
            .unwrap_or(0)
            .max("SETTING".len());
        let source_width = "config file".len();
        let _ = writeln!(
            out,
            "{:<key_width$}  {:<source_width$}  VALUE",
            "SETTING", "SOURCE"
        );

        for setting in &self.settings {
            let source = format!("{:<source_width$}", setting.source.label());
            let _ = write!(
                out,
                "{:<key_width$}  {}  {}",
                setting.key,
                paint(setting.source, &source),
                display_value(&setting.value)
            );
            let derived = setting.source == Layer::Derived;
            let others = setting
                .layers
                .iter()
                .filter(|l| {
                    l.layer != setting.source
                        && !l.value.is_null()
                        && (derived || l.value != setting.value)
                })
                .map(|l| {
                    format!(
                        "{} = {}",
                        l.layer.label(),
                        display_value(&l.value)
                    )
                })
                .collect::<Vec<_>>();
            if !others.is_empty() {
                let verb = if derived { "from" } else { "overrides" };
                let _ = write!(
                    out,
                    "  {}",
                    dim(&format!("({verb} {})", others.join(", ")))
                );
            }
            out.push('\n');
        }
        out
    }
}

fn display_value(value: &Value) -> String {
    match value {
        Value::Null => String::from("-"),
        other => other.to_string(),
    }
}

fn v<T: Serialize>(value: T) -> Value {
    match serde_json::to_value(value).unwrap_or(Value::Null) {
        // unset optional fields (e.g. most theme overrides) are just noise
        Value::Object(map) => Value::Object(
            map.into_iter().filter(|(_, v)| !v.is_null()).collect(),
        ),
        other => other,
    }
}

/// A boolean CLI flag only counts as an override when it is passed.
fn flag(set: bool) -> Option<Value> {
    set.then_some(Value::Bool(true))
}

/// The values each layer provides for a setting (`None`: not set there).
#[derive(Default)]
struct Candidates {
    cli: Option<Value>,
    channel: Option<Value>,
    config_file: Option<Value>,
    default: Option<Value>,
}

#[derive(Default)]
struct Builder {
    settings: Vec<Setting>,
}

impl Builder {
    fn add(&mut self, key: &'static str, effective: Value, c: Candidates) {
        let mut layers = Vec::new();
        if let Some(value) = c.cli {
            layers.push(LayerValue {
                layer: Layer::Cli,
                value,
            });
        }
        if let Some(value) = c.channel {
            layers.push(LayerValue {
                layer: Layer::Channel,
                value,
            });
        }
        // the config file only counts when it changes the built-in default
        if let Some(value) = c.config_file
            && c.default.as_ref() != Some(&value)
        {
            layers.push(LayerValue {
                layer: Layer::ConfigFile,
                value,
            });
        }
        if let Some(value) = c.default {
            layers.push(LayerValue {
                layer: Layer::Default,
                value,
            });
        }
        let source = layers
            .iter()
            .find(|l| l.value == effective)
            .map_or(Layer::Derived, |l| l.layer);
        self.settings.push(Setting {
            key,
            value: effective,
            source,
            layers,
        });
    }
}

fn commands(spec: &CommandSpec) -> &[SourceCommand] {
    &spec.inner
}

impl ConfigLayers {
    /// Explains where each value of the merged configuration comes from.
    ///
    /// `config_file` is the configuration file that was loaded, if any.
    pub fn explain(&self, config_file: Option<PathBuf>) -> Explanation {
        let defaults = default_config_from_file().unwrap_or_default();
        self.explain_with_defaults(&self.merge(), &defaults, config_file)
    }

    #[allow(clippy::too_many_lines)]
    fn explain_with_defaults(
        &self,
        m: &MergedConfig,
        d: &Config,
        config_file: Option<PathBuf>,
    ) -> Explanation {
        let cfg = &self.base_config;
        let ch = &self.channel;
        let cli = &self.channel_cli;
        let g = &self.global_cli;
        let ui = ch.ui.as_ref();
        let input_bar = ui.and_then(|ui| ui.input_bar.as_ref());
        let status_bar = ui.and_then(|ui| ui.status_bar.as_ref());
        let results = ui.and_then(|ui| ui.results_panel.as_ref());
        let preview = ui.and_then(|ui| ui.preview_panel.as_ref());
        let help = ui.and_then(|ui| ui.help_panel.as_ref());
        let remote = ui.and_then(|ui| ui.remote_control.as_ref());

        let mut b = Builder::default();

        // general
        b.add(
            "default_channel",
            v(&m.default_channel),
            Candidates {
                config_file: Some(v(&cfg.application.default_channel)),
                default: Some(v(&d.application.default_channel)),
                ..Default::default()
            },
        );
        b.add(
            "cable_dir",
            v(&m.cable_dir),
            Candidates {
                cli: g.cable_dir.as_ref().map(v),
                config_file: Some(v(&cfg.application.cable_dir)),
                default: Some(v(&d.application.cable_dir)),
                ..Default::default()
            },
        );
        b.add(
            "data_dir",
            v(&m.data_dir),
            Candidates {
                config_file: Some(v(&cfg.application.data_dir)),
                default: Some(v(&d.application.data_dir)),
                ..Default::default()
            },
        );
        b.add(
            "tick_rate",
            v(m.tick_rate),
            Candidates {
                cli: g.tick_rate.map(v),
                config_file: Some(v(cfg.application.tick_rate)),
                default: Some(v(d.application.tick_rate)),
                ..Default::default()
            },
        );
        b.add(
            "history_size",
            v(m.history_size),
            Candidates {
                config_file: Some(v(cfg.application.history_size)),
                default: Some(v(d.application.history_size)),
                ..Default::default()
            },
        );
        b.add(
            "global_history",
            v(m.global_history),
            Candidates {
                cli: flag(g.global_history),
                channel: ch.history.global_mode.map(v),
                config_file: Some(v(cfg.application.global_history)),
                default: Some(v(d.application.global_history)),
            },
        );
        b.add(
            "frecency_max_entries",
            v(m.frecency_max_entries),
            Candidates {
                config_file: Some(v(cfg.application.frecency_max_entries)),
                default: Some(v(d.application.frecency_max_entries)),
                ..Default::default()
            },
        );
        b.add(
            "shell",
            v(m.shell),
            Candidates {
                config_file: Some(v(cfg.application.shell)),
                default: Some(v(d.application.shell)),
                ..Default::default()
            },
        );
        b.add(
            "typo_resistance",
            v(m.typo_resistance),
            Candidates {
                cli: flag(g.typo_resistance),
                config_file: Some(v(cfg.application.typo_resistance)),
                default: Some(v(d.application.typo_resistance)),
                ..Default::default()
            },
        );

        // behavior
        for (key, effective, set) in [
            ("exact", m.exact_match, cli.exact),
            ("select_1", m.select_1, cli.select_1),
            ("take_1", m.take_1, cli.take_1),
            ("take_1_fast", m.take_1_fast, cli.take_1_fast),
            ("inline", m.inline, g.inline),
        ] {
            b.add(
                key,
                v(effective),
                Candidates {
                    cli: flag(set),
                    default: Some(Value::Bool(false)),
                    ..Default::default()
                },
            );
        }
        for (key, effective, cli_value) in
            [("height", m.height, g.height), ("width", m.width, g.width)]
        {
            b.add(
                key,
                v(effective),
                Candidates {
                    cli: cli_value.map(v),
                    default: Some(Value::Null),
                    ..Default::default()
                },
            );
        }
        b.add(
            "input",
            v(&m.input),
            Candidates {
                cli: cli.input.as_ref().map(v),
                default: Some(Value::Null),
                ..Default::default()
            },
        );
        b.add(
            "watch",
            v(m.watch),
            Candidates {
                cli: cli.watch_interval.map(v),
                channel: Some(v(ch.watch)),
                ..Default::default()
            },
        );

        // source
        b.add(
            "source.command",
            v(commands(&m.channel_source_command)),
            Candidates {
                cli: cli
                    .source_command
                    .as_ref()
                    .map(|t| v(CommandSpec::from(t.clone()).inner)),
                channel: Some(v(commands(&ch.source.command))),
                ..Default::default()
            },
        );
        b.add(
            "source.shell",
            v(m.channel_source_command.shell),
            Candidates {
                channel: ch
                    .source
                    .command
                    .shell
                    .filter(|_| cli.source_command.is_none())
                    .map(v),
                config_file: Some(v(cfg.application.shell)),
                default: Some(v(d.application.shell)),
                ..Default::default()
            },
        );
        for (key, effective, cli_value, channel_value) in [
            (
                "source.display",
                &m.channel_source_display,
                &cli.source_display,
                &ch.source.display,
            ),
            (
                "source.output",
                &m.channel_source_output,
                &cli.source_output,
                &ch.source.output,
            ),
        ] {
            b.add(
                key,
                v(effective),
                Candidates {
                    cli: cli_value.as_ref().map(v),
                    channel: channel_value.as_ref().map(v),
                    default: Some(Value::Null),
                    ..Default::default()
                },
            );
        }
        b.add(
            "source.entry_delimiter",
            v(m.channel_source_entry_delimiter),
            Candidates {
                cli: cli.source_entry_delimiter.map(v),
                channel: ch.source.entry_delimiter.map(v),
                default: Some(Value::Null),
                ..Default::default()
            },
        );
        b.add(
            "source.ansi",
            v(m.channel_source_ansi),
            Candidates {
                cli: flag(cli.ansi),
                channel: Some(v(ch.source.ansi)),
                ..Default::default()
            },
        );
        b.add(
            "source.no_sort",
            v(m.no_sort),
            Candidates {
                cli: flag(cli.no_sort),
                channel: Some(v(ch.source.no_sort)),
                ..Default::default()
            },
        );
        b.add(
            "source.frecency",
            v(m.channel_frecency),
            Candidates {
                channel: Some(v(ch.source.frecency)),
                ..Default::default()
            },
        );

        // preview
        b.add(
            "preview.command",
            v(m.channel_preview_command.as_ref().map(commands)),
            Candidates {
                cli: cli
                    .preview_command
                    .as_ref()
                    .map(|t| v(CommandSpec::from(t.clone()).inner)),
                channel: ch.preview.as_ref().map(|p| v(commands(&p.command))),
                default: Some(Value::Null),
                ..Default::default()
            },
        );
        b.add(
            "preview.offset",
            v(&m.channel_preview_offset),
            Candidates {
                cli: cli.preview_offset.as_ref().map(v),
                channel: ch
                    .preview
                    .as_ref()
                    .and_then(|p| p.offset.as_ref())
                    .map(v),
                default: Some(Value::Null),
                ..Default::default()
            },
        );
        b.add(
            "preview.cached",
            v(m.channel_preview_cached),
            Candidates {
                cli: cli.cache_preview.map(v),
                channel: ch.preview.as_ref().map(|p| v(p.cached)),
                default: Some(Value::Bool(true)),
                ..Default::default()
            },
        );

        // ui
        b.add(
            "ui.theme",
            v(&m.theme),
            Candidates {
                config_file: Some(v(&cfg.ui.theme)),
                default: Some(v(&d.ui.theme)),
                ..Default::default()
            },
        );
        b.add(
            "ui.ui_scale",
            v(m.ui_scale),
            Candidates {
                cli: cli.ui_scale.map(v),
                channel: ui.and_then(|ui| ui.ui_scale).map(v),
                config_file: Some(v(cfg.ui.ui_scale)),
                default: Some(v(d.ui.ui_scale)),
            },
        );
        b.add(
            "ui.orientation",
            v(m.layout),
            Candidates {
                cli: cli.layout.map(v),
                channel: ui.and_then(|ui| ui.orientation).map(v),
                config_file: Some(v(cfg.ui.orientation)),
                default: Some(v(d.ui.orientation)),
            },
        );
        b.add(
            "ui.theme_overrides",
            v(&m.theme_overrides),
            Candidates {
                channel: ui
                    .map(|ui| &ui.theme_overrides)
                    .filter(|o| **o != ThemeOverrides::default())
                    .map(v),
                config_file: Some(v(&cfg.ui.theme_overrides)),
                default: Some(v(&d.ui.theme_overrides)),
                ..Default::default()
            },
        );

        // input bar
        b.add(
            "ui.input_bar.position",
            v(m.input_bar_position),
            Candidates {
                cli: cli.input_position.map(v),
                channel: input_bar.map(|ib| v(ib.position)),
                config_file: Some(v(cfg.ui.input_bar.position)),
                default: Some(v(d.ui.input_bar.position)),
            },
        );
        b.add(
            "ui.input_bar.header",
            v(&m.input_bar_header),
            Candidates {
                cli: cli.input_header.as_ref().map(v),
                channel: input_bar.and_then(|ib| ib.header.as_ref()).map(v),
                config_file: Some(v(&cfg.ui.input_bar.header)),
                default: Some(v(&d.ui.input_bar.header)),
            },
        );
        b.add(
            "ui.input_bar.prompt",
            v(&m.input_bar_prompt),
            Candidates {
                cli: cli.input_prompt.as_ref().map(v),
                channel: input_bar.and_then(|ib| ib.prompt.as_ref()).map(v),
                config_file: Some(v(&cfg.ui.input_bar.prompt)),
                default: Some(v(&d.ui.input_bar.prompt)),
            },
        );
        b.add(
            "ui.input_bar.border_type",
            v(m.input_bar_border_type),
            Candidates {
                cli: cli.input_border.map(v),
                channel: input_bar.map(|ib| v(ib.border_type)),
                config_file: Some(v(cfg.ui.input_bar.border_type)),
                default: Some(v(d.ui.input_bar.border_type)),
            },
        );
        b.add(
            "ui.input_bar.padding",
            v(m.input_bar_padding),
            Candidates {
                cli: cli.input_padding.map(v),
                channel: input_bar.map(|ib| v(ib.padding)),
                config_file: Some(v(cfg.ui.input_bar.padding)),
                default: Some(v(d.ui.input_bar.padding)),
            },
        );

        // status bar
        b.add(
            "ui.status_bar.hidden",
            v(m.status_bar_hidden),
            Candidates {
                cli: if g.no_status_bar || cli.hide_status_bar {
                    Some(Value::Bool(true))
                } else if cli.show_status_bar {
                    Some(Value::Bool(false))
                } else {
                    None
                },
                channel: status_bar.map(|sb| v(sb.hidden)),
                config_file: Some(v(cfg.ui.status_bar.hidden)),
                default: Some(v(d.ui.status_bar.hidden)),
            },
        );
        b.add(
            "ui.status_bar.disabled",
            v(m.status_bar_disabled),
            Candidates {
                cli: flag(g.no_status_bar),
                default: Some(Value::Bool(false)),
                ..Default::default()
            },
        );

        // results panel
        b.add(
            "ui.results_panel.border_type",
            v(m.results_panel_border_type),
            Candidates {
                cli: cli.results_border.map(v),
                channel: results.map(|rp| v(rp.border_type)),
                config_file: Some(v(cfg.ui.results_panel.border_type)),
                default: Some(v(d.ui.results_panel.border_type)),
            },
        );
        b.add(
            "ui.results_panel.padding",
            v(m.results_panel_padding),
            Candidates {
                cli: cli.results_padding.map(v),
                channel: results.map(|rp| v(rp.padding)),
                config_file: Some(v(cfg.ui.results_panel.padding)),
                default: Some(v(d.ui.results_panel.padding)),
            },
        );

        // preview panel
        b.add(
            "ui.preview_panel.size",
            v(m.preview_panel_size),
            Candidates {
                cli: cli.preview_size.map(v),
                channel: preview.map(|pp| v(pp.size)),
                config_file: Some(v(cfg.ui.preview_panel.size)),
                default: Some(v(d.ui.preview_panel.size)),
            },
        );
        b.add(
            "ui.preview_panel.header",
            v(&m.preview_panel_header),
            Candidates {
                cli: cli.preview_header.as_ref().map(v),
                channel: preview.and_then(|pp| pp.header.as_ref()).map(v),
                config_file: Some(v(&cfg.ui.preview_panel.header)),
                default: Some(v(&d.ui.preview_panel.header)),
            },
        );
        b.add(
            "ui.preview_panel.footer",
            v(&m.preview_panel_footer),
            Candidates {
                cli: cli.preview_footer.as_ref().map(v),
                channel: preview.and_then(|pp| pp.footer.as_ref()).map(v),
                config_file: Some(v(&cfg.ui.preview_panel.footer)),
                default: Some(v(&d.ui.preview_panel.footer)),
            },
        );
        b.add(
            "ui.preview_panel.scrollbar",
            v(m.preview_panel_scrollbar),
            Candidates {
                cli: cli.hide_preview_scrollbar.then_some(Value::Bool(false)),
                channel: preview.map(|pp| v(pp.scrollbar)),
                config_file: Some(v(cfg.ui.preview_panel.scrollbar)),
                default: Some(v(d.ui.preview_panel.scrollbar)),
            },
        );
        b.add(
            "ui.preview_panel.border_type",
            v(m.preview_panel_border_type),
            Candidates {
                cli: cli.preview_border.map(v),
                channel: preview.map(|pp| v(pp.border_type)),
                config_file: Some(v(cfg.ui.preview_panel.border_type)),
                default: Some(v(d.ui.preview_panel.border_type)),
            },
        );
        b.add(
            "ui.preview_panel.padding",
            v(m.preview_panel_padding),
            Candidates {
                cli: cli.preview_padding.map(v),
                channel: preview.map(|pp| v(pp.padding)),
                config_file: Some(v(cfg.ui.preview_panel.padding)),
                default: Some(v(d.ui.preview_panel.padding)),
            },
        );
        b.add(
            "ui.preview_panel.word_wrap",
            v(m.preview_panel_word_wrap),
            Candidates {
                cli: flag(cli.preview_word_wrap),
                channel: preview.map(|pp| v(pp.word_wrap)),
                config_file: Some(v(cfg.ui.preview_panel.word_wrap)),
                default: Some(v(d.ui.preview_panel.word_wrap)),
            },
        );
        b.add(
            "ui.preview_panel.hidden",
            v(m.preview_panel_hidden),
            Candidates {
                cli: if m.preview_panel_disabled || cli.hide_preview {
                    Some(Value::Bool(true))
                } else if cli.show_preview {
                    Some(Value::Bool(false))
                } else {
                    None
                },
                channel: preview.map(|pp| v(pp.hidden)),
                config_file: Some(v(cfg.ui.preview_panel.hidden)),
                default: Some(v(d.ui.preview_panel.hidden)),
            },
        );
        b.add(
            "ui.preview_panel.disabled",
            v(m.preview_panel_disabled),
            Candidates {
                cli: flag(g.no_preview || cli.no_preview),
                default: Some(Value::Bool(false)),
                ..Default::default()
            },
        );

        // help panel
        b.add(
            "ui.help_panel.show_categories",
            v(m.help_panel_show_categories),
            Candidates {
                channel: help.map(|hp| v(hp.show_categories)),
                config_file: Some(v(cfg.ui.help_panel.show_categories)),
                default: Some(v(d.ui.help_panel.show_categories)),
                ..Default::default()
            },
        );
        b.add(
            "ui.help_panel.hidden",
            v(m.help_panel_hidden),
            Candidates {
                cli: if cli.show_help_panel {
                    Some(Value::Bool(false))
                } else {
                    flag(cli.hide_help_panel)
                },
                channel: help.map(|hp| v(hp.hidden)),
                config_file: Some(v(cfg.ui.help_panel.hidden)),
                default: Some(v(d.ui.help_panel.hidden)),
            },
        );
        b.add(
            "ui.help_panel.disabled",
            v(m.help_panel_disabled),
            Candidates {
                cli: flag(g.no_help_panel),
                channel: help.map(|hp| v(hp.disabled)),
                config_file: Some(v(cfg.ui.help_panel.disabled)),
                default: Some(v(d.ui.help_panel.disabled)),
            },
        );

        // remote control
        b.add(
            "ui.remote_control.show_channel_descriptions",
            v(m.remote_show_channel_descriptions),
            Candidates {
                channel: remote.map(|rc| v(rc.show_channel_descriptions)),
                config_file: Some(v(cfg
                    .ui
                    .remote_control
                    .show_channel_descriptions)),
                default: Some(v(d
                    .ui
                    .remote_control
                    .show_channel_descriptions)),
                ..Default::default()
            },
        );
        b.add(
            "ui.remote_control.sort_alphabetically",
            v(m.remote_sort_alphabetically),
            Candidates {
                channel: remote.map(|rc| v(rc.sort_alphabetically)),
                config_file: Some(v(cfg
                    .ui
                    .remote_control
                    .sort_alphabetically)),
                default: Some(v(d.ui.remote_control.sort_alphabetically)),
                ..Default::default()
            },
        );
        b.add(
            "ui.remote_control.disabled",
            v(m.remote_disabled),
            Candidates {
                cli: flag(g.no_remote),
                channel: remote.map(|rc| v(rc.disabled)),
                default: Some(Value::Bool(false)),
                ..Default::default()
            },
        );

        Explanation {
            channel: m.channel_name.clone(),
            config_file,
            precedence: PRECEDENCE.to_vec(),
            settings: b.settings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cable::Cable,
        channels::prototypes::ChannelPrototype,
        cli::{args::Cli, post_process},
    };
    use clap::Parser;

    fn explain(
        config: Config,
        prototype: &ChannelPrototype,
        flags: &[&str],
    ) -> Explanation {
        let cable = Cable::from_prototypes(vec![prototype.clone()]);
        let args = ["tv", "test"].iter().chain(flags);
        let cli =
            post_process(Cli::try_parse_from(args).unwrap(), false, &cable);
        let layers = ConfigLayers::new(config, prototype.clone(), cli);
        let defaults = default_config_from_file().unwrap();
        layers.explain_with_defaults(&layers.merge(), &defaults, None)
    }

    fn prototype() -> ChannelPrototype {
        toml::from_str(
            r#"
            [metadata]
            name = "test"
            [source]
            command = "echo entry"
            [preview]
            command = "echo preview"
            [ui.preview_panel]
            size = 60
            "#,
        )
        .unwrap()
    }

    fn source_of(e: &Explanation, key: &str) -> Layer {
        e.setting(key).unwrap().source
    }

    #[test]
    fn untouched_settings_come_from_defaults() {
        let e =
            explain(default_config_from_file().unwrap(), &prototype(), &[]);
        assert_eq!(source_of(&e, "tick_rate"), Layer::Default);
        assert_eq!(source_of(&e, "ui.ui_scale"), Layer::Default);
        assert_eq!(source_of(&e, "exact"), Layer::Default);
        assert_eq!(e.channel, "test");
        assert_eq!(e.precedence, PRECEDENCE.to_vec());
    }

    #[test]
    fn precedence_cli_over_channel_over_config_over_default() {
        let mut config = default_config_from_file().unwrap();
        config.ui.preview_panel.size = 40;
        config.application.tick_rate = 25;

        // channel beats config file
        let e = explain(config.clone(), &prototype(), &[]);
        let size = e.setting("ui.preview_panel.size").unwrap();
        assert_eq!(size.source, Layer::Channel);
        assert_eq!(size.value, Value::from(60));
        assert_eq!(
            size.layers.iter().map(|l| l.layer).collect::<Vec<_>>(),
            vec![Layer::Channel, Layer::ConfigFile, Layer::Default]
        );
        // config file beats default
        assert_eq!(source_of(&e, "tick_rate"), Layer::ConfigFile);

        // cli beats everything
        let e = explain(config, &prototype(), &["--preview-size", "70"]);
        let size = e.setting("ui.preview_panel.size").unwrap();
        assert_eq!(size.source, Layer::Cli);
        assert_eq!(size.value, Value::from(70));
        assert_eq!(size.layers[0].layer, Layer::Cli);
    }

    #[test]
    fn config_file_value_equal_to_default_is_reported_as_default() {
        let config = default_config_from_file().unwrap();
        let e = explain(config, &prototype(), &[]);
        let tick_rate = e.setting("tick_rate").unwrap();
        assert!(
            !tick_rate
                .layers
                .iter()
                .any(|l| l.layer == Layer::ConfigFile)
        );
    }

    #[test]
    fn effective_values_match_merged_config() {
        let e = explain(
            default_config_from_file().unwrap(),
            &prototype(),
            &["--exact", "--layout", "portrait", "--no-cache-preview"],
        );
        assert_eq!(e.setting("exact").unwrap().value, Value::Bool(true));
        assert_eq!(source_of(&e, "exact"), Layer::Cli);
        assert_eq!(
            e.setting("ui.orientation").unwrap().value,
            Value::from("portrait")
        );
        assert_eq!(
            e.setting("preview.cached").unwrap().value,
            Value::Bool(false)
        );
        assert_eq!(
            e.setting("preview.command").unwrap().value,
            serde_json::json!(["echo preview"])
        );
        assert_eq!(source_of(&e, "preview.command"), Layer::Channel);
    }

    #[test]
    fn computed_values_are_reported_as_derived() {
        // in inline mode the status bar is hidden by the minimal UI preset
        let e = explain(
            default_config_from_file().unwrap(),
            &prototype(),
            &["--inline"],
        );
        let hidden = e.setting("ui.status_bar.hidden").unwrap();
        assert_eq!(hidden.value, Value::Bool(true));
        assert_eq!(hidden.source, Layer::Derived);
    }

    #[test]
    fn json_and_human_output() {
        let e = explain(
            default_config_from_file().unwrap(),
            &prototype(),
            &["--preview-size", "70"],
        );
        let json: Value = serde_json::from_str(&e.to_json()).unwrap();
        assert_eq!(json["channel"], "test");
        assert_eq!(
            json["precedence"],
            serde_json::json!(["cli", "channel", "config-file", "default"])
        );
        let size = json["settings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["key"] == "ui.preview_panel.size")
            .unwrap();
        assert_eq!(size["source"], "cli");
        assert_eq!(size["value"], 70);

        let text = e.render(false);
        assert!(text.contains("Effective configuration for channel 'test'"));
        assert!(text.contains("cli > channel > config file > default"));
        let line = text
            .lines()
            .find(|l| l.starts_with("ui.preview_panel.size"))
            .unwrap();
        assert!(line.contains("cli"), "{line}");
        assert!(line.contains("70"), "{line}");
        assert!(line.contains("overrides channel = 60"), "{line}");
    }
}
