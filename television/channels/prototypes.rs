use std::fmt::{self, Display, Formatter};

use crate::{
    config::KeyBindings,
    screen::layout::{InputPosition, Orientation},
};
use rustc_hash::FxHashMap;
use string_pipeline::MultiTemplate;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CommandSpec {
    #[serde(
        rename = "command",
        deserialize_with = "deserialize_template",
        serialize_with = "serialize_template"
    )]
    pub inner: MultiTemplate,
    #[serde(default)]
    pub interactive: bool,
    #[serde(default)]
    pub env: FxHashMap<String, String>,
}

impl Display for CommandSpec {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inner.template_string())
    }
}

impl CommandSpec {
    pub fn new(
        inner: MultiTemplate,
        interactive: bool,
        env: FxHashMap<String, String>,
    ) -> Self {
        Self {
            inner,
            interactive,
            env,
        }
    }
}

fn serialize_template<S>(
    command: &MultiTemplate,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let raw = command.template_string();
    serializer.serialize_str(raw)
}

#[allow(clippy::ref_option)]
fn serialize_maybe_template<S>(
    command: &Option<MultiTemplate>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match command {
        Some(template) => serialize_template(template, serializer),
        None => serializer.serialize_none(),
    }
}

fn deserialize_template<'de, D>(
    deserializer: D,
) -> Result<MultiTemplate, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: String = serde::Deserialize::deserialize(deserializer)?;
    MultiTemplate::parse(&raw).map_err(serde::de::Error::custom)
}

fn deserialize_maybe_template<'de, D>(
    deserializer: D,
) -> Result<Option<MultiTemplate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: Option<String> = serde::Deserialize::deserialize(deserializer)?;
    match raw {
        Some(template) => MultiTemplate::parse(&template)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
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
    pub keybindings: Option<KeyBindings>,
    // actions: Vec<Action>,
}

impl ChannelPrototype {
    // FIXME: is this really needed? maybe tests should use a toml spec directly
    pub fn new(name: &str, source_command: &str) -> Self {
        Self {
            metadata: Metadata {
                name: name.to_string(),
                description: Some(String::new()),
                requirements: vec![],
            },
            source: SourceSpec {
                command: CommandSpec {
                    inner: MultiTemplate::parse(source_command).expect(
                        "Failed to parse source command MultiTemplate",
                    ),
                    interactive: false,
                    env: FxHashMap::default(),
                },
                display: None,
                output: None,
            },
            preview: None,
            ui: None,
            keybindings: None,
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
                    inner: MultiTemplate::parse("cat").unwrap(),
                    interactive: false,
                    env: FxHashMap::default(),
                },
                display: None,
                output: None,
            },
            preview,
            ui: None,
            keybindings: None,
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
    requirements: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SourceSpec {
    #[serde(flatten)]
    pub command: CommandSpec,
    #[serde(
        default,
        deserialize_with = "deserialize_maybe_template",
        serialize_with = "serialize_maybe_template"
    )]
    pub display: Option<MultiTemplate>,
    #[serde(
        default,
        deserialize_with = "deserialize_maybe_template",
        serialize_with = "serialize_maybe_template"
    )]
    pub output: Option<MultiTemplate>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PreviewSpec {
    #[serde(flatten)]
    pub command: CommandSpec,
    #[serde(
        default,
        deserialize_with = "deserialize_maybe_template",
        serialize_with = "serialize_maybe_template"
    )]
    pub offset: Option<MultiTemplate>,
}

impl PreviewSpec {
    pub fn new(command: CommandSpec, offset: Option<MultiTemplate>) -> Self {
        Self { command, offset }
    }

    pub fn from_str_command(command: &str) -> Self {
        Self {
            command: CommandSpec {
                inner: MultiTemplate::parse(command)
                    .expect("Failed to parse preview command"),
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
    #[serde(rename = "layout", default)]
    pub orientation: Option<Orientation>,
    #[serde(default)]
    pub input_bar_position: Option<InputPosition>,
}

pub const DEFAULT_PROTOTYPE_NAME: &str = "files";

#[cfg(test)]
mod tests {
    use crate::{action::Action, config::Binding, event::Key};

    use super::*;
    use toml::from_str;

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
        ansi = false
        output = "{}"            # output the full path

        [preview]
        command = "bat -n --color=always {}"
        env = { "BAT_THEME" = "ansi" }
        interactive = false

        [ui]
        layout = "landscape"
        ui_scale = 100
        show_help_bar = false
        show_preview_panel = true
        input_bar_position = "bottom"

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
        assert_eq!(format!("{}", prototype.source.command.inner), "fd -t f");
        assert!(!prototype.source.command.interactive);
        assert_eq!(
            prototype.source.display.unwrap().template_string(),
            "{split:/:-1}"
        );
        assert_eq!(prototype.source.output.unwrap().template_string(), "{}");
        assert_eq!(
            format!("{}", prototype.preview.unwrap().command.inner),
            "bat -n --color=always {}"
        );
        let ui = prototype.ui.unwrap();
        assert_eq!(ui.orientation, Some(Orientation::Landscape));
        assert_eq!(ui.ui_scale, Some(100));
        assert!(!(ui.show_help_bar.unwrap()));
        assert!(ui.show_preview_panel.unwrap());
        assert_eq!(ui.input_bar_position, Some(InputPosition::Bottom));
        let keybindings = prototype.keybindings.unwrap();
        assert_eq!(
            keybindings.0.get(&Action::Quit),
            Some(&Binding::MultipleKeys(vec![Key::Esc, Key::Ctrl('c')]))
        );
        assert_eq!(
            keybindings.0.get(&Action::SelectNextEntry),
            Some(&Binding::MultipleKeys(vec![
                Key::Down,
                Key::Ctrl('n'),
                Key::Ctrl('j')
            ]))
        );
        assert_eq!(
            keybindings.0.get(&Action::SelectPrevEntry),
            Some(&Binding::MultipleKeys(vec![
                Key::Up,
                Key::Ctrl('p'),
                Key::Ctrl('k')
            ]))
        );
        assert_eq!(
            keybindings.0.get(&Action::ConfirmSelection),
            Some(&Binding::SingleKey(Key::Enter))
        );
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
        assert_eq!(format!("{}", prototype.source.command.inner), "fd -t f");
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
        "#;

        let prototype: ChannelPrototype = from_str(toml_data).unwrap();

        assert_eq!(prototype.metadata.name, "files");
        assert_eq!(
            prototype.metadata.description,
            Some("A channel to select files and directories".to_string())
        );
        assert_eq!(format!("{}", prototype.source.command.inner), "fd -t f");
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
    }
}
