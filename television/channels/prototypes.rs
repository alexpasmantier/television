use crate::cli::parse_source_entry_delimiter;
use crate::config::ui::{InputBarConfig, ThemeOverrides};
use crate::{
    config::{KeyBindings, ui},
    event::Key,
    screen::layout::Orientation,
};
use anyhow::Result;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_with::{OneOrMany, serde_as};
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use string_pipeline::MultiTemplate;
use which::which;

#[derive(Debug, Clone)]
pub enum Template {
    StringPipeline(MultiTemplate),
    Raw(String),
}

impl Template {
    pub fn raw(&self) -> &str {
        match self {
            Template::StringPipeline(template) => template.template_string(),
            Template::Raw(raw) => raw,
        }
    }

    pub fn parse(template: &str) -> Result<Self, String> {
        match MultiTemplate::parse(template) {
            Ok(multi_template) => Ok(Template::StringPipeline(multi_template)),
            Err(_) => Ok(Template::Raw(template.to_string())),
        }
    }

    pub fn format(&self, input: &str) -> Result<String> {
        match self {
            Template::StringPipeline(template) => {
                template.format(input).map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to format template '{}' with '{}': {}",
                        self.raw(),
                        input,
                        e
                    )
                })
            }
            Template::Raw(raw) => Ok(raw.replace("{}", input)),
        }
    }
}

impl Display for Template {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.raw())
    }
}

impl PartialEq for Template {
    fn eq(&self, other: &Self) -> bool {
        self.raw() == other.raw()
            && matches!(
                (self, other),
                (Template::StringPipeline(_), Template::StringPipeline(_))
                    | (Template::Raw(_), Template::Raw(_))
            )
    }
}

impl Eq for Template {}

impl Hash for Template {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw().hash(state);
    }
}

impl Serialize for Template {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.raw())
    }
}

impl<'de> Deserialize<'de> for Template {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Template::parse(&raw).map_err(serde::de::Error::custom)
    }
}

#[serde_as]
#[derive(
    Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Default,
)]
pub struct CommandSpec {
    #[serde(rename = "command")]
    #[serde_as(as = "OneOrMany<_>")]
    pub inner: Vec<Template>,
    #[serde(default)]
    pub interactive: bool,
    #[serde(default)]
    pub env: FxHashMap<String, String>,
}

impl Display for CommandSpec {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.inner
                .iter()
                .map(Template::raw)
                .collect::<Vec<_>>()
                .join(";")
        )
    }
}

impl From<Template> for CommandSpec {
    fn from(template: Template) -> Self {
        Self::new(vec![template], false, FxHashMap::default())
    }
}

impl CommandSpec {
    pub fn new(
        inner: Vec<Template>,
        interactive: bool,
        env: FxHashMap<String, String>,
    ) -> Self {
        Self {
            inner,
            interactive,
            env,
        }
    }

    pub fn command_count(&self) -> usize {
        self.inner.len()
    }

    pub fn has_multiple_commands(&self) -> bool {
        self.inner.len() > 1
    }

    /// This wraps back to the first command in a circular manner.
    ///
    /// # Panics
    /// If the command spec does not contain any commands.
    pub fn get_nth(&self, index: usize) -> &Template {
        &self.inner[index % self.inner.len()]
    }

    pub fn from_template(template: Template) -> Self {
        Self::new(vec![template], false, FxHashMap::default())
    }
}

/// Execution mode for external actions
#[derive(
    Debug, Clone, Default, serde::Deserialize, serde::Serialize, PartialEq,
)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    /// Fork the command as a child process (tv stays open)
    #[default]
    Fork,
    /// Replace the current process with the command (tv exits, command takes over)
    Execute,
}

