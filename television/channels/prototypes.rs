use anyhow::Result;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use which::which;

use crate::config::Binding;
use crate::{
    config::KeyBindings,
    screen::layout::{InputPosition, Orientation},
};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_with::{OneOrMany, serde_as};
use string_pipeline::MultiTemplate;

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
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct ChannelKeyBindings {
    /// Optional channel specific shortcut that, when pressed, switches directly to this channel.
    #[serde(default)]
    pub shortcut: Option<Binding>,
    /// Regular action -> binding mappings living at channel level.
    #[serde(flatten)]
    #[serde(default)]
    pub bindings: KeyBindings,
}

impl ChannelKeyBindings {
    pub fn channel_shortcut(&self) -> Option<&Binding> {
        self.shortcut.as_ref()
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChannelPrototype {
    pub metadata: Metadata,
    #[serde(rename = "source")]
    pub source: SourceSpec,
    #[serde(default, rename = "preview")]
    pub preview: Option<PreviewSpec>,
    #[serde(default, rename = "ui")]
    pub ui: Option<UiSpec>,
    #[serde(default)]
    pub keybindings: Option<ChannelKeyBindings>,
    /// Watch interval in seconds for automatic reloading (0 = disabled)
    #[serde(default)]
    pub watch: f64,
    // actions: Vec<Action>,
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
                display: None,
                output: None,
            },
            preview: None,
            ui: None,
            keybindings: None,
            watch: 0.0,
        }
    }

    pub fn stdin(preview: Option<PreviewSpec>) -> Self {
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
                    interactive: false,
                    env: FxHashMap::default(),
                },
                display: None,
                output: None,
            },
            preview,
            ui: None,
            keybindings: None,
            watch: 0.0,
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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SourceSpec {
    #[serde(flatten)]
    pub command: CommandSpec,
    #[serde(default)]
    pub display: Option<Template>,
    #[serde(default)]
    pub output: Option<Template>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PreviewSpec {
    #[serde(flatten)]
    pub command: CommandSpec,
    #[serde(default)]
    pub offset: Option<Template>,
}

impl PreviewSpec {
    pub fn new(command: CommandSpec, offset: Option<Template>) -> Self {
        Self { command, offset }
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
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UiSpec {
    #[serde(default)]
    pub ui_scale: Option<u16>,
    #[serde(default)]
    pub show_help_bar: Option<bool>,
    #[serde(default)]
    pub show_preview_panel: Option<bool>,
    // `layout` is clearer for the user but collides with the overall `Layout` type.
    #[serde(rename = "layout", alias = "orientation", default)]
    pub orientation: Option<Orientation>,
    #[serde(default)]
    pub input_bar_position: Option<InputPosition>,
    #[serde(default)]
    pub preview_size: Option<u16>,
    #[serde(default)]
    pub input_header: Option<Template>,
    #[serde(default)]
    pub preview_header: Option<Template>,
    #[serde(default)]
    pub preview_footer: Option<Template>,
}

pub const DEFAULT_PROTOTYPE_NAME: &str = "files";

#[cfg(test)]
mod tests {
    use crate::{action::Action, config::Binding, event::Key};

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
        show_help_bar = false
        show_preview_panel = true
        input_bar_position = "bottom"
        preview_size = 66
        input_header = "Input: {}"
        preview_header = "Preview: {}"
        preview_footer = "Press 'q' to quit"

        [keybindings]
        quit = ["esc", "ctrl-c"]
        select_next_entry = ["down", "ctrl-n", "ctrl-j"]
        select_prev_entry = ["up", "ctrl-p", "ctrl-k"]
        confirm_selection = "enter"
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
        assert!(!(ui.show_help_bar.unwrap()));
        assert!(ui.show_preview_panel.unwrap());
        assert_eq!(ui.input_bar_position, Some(InputPosition::Bottom));
        assert_eq!(ui.preview_size, Some(66));
        assert_eq!(ui.input_header.as_ref().unwrap().raw(), "Input: {}");
        assert_eq!(ui.preview_header.as_ref().unwrap().raw(), "Preview: {}");
        assert_eq!(
            ui.preview_footer.as_ref().unwrap().raw(),
            "Press 'q' to quit"
        );

        let keybindings = prototype.keybindings.unwrap();
        assert_eq!(
            keybindings.bindings.0.get(&Action::Quit),
            Some(&Binding::MultipleKeys(vec![Key::Esc, Key::Ctrl('c')]))
        );
        assert_eq!(
            keybindings.bindings.0.get(&Action::SelectNextEntry),
            Some(&Binding::MultipleKeys(vec![
                Key::Down,
                Key::Ctrl('n'),
                Key::Ctrl('j')
            ]))
        );
        assert_eq!(
            keybindings.bindings.0.get(&Action::SelectPrevEntry),
            Some(&Binding::MultipleKeys(vec![
                Key::Up,
                Key::Ctrl('p'),
                Key::Ctrl('k')
            ]))
        );
        assert_eq!(
            keybindings.bindings.0.get(&Action::ConfirmSelection),
            Some(&Binding::SingleKey(Key::Enter))
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
        preview_footer = "Press 'q' to quit"
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
        assert!(ui.show_help_bar.is_none());
        assert!(ui.show_preview_panel.is_none());
        assert!(ui.input_bar_position.is_none());
        assert!(ui.preview_size.is_none());
        assert!(ui.input_header.is_none());
        assert!(ui.preview_header.is_none());
        assert_eq!(
            ui.preview_footer.as_ref().unwrap().raw(),
            "Press 'q' to quit"
        );
    }
}
