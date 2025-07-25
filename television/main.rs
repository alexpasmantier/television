use anyhow::Result;
use clap::Parser;
use std::env;
use std::io::{BufWriter, IsTerminal, Write, stdout};
use std::path::PathBuf;
use std::process::exit;
use television::{
    app::{App, AppOptions},
    cable::{Cable, cable_empty_exit, load_cable},
    channels::prototypes::{
        ChannelPrototype, CommandSpec, PreviewSpec, Template, UiSpec,
    },
    cli::{
        PostProcessedCli,
        args::{Cli, Command},
        guess_channel_from_prompt, list_channels, post_process,
    },
    config::{Config, ConfigEnv, ui::InputBarConfig},
    errors::os_error_exit,
    features::FeatureFlags,
    gh::update_local_channels,
    television::Mode,
    utils::clipboard::CLIPBOARD,
    utils::{
        shell::{
            Shell, completion_script, render_autocomplete_script_template,
        },
        stdin::is_readable_stdin,
    },
};
use tracing::{debug, info};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    television::errors::init()?;
    television::logging::init()?;

    debug!("\n\n====  NEW SESSION  =====\n");

    // process the CLI arguments
    let cli = Cli::parse();
    debug!("CLI: {:?}", cli);

    let readable_stdin = is_readable_stdin();

    let args = post_process(cli, readable_stdin);
    debug!("PostProcessedCli: {:?}", args);

    // load the configuration file
    debug!("Loading configuration...");
    let mut config =
        Config::new(&ConfigEnv::init()?, args.config_file.as_deref())?;

    // override configuration values with provided CLI arguments
    debug!("Applying CLI overrides...");
    config.apply_cli_overrides(&args);

    // handle subcommands
    debug!("Handling subcommands...");
    if let Some(subcommand) = &args.command {
        handle_subcommand(subcommand, &config)?;
    }

    debug!("Loading cable channels...");
    let cable = load_cable(&config.application.cable_dir).unwrap_or_default();

    // optionally change the working directory
    if let Some(ref working_dir) = args.working_directory {
        set_current_dir(working_dir)
            .unwrap_or_else(|e| os_error_exit(&e.to_string()));
    }

    // determine the channel to use based on the CLI arguments and configuration
    debug!("Determining channel...");
    let channel_prototype =
        determine_channel(&args, &config, readable_stdin, Some(&cable));

    CLIPBOARD.with(<_>::default);

    debug!("Creating application...");
    // Determine the effective watch interval (CLI override takes precedence)
    let watch_interval =
        args.watch_interval.unwrap_or(channel_prototype.watch);
    let options = AppOptions::new(
        args.exact,
        args.select_1,
        args.take_1,
        args.take_1_fast,
        args.no_remote,
        args.no_preview,
        args.preview_size,
        config.application.tick_rate,
        watch_interval,
        args.height,
        args.width,
        args.inline,
    );
    let mut app = App::new(channel_prototype, config, options, cable, &args);

    // If the user requested to show the remote control on startup, switch the
    // television into Remote Control mode before the application event loop
    // begins. This mirrors the behaviour of toggling the remote control via
    // the corresponding keybinding after launch, ensuring the panel is
    // visible from the start.
    // TODO: This is a hack, preview is not initialised yet, find a better way to do it.
    if args.show_remote && app.television.remote_control.is_some() {
        app.television.mode = Mode::RemoteControl;
    }

    stdout().flush()?;
    debug!("Running application...");
    let output = app.run(stdout().is_terminal(), false).await?;
    info!("App output: {:?}", output);
    let stdout_handle = stdout().lock();
    let mut bufwriter = BufWriter::new(stdout_handle);
    if let Some(key) = output.expect_key {
        writeln!(bufwriter, "{}", key)?;
    }
    if let Some(entries) = output.selected_entries {
        for entry in &entries {
            writeln!(
                bufwriter,
                "{}",
                entry.output(&app.television.channel.prototype.source.output)
            )?;
        }
    }
    bufwriter.flush()?;
    exit(0);
}