fn default_separator() -> String {
    " ".to_string()
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct ActionSpec {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(flatten)]
    pub command: CommandSpec,
    /// How to execute the command
    #[serde(default)]
    pub mode: ExecutionMode,
    /// Separator to use when formatting multiple entries into the command
    ///
    /// Example: `rm file1+SEPARATOR+file2+SEPARATOR+file3`
    #[serde(default = "default_separator")]
    pub separator: String,
    // TODO: add `requirements` (see `prototypes::BinaryRequirement`)
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChannelKeyBindings {
    /// Optional channel specific shortcut that, when pressed, switches directly to this channel.
    #[serde(default)]
    pub shortcut: Option<Key>,
    /// Regular action -> binding mappings living at channel level.
    #[serde(flatten)]
    #[serde(default)]
    pub bindings: KeyBindings,
}

impl ChannelKeyBindings {
    pub fn channel_shortcut(&self) -> Option<&Key> {
        self.shortcut.as_ref()
    }
}

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct HistoryConfig {
    /// Whether to use global history for this channel (overrides global setting)
    #[serde(default)]
    pub global_mode: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChannelPrototype {
    pub metadata: Metadata,
    pub source: SourceSpec,
    #[serde(default)]
    pub preview: Option<PreviewSpec>,
    #[serde(default)]
    pub ui: Option<UiSpec>,
    #[serde(default)]
    pub keybindings: Option<ChannelKeyBindings>,
    /// Watch interval in seconds for automatic reloading (0 = disabled)
    #[serde(default)]
    pub watch: f64,
    #[serde(default)]
    pub history: HistoryConfig,
    #[serde(default)]
    pub actions: FxHashMap<String, ActionSpec>,
}

impl ChannelPrototype {
    pub fn new(name: &str, command: &str) -> Self {
        Self {
            metadata: Metadata {
                name: name.to_string(),
                description: None,
                requirements: vec![],
            },
            source: SourceSpec {
                command: CommandSpec {
                    inner: vec![
                        Template::parse(command)
                            .expect("Failed to parse command"),
                    ],
                    interactive: false,
                    env: FxHashMap::default(),
                },
                entry_delimiter: None,
                ansi: false,
                display: None,
                output: None,
            },
            preview: None,
            ui: None,
            keybindings: None,
            watch: 0.0,
            history: HistoryConfig::default(),
            actions: FxHashMap::default(),
        }
    }

    pub fn stdin() -> Self {
        Self {
            metadata: Metadata {
                name: "stdin".to_string(),
                description: Some(
                    "A channel that reads from stdin".to_string(),
                ),
                requirements: vec![],
            },
            source: SourceSpec {
                command: CommandSpec {
                    inner: vec![Template::parse("cat").unwrap()],
                    ..Default::default()
                },
                ..Default::default()
            },
            preview: None,
            ui: None,
            keybindings: None,
            watch: 0.0,
            history: HistoryConfig::default(),
            actions: FxHashMap::default(),
        }
    }

    pub fn with_preview(mut self, preview: Option<PreviewSpec>) -> Self {
        self.preview = preview;
        self
    }
}

impl Display for ChannelPrototype {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.metadata.name)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Metadata {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub requirements: Vec<BinaryRequirement>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct BinaryRequirement {
    pub bin_name: String,
    #[serde(skip)]
    met: bool,
}

impl BinaryRequirement {
    pub fn new(bin_name: &str) -> Self {
        Self {
            bin_name: bin_name.to_string(),
            met: false,
        }
    }

    /// Check if the required binary is available in the system's PATH.
    ///
    /// This method updates the requirement's state in place to reflect whether the binary was
    /// found.
    pub fn init(&mut self) {
        self.met = which(&self.bin_name).is_ok();
    }

    /// Whether the requirement is available in the system's PATH.
    ///
    /// This should be called after `init()`.
    pub fn is_met(&self) -> bool {
        self.met
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct SourceSpec {
    #[serde(flatten)]
    pub command: CommandSpec,
    #[serde(deserialize_with = "deserialize_entry_delimiter", default)]
    pub entry_delimiter: Option<char>,
    #[serde(default)]
    pub ansi: bool,
    #[serde(default)]
    pub display: Option<Template>,
    #[serde(default)]
    pub output: Option<Template>,
}

/// Just a helper function to adapt cli parsing to serde deserialization.
fn deserialize_entry_delimiter<'de, D>(
    deserializer: D,
) -> Result<Option<char>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    if let Ok(Some(delimiter)) = Option::<String>::deserialize(deserializer) {
        parse_source_entry_delimiter(&delimiter)
            .map(Some)
            .map_err(serde::de::Error::custom)
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PreviewSpec {
    #[serde(flatten)]
    pub command: CommandSpec,
    #[serde(default)]
    pub offset: Option<Template>,
    #[serde(default)]
    pub cached: bool,
}

impl PreviewSpec {
    pub fn new(command: CommandSpec, offset: Option<Template>) -> Self {
        Self {
            command,
            offset,
            cached: false,
        }
    }

    pub fn from_str_command(command: &str) -> Self {
        Self {
            command: CommandSpec {
                inner: vec![
                    Template::parse(command)
                        .expect("Failed to parse preview command"),
                ],
                interactive: false,
                env: FxHashMap::default(),
            },
            offset: None,
            cached: false,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UiSpec {
    #[serde(default)]
    pub ui_scale: Option<u16>,
    // `layout` is clearer for the user but collides with the overall `Layout` type.
    #[serde(rename = "layout", alias = "orientation", default)]
    pub orientation: Option<Orientation>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub theme_overrides: ThemeOverrides,
    // Feature-specific configurations
    #[serde(default)]
    pub input_bar: Option<InputBarConfig>,
    #[serde(default)]
    pub preview_panel: Option<ui::PreviewPanelConfig>,
    #[serde(default)]
    pub results_panel: Option<ui::ResultsPanelConfig>,
    #[serde(default)]
    pub status_bar: Option<ui::StatusBarConfig>,
    #[serde(default)]
    pub help_panel: Option<ui::HelpPanelConfig>,
    #[serde(default)]
    pub remote_control: Option<ui::RemoteControlConfig>,
}

pub const DEFAULT_PROTOTYPE_NAME: &str = "files";

impl From<&crate::config::UiConfig> for UiSpec {
    fn from(config: &crate::config::UiConfig) -> Self {
        UiSpec {
            ui_scale: Some(config.ui_scale),
            orientation: Some(config.orientation),
            theme: Some(config.theme.clone()),
            theme_overrides: config.theme_overrides.clone(),
            input_bar: Some(config.input_bar.clone()),
            preview_panel: Some(config.preview_panel.clone()),
            results_panel: Some(config.results_panel.clone()),
            status_bar: Some(config.status_bar.clone()),
            help_panel: Some(config.help_panel.clone()),
            remote_control: Some(config.remote_control.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        action::Action, config::ui::BorderType, event::Key,
        screen::layout::InputPosition,
    };

    use super::*;
    use toml::from_str;

    #[test]
    fn test_command_spec_get_nth() {
        let command_spec = CommandSpec {
            inner: vec![
                Template::parse("cmd1").unwrap(),
                Template::parse("cmd2").unwrap(),
                Template::parse("cmd3").unwrap(),
            ],
            interactive: false,
            env: FxHashMap::default(),
        };

        assert_eq!(command_spec.get_nth(0).raw(), "cmd1");
        assert_eq!(command_spec.get_nth(1).raw(), "cmd2");
        assert_eq!(command_spec.get_nth(2).raw(), "cmd3");
        assert_eq!(command_spec.get_nth(3).raw(), "cmd1"); // wraps around
    }

    #[test]
    fn test_template_serialization() {
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct TestStruct {
            template: Template,
        }
        let raw_1 = r#"template = "Hello, {}""#;
        let raw_2 = r#"template = "Hello, World""#;
        let raw_3 = r#"template = "docker images --format '{{.Repository}}:{{.Tag}} {{.ID}}'""#;

        let test_1: TestStruct = from_str(raw_1).unwrap();
        let test_2: TestStruct = from_str(raw_2).unwrap();
        let test_3: TestStruct = from_str(raw_3).unwrap();

        assert_eq!(
            test_1.template,
            Template::StringPipeline(
                MultiTemplate::parse("Hello, {}").unwrap()
            )
        );
        assert_eq!(
            test_2.template,
            Template::StringPipeline(
                MultiTemplate::parse("Hello, World").unwrap()
            )
        );
        assert_eq!(
            test_3.template,
            Template::Raw(
                "docker images --format '{{.Repository}}:{{.Tag}} {{.ID}}'"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_channel_prototype_deserialization() {
        let toml_data = r#"
        [metadata]
        name = "files"
        description = "A channel to select files and directories"
        requirements = ["fd", "bat"]

        [source]
        command = "fd -t f"
        interactive = false
        env = {}
        display = "{split:/:-1}" # only show the last path segment ('/a/b/c' -> 'c')
        output = "{}"            # output the full path
        unknown_field = "ignored" # should be ignored

        [preview]
        command = "bat -n --color=always {}"
        env = { "BAT_THEME" = "ansi" }
        interactive = false
        offset = "3" # why not

        [ui]
        layout = "landscape"
        ui_scale = 100

        [ui.features]
        preview_panel = { enabled = true, visible = true }

        [ui.input_bar]
        position = "bottom"
        header = "Input: {}"
        border_type = "plain"

        [ui.preview_panel]
        size = 66
        header = "Preview: {}"
        footer = "Press 'q' to quit"
        border_type = "thick"

        [ui.results_panel]
        border_type = "none"

        [keybindings]
        esc = "quit"
        ctrl-c = "quit"
        down = "select_next_entry"
        ctrl-n = "select_next_entry"
        ctrl-j = "select_next_entry"
        up = "select_prev_entry"
        ctrl-p = "select_prev_entry"
        ctrl-k = "select_prev_entry"
        enter = "confirm_selection"
        "#;

        let prototype: ChannelPrototype = from_str(toml_data).unwrap();

        assert_eq!(prototype.metadata.name, "files");
        assert_eq!(
            prototype.metadata.description,
            Some("A channel to select files and directories".to_string())
        );
        assert_eq!(
            format!("{}", prototype.source.command.inner[0]),
            "fd -t f"
        );

        assert!(!prototype.source.command.interactive);
        assert_eq!(prototype.source.display.unwrap().raw(), "{split:/:-1}");
        assert_eq!(prototype.source.output.unwrap().raw(), "{}");

        let preview = prototype.preview.as_ref().unwrap();
        assert_eq!(
            format!("{}", preview.command.inner[0]),
            "bat -n --color=always {}"
        );
        assert!(!preview.command.interactive);
        assert_eq!(
            preview.command.env.get("BAT_THEME"),
            Some(&"ansi".to_string())
        );
        assert_eq!(preview.offset.as_ref().unwrap().raw(), "3");

        let ui = prototype.ui.unwrap();
        assert_eq!(ui.orientation, Some(Orientation::Landscape));
        assert_eq!(ui.ui_scale, Some(100));
        assert_eq!(ui.preview_panel.as_ref().unwrap().size, 66);
        let input_bar = ui.input_bar.as_ref().unwrap();
        assert_eq!(input_bar.position, InputPosition::Bottom);
        assert_eq!(input_bar.header.as_ref().unwrap(), "Input: {}");
        assert_eq!(input_bar.border_type, BorderType::Plain);
        let preview_panel = ui.preview_panel.as_ref().unwrap();
        assert_eq!(
            preview_panel.header.as_ref().unwrap().raw(),
            "Preview: {}"
        );
        assert_eq!(
            preview_panel.footer.as_ref().unwrap().raw(),
            "Press 'q' to quit"
        );
        assert_eq!(preview_panel.border_type, BorderType::Thick);

        assert_eq!(ui.results_panel.unwrap().border_type, BorderType::None);

        let keybindings = prototype.keybindings.unwrap();
        assert_eq!(
            keybindings.bindings.get(&Key::Esc),
            Some(&Action::Quit.into())
        );
        assert_eq!(
            keybindings.bindings.get(&Key::Ctrl('c')),
            Some(&Action::Quit.into())
        );
        assert_eq!(
            keybindings.bindings.get(&Key::Down),
            Some(&Action::SelectNextEntry.into())
        );
        assert_eq!(
            keybindings.bindings.get(&Key::Ctrl('n')),
            Some(&Action::SelectNextEntry.into())
        );
        assert_eq!(
            keybindings.bindings.get(&Key::Ctrl('j')),
            Some(&Action::SelectNextEntry.into())
        );
        assert_eq!(
            keybindings.bindings.get(&Key::Up),
            Some(&Action::SelectPrevEntry.into())
        );
        assert_eq!(
            keybindings.bindings.get(&Key::Ctrl('p')),
            Some(&Action::SelectPrevEntry.into())
        );
        assert_eq!(
            keybindings.bindings.get(&Key::Ctrl('k')),
            Some(&Action::SelectPrevEntry.into())
        );
        assert_eq!(
            keybindings.bindings.get(&Key::Enter),
            Some(&Action::ConfirmSelection.into())
        );
    }

    #[test]
    fn test_channel_prototype_deserialization_multiple_commands() {
        let toml_data = r#"
        [metadata]
        name = "files"
        description = "A channel to select files and directories"
        requirements = ["fd", "bat"]

        [source]
        command = ["fd -t f", "fd -t f --hidden"]
        output = "{}"            # output the full path
        "#;

        let prototype: ChannelPrototype = from_str(toml_data).unwrap();

        assert_eq!(prototype.metadata.name, "files");
        assert_eq!(
            prototype.metadata.description,
            Some("A channel to select files and directories".to_string())
        );
        assert_eq!(
            prototype
                .source
                .command
                .inner
                .iter()
                .map(Template::raw)
                .collect::<Vec<_>>(),
            vec!["fd -t f", "fd -t f --hidden"]
        );
        assert!(!prototype.source.command.interactive);
        assert!(prototype.source.command.env.is_empty());
        assert_eq!(prototype.source.output.unwrap().raw(), "{}");
    }

    #[test]
    fn test_channel_prototype_deserialization_bare_minimum() {
        let toml_data = r#"
        [metadata]
        name = "files"
        description = "A channel to select files and directories"
        requirements = ["fd"]

        [source]
        command = "fd -t f"
        "#;

        let prototype: ChannelPrototype = from_str(toml_data).unwrap();

        assert_eq!(prototype.metadata.name, "files");
        assert_eq!(
            prototype.metadata.description,
            Some("A channel to select files and directories".to_string())
        );
        assert_eq!(
            format!("{}", prototype.source.command.inner[0]),
            "fd -t f"
        );
        assert!(!prototype.source.command.interactive);
        assert!(prototype.source.command.env.is_empty());
        assert!(prototype.source.display.is_none());
        assert!(prototype.source.output.is_none());
        assert!(prototype.preview.is_none());
        assert!(prototype.ui.is_none());
        assert!(prototype.keybindings.is_none());
    }

    #[test]
    fn test_channel_prototype_deserialization_partial_ui_options() {
        let toml_data = r#"
        [metadata]
        name = "files"
        description = "A channel to select files and directories"
        requirements = ["fd"]

        [source]
        command = "fd -t f"

        [ui]
        layout = "landscape"
        ui_scale = 40

        [ui.input_bar]
        border_type = "none"

        [ui.preview_panel]
        footer = "Press 'q' to quit"
        "#;

        let prototype: ChannelPrototype = from_str(toml_data).unwrap();

        assert_eq!(prototype.metadata.name, "files");
        assert_eq!(
            prototype.metadata.description,
            Some("A channel to select files and directories".to_string())
        );
        assert_eq!(
            format!("{}", prototype.source.command.inner[0]),
            "fd -t f"
        );
        assert!(!prototype.source.command.interactive);
        assert!(prototype.source.command.env.is_empty());
        assert!(prototype.source.display.is_none());
        assert!(prototype.source.output.is_none());

        let ui = prototype.ui.unwrap();
        assert_eq!(ui.orientation, Some(Orientation::Landscape));
        assert_eq!(ui.ui_scale, Some(40));
        assert_eq!(
            ui.input_bar.as_ref().unwrap().border_type,
            BorderType::None
        );
        assert!(ui.preview_panel.is_some());
        assert_eq!(
            ui.preview_panel
                .as_ref()
                .unwrap()
                .footer
                .as_ref()
                .unwrap()
                .raw(),
            "Press 'q' to quit"
        );
        assert!(ui.status_bar.is_none());
        assert!(ui.help_panel.is_none());
        assert!(ui.remote_control.is_none());
    }

    #[test]
    fn test_channel_prototype_with_actions() {
        // Create a custom files.toml with external actions
        let toml_data = r#"
        [metadata]
        name = "custom_files"
        description = "A channel to select files and directories"
        requirements = ["fd", "bat"]

        [source]
        command = ["fd -t f", "fd -t f -H"]

        [preview]
        command = "bat -n --color=always '{}'"
        env = { BAT_THEME = "ansi" }

        [keybindings]
        shortcut = "f1"
        f8 = "actions:thebatman"
        f9 = "actions:lsman"

        [actions.thebatman]
        description = "cats the file"
        command = "bat '{}'"
        env = { BAT_THEME = "ansi" }

        [actions.lsman]
        description = "show stats"
        command = "ls '{}'"
        "#;

        let prototype: ChannelPrototype = from_str(toml_data).unwrap();

        // Verify basic prototype properties
        assert_eq!(prototype.metadata.name, "custom_files");

        // Verify actions are loaded
        assert_eq!(prototype.actions.len(), 2);
        assert!(prototype.actions.contains_key("thebatman"));
        assert!(prototype.actions.contains_key("lsman"));

        // Verify edit action
        let thebatman = prototype.actions.get("thebatman").unwrap();
        assert_eq!(thebatman.description, Some("cats the file".to_string()));
        assert_eq!(thebatman.command.inner[0].raw(), "bat '{}'");
        assert_eq!(
            thebatman.command.env.get("BAT_THEME"),
            Some(&"ansi".to_string())
        );

        // Verify lsman action
        let lsman = prototype.actions.get("lsman").unwrap();
        assert_eq!(lsman.description, Some("show stats".to_string()));
        assert_eq!(lsman.command.inner[0].raw(), "ls '{}'");
        assert!(lsman.command.env.is_empty());

        // Verify keybindings reference the actions
        let keybindings = prototype.keybindings.as_ref().unwrap();
        assert_eq!(
            keybindings.bindings.get(&Key::F(8)),
            Some(
                &crate::action::Action::ExternalAction(
                    "thebatman".to_string()
                )
                .into()
            )
        );
        assert_eq!(
            keybindings.bindings.get(&Key::F(9)),
            Some(
                &crate::action::Action::ExternalAction("lsman".to_string())
                    .into()
            )
        );
    }
}