pub fn set_current_dir(path: &PathBuf) -> Result<()> {
    env::set_current_dir(path)?;
    Ok(())
}

pub fn handle_subcommand(command: &Command, config: &Config) -> Result<()> {
    match command {
        Command::ListChannels => {
            list_channels(&config.application.cable_dir);
            exit(0);
        }
        Command::InitShell { shell } => {
            let target_shell = Shell::from(shell);
            // the completion scripts for the various shells are templated
            // so that it's possible to override the keybindings triggering
            // shell autocomplete and command history in tv
            let script = render_autocomplete_script_template(
                target_shell,
                completion_script(target_shell)?,
                &config.shell_integration,
            )?;
            println!("{script}");
            exit(0);
        }
        Command::UpdateChannels { force } => {
            update_local_channels(force)?;
            exit(0);
        }
    }
}

/// Creates a stdin channel prototype with optional preview configuration
fn create_stdin_channel(
    args: &PostProcessedCli,
    config: &Config,
) -> ChannelPrototype {
    debug!("Using stdin channel");
    let stdin_preview =
        args.preview_command_override.as_ref().map(|preview_cmd| {
            PreviewSpec::new(
                CommandSpec::from(preview_cmd.clone()),
                args.preview_offset_override.clone(),
            )
        });

    let mut prototype =
        ChannelPrototype::stdin(stdin_preview, args.source_entry_delimiter);

    // Inherit UI features from global config (which has CLI overrides applied)
    let mut features = config.ui.features.clone();
    if args.preview_command_override.is_some() {
        features.enable(FeatureFlags::PreviewPanel);
    } else {
        features.disable(FeatureFlags::PreviewPanel);
    }

    // Set UI specification to properly control feature visibility
    let mut ui_spec = UiSpec::from(&config.ui);
    ui_spec.features = Some(features);
    prototype.ui = Some(ui_spec);

    prototype
}

/// Default header for ad-hoc channels when no custom header is provided
const DEFAULT_ADHOC_CHANNEL_HEADER: &str = "Custom Channel";

/// Creates an ad-hoc channel prototype from CLI arguments
fn create_adhoc_channel(
    args: &PostProcessedCli,
    config: &Config,
) -> ChannelPrototype {
    debug!("Creating ad-hoc channel with source command override");
    let source_cmd = args.source_command_override.as_ref().unwrap();

    // Create base prototype
    let mut prototype = ChannelPrototype::new("custom", source_cmd.raw());

    // Determine input header
    let input_header = args
        .input_header
        .as_ref()
        .and_then(|ih| Template::parse(ih).ok())
        .unwrap_or_else(|| {
            Template::parse(DEFAULT_ADHOC_CHANNEL_HEADER).unwrap()
        });

    // Inherit UI features from global config (which has CLI overrides applied)
    let mut features = config.ui.features.clone();
    if args.preview_command_override.is_some() {
        features.enable(FeatureFlags::PreviewPanel);
    } else {
        features.disable(FeatureFlags::PreviewPanel);
    }

    // Set UI specification
    let mut ui_spec = UiSpec::from(&config.ui);
    let input_bar = ui_spec
        .input_bar
        .get_or_insert_with(InputBarConfig::default);
    input_bar.header = Some(input_header);
    ui_spec.features = Some(features);
    prototype.ui = Some(ui_spec);

    prototype
}

/// Applies source-related CLI overrides to the channel prototype
fn apply_source_overrides(
    prototype: &mut ChannelPrototype,
    args: &PostProcessedCli,
) {
    if let Some(source_cmd) = &args.source_command_override {
        prototype.source.command = CommandSpec::from(source_cmd.clone());
    }
    if let Some(source_display) = &args.source_display_override {
        prototype.source.display = Some(source_display.clone());
    }
    if let Some(source_output) = &args.source_output_override {
        prototype.source.output = Some(source_output.clone());
    }
    if args.ansi {
        prototype.source.ansi = true;
    }
}

/// Applies preview-related CLI overrides to the channel prototype
fn apply_preview_overrides(
    prototype: &mut ChannelPrototype,
    args: &PostProcessedCli,
) {
    if let Some(preview_cmd) = &args.preview_command_override {
        if let Some(ref mut preview) = prototype.preview {
            preview.command = CommandSpec::from(preview_cmd.clone());
        } else {
            prototype.preview = Some(PreviewSpec::new(
                CommandSpec::from(preview_cmd.clone()),
                None,
            ));
        }
    }

    if let Some(preview_offset) = &args.preview_offset_override {
        if let Some(ref mut preview) = prototype.preview {
            preview.offset = Some(preview_offset.clone());
        }
    }
}

/// Applies UI-related CLI overrides to the channel prototype
fn apply_ui_overrides(
    prototype: &mut ChannelPrototype,
    args: &PostProcessedCli,
) {
    let mut ui_changes_needed = false;
    let mut ui_spec = prototype.ui.clone().unwrap_or(UiSpec {
        ui_scale: None,
        features: None,
        orientation: None,
        input_bar: None,
        preview_panel: None,
        results_panel: None,
        status_bar: None,
        help_panel: None,
        remote_control: None,
    });

    // Apply input header override
    if let Some(input_header_str) = &args.input_header {
        if let Ok(template) = Template::parse(input_header_str) {
            let input_bar = ui_spec
                .input_bar
                .get_or_insert_with(InputBarConfig::default);
            input_bar.header = Some(template);
            ui_changes_needed = true;
        }
    }

    // Apply input prompt override
    if let Some(input_prompt_str) = &args.input_prompt {
        let input_bar = ui_spec
            .input_bar
            .get_or_insert_with(InputBarConfig::default);
        input_bar.prompt.clone_from(input_prompt_str);
        ui_changes_needed = true;
    }

    // Apply input bar border override
    if let Some(input_border) = args.input_border {
        let input_bar = ui_spec
            .input_bar
            .get_or_insert_with(InputBarConfig::default);
        input_bar.border_type = input_border;
        ui_changes_needed = true;
    }

    // Apply input bar padding override
    if let Some(input_padding) = &args.input_padding {
        let input_bar = ui_spec
            .input_bar
            .get_or_insert_with(InputBarConfig::default);
        input_bar.padding = *input_padding;
        ui_changes_needed = true;
    }

    // Apply preview panel border override
    if let Some(preview_border) = args.preview_border {
        let preview_panel = ui_spec.preview_panel.get_or_insert_with(
            television::config::ui::PreviewPanelConfig::default,
        );
        preview_panel.border_type = preview_border;
        ui_changes_needed = true;
    }

    // Apply preview panel padding override
    if let Some(preview_padding) = &args.preview_padding {
        let preview_panel = ui_spec.preview_panel.get_or_insert_with(
            television::config::ui::PreviewPanelConfig::default,
        );
        preview_panel.padding = *preview_padding;
        ui_changes_needed = true;
    }

    // Apply results panel border override
    if let Some(results_border) = args.results_border {
        let results_panel = ui_spec.results_panel.get_or_insert_with(
            television::config::ui::ResultsPanelConfig::default,
        );
        results_panel.border_type = results_border;
        ui_changes_needed = true;
    }

    // Apply results panel padding override
    if let Some(results_padding) = &args.results_padding {
        let results_panel = ui_spec.results_panel.get_or_insert_with(
            television::config::ui::ResultsPanelConfig::default,
        );
        results_panel.padding = *results_padding;
        ui_changes_needed = true;
    }

    // Apply layout/orientation override
    if let Some(layout) = args.layout {
        ui_spec.orientation = Some(layout);
        ui_changes_needed = true;
    }

    // Apply preview panel overrides (header and footer)
    if args.preview_header.is_some() || args.preview_footer.is_some() {
        let mut preview_panel =
            ui_spec.preview_panel.clone().unwrap_or_default();

        if let Some(preview_header_str) = &args.preview_header {
            if let Ok(template) = Template::parse(preview_header_str) {
                preview_panel.header = Some(template);
            }
        }

        if let Some(preview_footer_str) = &args.preview_footer {
            if let Ok(template) = Template::parse(preview_footer_str) {
                preview_panel.footer = Some(template);
            }
        }

        ui_spec.preview_panel = Some(preview_panel);
        ui_changes_needed = true;
    }

    // Apply the UI specification if any changes were made
    if ui_changes_needed {
        prototype.ui = Some(ui_spec);
    }
}

/// Determines which channel prototype to use based on CLI arguments and configuration.
///
/// This function handles multiple modes of operation:
/// 1. **Stdin mode**: When stdin is readable, creates a stdin channel
/// 2. **Autocomplete mode**: When autocomplete prompt is provided, guesses channel from prompt
/// 3. **Ad-hoc mode**: When no channel is specified but source command is provided
/// 4. **Channel mode**: Uses a named channel from CLI args or config default
///
/// After determining the base channel, it applies all relevant CLI overrides.
pub fn determine_channel(
    args: &PostProcessedCli,
    config: &Config,
    readable_stdin: bool,
    cable: Option<&Cable>,
) -> ChannelPrototype {
    // Determine the base channel prototype
    let mut channel_prototype = if readable_stdin {
        create_stdin_channel(args, config)
    } else if let Some(prompt) = &args.autocomplete_prompt {
        if cable.is_none() {
            cable_empty_exit()
        }
        debug!("Using autocomplete prompt: {:?}", prompt);
        let prototype = guess_channel_from_prompt(
            prompt,
            &config.shell_integration.commands,
            &config.shell_integration.fallback_channel,
            cable.unwrap(),
        );
        debug!("Using guessed channel: {:?}", prototype);
        prototype
    } else if args.channel.is_none() && args.source_command_override.is_some()
    {
        create_adhoc_channel(args, config)
    } else {
        if cable.is_none() {
            cable_empty_exit()
        }
        let channel_name = args
            .channel
            .as_ref()
            .unwrap_or(&config.application.default_channel);
        debug!("Using channel: {:?}", channel_name);
        cable.unwrap().get_channel(channel_name)
    };

    // Apply CLI overrides to the prototype
    apply_source_overrides(&mut channel_prototype, args);
    apply_preview_overrides(&mut channel_prototype, args);
    apply_ui_overrides(&mut channel_prototype, args);

    // Apply watch interval override
    if let Some(watch_interval) = args.watch_interval {
        channel_prototype.watch = watch_interval;
    }

    channel_prototype
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashMap;
    use television::{
        channels::prototypes::{
            ChannelPrototype, CommandSpec, PreviewSpec, Template,
        },
        config::ui::{BorderType, InputBarConfig, Padding},
        screen::layout::InputPosition,
    };

    use super::*;

    fn assert_is_correct_channel(
        args: &PostProcessedCli,
        config: &Config,
        readable_stdin: bool,
        expected_channel: &ChannelPrototype,
        cable_channels: Option<Cable>,
    ) {
        let channels: Cable =
            cable_channels.unwrap_or(Cable::from_prototypes(vec![
                ChannelPrototype::new("files", "fd -t f"),
                ChannelPrototype::new("dirs", "ls"),
                ChannelPrototype::new("git", "git status"),
            ]));
        let channel =
            determine_channel(args, config, readable_stdin, Some(&channels));

        assert_eq!(
            channel.metadata.name, expected_channel.metadata.name,
            "Expected {:?} but got {:?}",
            expected_channel.metadata.name, channel.metadata.name
        );
    }

    #[test]
    /// Test that the channel is stdin when stdin is readable
    fn test_determine_channel_readable_stdin() {
        let args = PostProcessedCli::default();
        let config = Config::default();
        assert_is_correct_channel(
            &args,
            &config,
            true,
            &ChannelPrototype::new("stdin", "cat"),
            None,
        );
    }

    #[test]
    fn test_determine_channel_stdin_disables_preview_without_command() {
        let args = PostProcessedCli::default();
        let config = Config::default();
        let cable = Cable::from_prototypes(vec![]);

        let channel = determine_channel(&args, &config, true, Some(&cable));

        assert_eq!(channel.metadata.name, "stdin");
        assert!(channel.preview.is_none()); // No preview spec should be created

        // Check that preview feature is explicitly disabled
        assert!(channel.ui.is_some());
        let ui_spec = channel.ui.as_ref().unwrap();
        assert!(ui_spec.features.is_some());
        let features = ui_spec.features.as_ref().unwrap();
        assert!(!features.is_enabled(FeatureFlags::PreviewPanel));
        assert!(!features.is_visible(FeatureFlags::PreviewPanel));
    }

    #[test]
    fn test_determine_channel_stdin_enables_preview_with_command() {
        let args = PostProcessedCli {
            preview_command_override: Some(Template::parse("cat {}").unwrap()),
            ..Default::default()
        };
        let config = Config::default();
        let cable = Cable::from_prototypes(vec![]);

        let channel = determine_channel(&args, &config, true, Some(&cable));

        assert_eq!(channel.metadata.name, "stdin");
        assert!(channel.preview.is_some()); // Preview spec should be created

        // Check that preview feature is enabled and visible
        assert!(channel.ui.is_some());
        let ui_spec = channel.ui.as_ref().unwrap();
        assert!(ui_spec.features.is_some());
        let features = ui_spec.features.as_ref().unwrap();
        assert!(features.is_enabled(FeatureFlags::PreviewPanel));
        assert!(features.is_visible(FeatureFlags::PreviewPanel));
    }

    #[test]
    fn test_determine_channel_autocomplete_prompt() {
        let autocomplete_prompt = Some("cd".to_string());
        let expected_channel = ChannelPrototype::new("dirs", "ls {}");
        let args = PostProcessedCli {
            autocomplete_prompt,
            ..Default::default()
        };
        let mut config = Config {
            shell_integration:
                television::config::shell_integration::ShellIntegrationConfig {
                    fallback_channel: "files".to_string(),
                    commands: FxHashMap::default(),
                    channel_triggers: {
                        let mut m = FxHashMap::default();
                        m.insert("dirs".to_string(), vec!["cd".to_string()]);
                        m
                    },
                    keybindings: FxHashMap::default(),
                },
            ..Default::default()
        };
        config.shell_integration.merge_triggers();

        assert_is_correct_channel(
            &args,
            &config,
            false,
            &expected_channel,
            None,
        );
    }

    #[test]
    fn test_determine_channel_standard_case() {
        let channel = Some(String::from("dirs"));
        let args = PostProcessedCli {
            channel,
            ..Default::default()
        };
        let config = Config::default();
        assert_is_correct_channel(
            &args,
            &config,
            false,
            &ChannelPrototype::new("dirs", "ls {}"),
            None,
        );
    }

    #[test]
    fn test_determine_channel_config_fallback() {
        let args = PostProcessedCli {
            channel: None,
            ..Default::default()
        };
        let mut config = Config::default();
        config.application.default_channel = String::from("dirs");
        assert_is_correct_channel(
            &args,
            &config,
            false,
            &ChannelPrototype::new("dirs", "ls"),
            None,
        );
    }

    #[test]
    fn test_determine_channel_with_cli_preview() {
        let preview_command = Template::parse("echo hello").unwrap();
        let preview_spec = PreviewSpec::new(
            CommandSpec::new(
                vec![preview_command.clone()],
                false,
                FxHashMap::default(),
            ),
            None,
        );

        let args = PostProcessedCli {
            channel: Some(String::from("dirs")),
            preview_command_override: Some(preview_command),
            ..Default::default()
        };
        let config = Config::default();

        let expected_prototype = ChannelPrototype::new("dirs", "ls")
            .with_preview(Some(preview_spec));

        assert_is_correct_channel(
            &args,
            &config,
            false,
            &expected_prototype,
            None,
        );
    }

    #[test]
    fn test_determine_channel_adhoc_with_source_command() {
        let args = PostProcessedCli {
            channel: None,
            source_command_override: Some(
                Template::parse("fd -t f -H").unwrap(),
            ),
            ..Default::default()
        };
        let config = Config::default();

        let channel =
            determine_channel(&args, &config, false, Some(&Cable::default()));

        assert_eq!(channel.metadata.name, "custom");
        assert_eq!(channel.source.command.inner[0].raw(), "fd -t f -H");

        // Check that UI options are set using the new features system
        assert!(channel.ui.is_some());
        let ui_spec = channel.ui.as_ref().unwrap();
        assert!(ui_spec.features.is_some());
        let features = ui_spec.features.as_ref().unwrap();
        // Preview should be disabled since no preview command was provided
        assert!(!features.is_enabled(FeatureFlags::PreviewPanel));
        assert_eq!(
            ui_spec.input_bar.as_ref().unwrap().header,
            Some(Template::parse("Custom Channel").unwrap())
        );
    }

    #[test]
    fn test_apply_cli_overrides() {
        let mut config = Config::default();
        let args = PostProcessedCli {
            tick_rate: Some(100_f64),
            no_preview: true,
            input_header: Some("Input Header".to_string()),
            preview_header: Some("Preview Header".to_string()),
            preview_footer: Some("Preview Footer".to_string()),
            preview_border: Some(BorderType::Thick),
            input_border: Some(BorderType::Thick),
            results_border: Some(BorderType::Thick),
            input_padding: Some(Padding::new(1, 2, 3, 4)),
            preview_padding: Some(Padding::new(5, 6, 7, 8)),
            results_padding: Some(Padding::new(9, 10, 11, 12)),
            ..Default::default()
        };
        config.apply_cli_overrides(&args);

        assert_eq!(config.application.tick_rate, 100_f64);
        assert!(!config.ui.features.is_enabled(FeatureFlags::PreviewPanel));
        assert_eq!(
            config.ui.input_bar.header,
            Some(Template::parse("Input Header").unwrap())
        );
        assert_eq!(
            config.ui.preview_panel.header,
            Some(Template::parse("Preview Header").unwrap())
        );
        assert_eq!(
            config.ui.preview_panel.footer,
            Some(Template::parse("Preview Footer").unwrap())
        );
        assert_eq!(config.ui.preview_panel.border_type, BorderType::Thick);
        assert_eq!(config.ui.input_bar.border_type, BorderType::Thick);
        assert_eq!(config.ui.results_panel.border_type, BorderType::Thick);
        assert_eq!(config.ui.input_bar.padding, Padding::new(1, 2, 3, 4));
        assert_eq!(config.ui.preview_panel.padding, Padding::new(5, 6, 7, 8));
        assert_eq!(
            config.ui.results_panel.padding,
            Padding::new(9, 10, 11, 12)
        );
    }

    #[test]
    fn test_determine_channel_cli_ui_overrides() {
        use television::screen::layout::Orientation;

        // Create a channel with default UI settings
        let mut channel_prototype = ChannelPrototype::new("test", "ls");
        // Set some initial UI values that should be overridden
        channel_prototype.ui = Some(UiSpec {
            ui_scale: None,
            features: None,
            orientation: Some(Orientation::Portrait),
            input_bar: Some(InputBarConfig {
                position: InputPosition::default(),
                header: Some(Template::parse("Original Header").unwrap()),
                prompt: ">".to_string(),
                border_type: BorderType::Thick,
                padding: Padding::uniform(1),
            }),
            preview_panel: Some(television::config::ui::PreviewPanelConfig {
                size: 60,
                header: Some(
                    Template::parse("Original Preview Header").unwrap(),
                ),
                footer: Some(
                    Template::parse("Original Preview Footer").unwrap(),
                ),
                scrollbar: false,
                border_type: BorderType::Thick,
                padding: Padding::uniform(2),
            }),
            results_panel: Some(television::config::ui::ResultsPanelConfig {
                border_type: BorderType::Thick,
                padding: Padding::uniform(2),
            }),
            status_bar: None,
            help_panel: None,
            remote_control: None,
        });

        let cable = Cable::from_prototypes(vec![channel_prototype]);

        // Test CLI arguments that should override channel settings
        let args = PostProcessedCli {
            channel: Some("test".to_string()),
            input_header: Some("CLI Input Header".to_string()),
            preview_header: Some("CLI Preview Header".to_string()),
            preview_footer: Some("CLI Preview Footer".to_string()),
            layout: Some(Orientation::Landscape),
            input_border: Some(BorderType::Plain),
            preview_border: Some(BorderType::Plain),
            results_border: Some(BorderType::Plain),
            input_padding: Some(Padding::new(1, 2, 3, 4)),
            preview_padding: Some(Padding::new(5, 6, 7, 8)),
            results_padding: Some(Padding::new(9, 10, 11, 12)),
            ..Default::default()
        };
        let config = Config::default();

        let result_channel =
            determine_channel(&args, &config, false, Some(&cable));

        // Verify that CLI arguments overrode the channel prototype's UI settings
        assert!(result_channel.ui.is_some());
        let ui_spec = result_channel.ui.as_ref().unwrap();

        assert_eq!(
            ui_spec.input_bar.as_ref().unwrap().header,
            Some(Template::parse("CLI Input Header").unwrap())
        );
        assert_eq!(ui_spec.orientation, Some(Orientation::Landscape));

        assert!(ui_spec.preview_panel.is_some());
        let preview_panel = ui_spec.preview_panel.as_ref().unwrap();
        assert_eq!(
            preview_panel.header,
            Some(Template::parse("CLI Preview Header").unwrap())
        );
        assert_eq!(
            preview_panel.footer,
            Some(Template::parse("CLI Preview Footer").unwrap())
        );
        assert_eq!(preview_panel.border_type, BorderType::Plain);
        assert_eq!(preview_panel.padding, Padding::new(5, 6, 7, 8));

        assert_eq!(
            ui_spec.results_panel.as_ref().unwrap().border_type,
            BorderType::Plain
        );
        assert_eq!(
            ui_spec.results_panel.as_ref().unwrap().padding,
            Padding::new(9, 10, 11, 12)
        );

        assert_eq!(
            ui_spec.input_bar.as_ref().unwrap().border_type,
            BorderType::Plain
        );
        assert_eq!(
            ui_spec.input_bar.as_ref().unwrap().padding,
            Padding::new(1, 2, 3, 4)
        );
    }

    #[test]
    fn test_apply_cli_overrides_ui_scale() {
        // Test that the CLI ui_scale override is applied correctly
        let mut config = Config::default();
        let args = PostProcessedCli {
            ui_scale: Some(90),
            ..Default::default()
        };
        config.apply_cli_overrides(&args);

        assert_eq!(config.ui.ui_scale, 90);

        // Test that the config value is used when no CLI override is provided
        let mut config = Config::default();
        config.ui.ui_scale = 70;
        let args = PostProcessedCli::default();
        config.apply_cli_overrides(&args);

        assert_eq!(config.ui.ui_scale, 70);
    }
}
