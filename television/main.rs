use anyhow::Result;
use clap::Parser;
use std::env;
use std::io::{BufWriter, IsTerminal, Write, stdout};
use std::path::PathBuf;
use std::process::exit;
use television::{
    action::Action,
    app::{App, AppOptions},
    cable::{Cable, load_cable},
    channels::prototypes::{
        ChannelPrototype, CommandSpec, PreviewSpec, Template, UiSpec,
    },
    cli::post_process,
    cli::{
        PostProcessedCli,
        args::{Cli, Command},
        guess_channel_from_prompt, list_channels,
    },
    config::{Config, ConfigEnv, merge_keybindings},
    errors::os_error_exit,
    features::{FeatureFlags, Features},
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
    apply_cli_overrides(&args, &mut config);

    // handle subcommands
    debug!("Handling subcommands...");
    if let Some(subcommand) = &args.command {
        handle_subcommand(subcommand, &config)?;
    }

    debug!("Loading cable channels...");
    let cable =
        load_cable(&config.application.cable_dir).unwrap_or_else(|| exit(1));

    // optionally change the working directory
    if let Some(ref working_dir) = args.working_directory {
        set_current_dir(working_dir)
            .unwrap_or_else(|e| os_error_exit(&e.to_string()));
    }

    // determine the channel to use based on the CLI arguments and configuration
    debug!("Determining channel...");
    let channel_prototype =
        determine_channel(&args, &config, readable_stdin, &cable);

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
    let mut app = App::new(
        channel_prototype,
        config,
        args.input.clone(),
        options,
        cable,
    );

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

/// Apply overrides from the CLI arguments to the configuration.
///
/// This function mutates the configuration in place.
fn apply_cli_overrides(args: &PostProcessedCli, config: &mut Config) {
    if let Some(cable_dir) = &args.cable_dir {
        config.application.cable_dir.clone_from(cable_dir);
    }
    if let Some(tick_rate) = args.tick_rate {
        config.application.tick_rate = tick_rate;
    }
    if args.global_history {
        config.application.global_history = true;
    }
    // Handle preview panel flags
    if args.no_preview {
        config.ui.features.disable(FeatureFlags::PreviewPanel);
        config.keybindings.remove(&Action::TogglePreview);
    } else if args.hide_preview {
        config.ui.features.hide(FeatureFlags::PreviewPanel);
    } else if args.show_preview {
        config.ui.features.enable(FeatureFlags::PreviewPanel);
    }

    if let Some(ps) = args.preview_size {
        config.ui.preview_panel.size = ps;
    }

    // Handle status bar flags
    if args.no_status_bar {
        config.ui.features.disable(FeatureFlags::StatusBar);
        config.keybindings.remove(&Action::ToggleStatusBar);
    } else if args.hide_status_bar {
        config.ui.features.hide(FeatureFlags::StatusBar);
    } else if args.show_status_bar {
        config.ui.features.enable(FeatureFlags::StatusBar);
    }

    // Handle remote control flags
    if args.no_remote {
        config.ui.features.disable(FeatureFlags::RemoteControl);
        config.keybindings.remove(&Action::ToggleRemoteControl);
    } else if args.hide_remote {
        config.ui.features.hide(FeatureFlags::RemoteControl);
    } else if args.show_remote {
        config.ui.features.enable(FeatureFlags::RemoteControl);
    }

    // Handle help panel flags
    if args.no_help_panel {
        config.ui.features.disable(FeatureFlags::HelpPanel);
        config.keybindings.remove(&Action::ToggleHelp);
    } else if args.hide_help_panel {
        config.ui.features.hide(FeatureFlags::HelpPanel);
    } else if args.show_help_panel {
        config.ui.features.enable(FeatureFlags::HelpPanel);
    }

    if let Some(keybindings) = &args.keybindings {
        config.keybindings =
            merge_keybindings(config.keybindings.clone(), keybindings);
    }
    config.ui.ui_scale = args.ui_scale;
    if let Some(input_header) = &args.input_header {
        if let Ok(t) = Template::parse(input_header) {
            config.ui.input_header = Some(t);
        }
    }
    if let Some(preview_header) = &args.preview_header {
        if let Ok(t) = Template::parse(preview_header) {
            config.ui.preview_panel.header = Some(t);
        }
    }
    if let Some(preview_footer) = &args.preview_footer {
        if let Ok(t) = Template::parse(preview_footer) {
            config.ui.preview_panel.footer = Some(t);
        }
    }
    if let Some(layout) = args.layout {
        config.ui.orientation = layout;
    }
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
        Command::UpdateChannels => {
            update_local_channels()?;
            exit(0);
        }
    }
}

/// Creates a stdin channel prototype with optional preview configuration
fn create_stdin_channel(args: &PostProcessedCli) -> ChannelPrototype {
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

    // Configure UI features based on whether preview command is available
    let mut features = Features::default();
    if args.preview_command_override.is_some() {
        features.enable(FeatureFlags::PreviewPanel);
    } else {
        features.disable(FeatureFlags::PreviewPanel);
    }

    // Set UI specification to properly control feature visibility
    prototype.ui = Some(UiSpec {
        ui_scale: None,
        features: Some(features),
        orientation: None,
        input_bar_position: None,
        input_header: None,
        status_bar: None,
        preview_panel: None,
        help_panel: None,
        remote_control: None,
    });

    prototype
}

/// Default header for ad-hoc channels when no custom header is provided
const DEFAULT_ADHOC_CHANNEL_HEADER: &str = "Custom Channel";

/// Creates an ad-hoc channel prototype from CLI arguments
fn create_adhoc_channel(args: &PostProcessedCli) -> ChannelPrototype {
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

    // Configure features based on available commands
    let mut features = Features::default();
    if args.preview_command_override.is_some() {
        features.enable(FeatureFlags::PreviewPanel);
    } else {
        features.disable(FeatureFlags::PreviewPanel);
    }

    // Set UI specification
    prototype.ui = Some(UiSpec {
        ui_scale: None,
        features: Some(features),
        orientation: None,
        input_bar_position: None,
        input_header: Some(input_header),
        status_bar: None,
        preview_panel: None,
        help_panel: None,
        remote_control: None,
    });

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
        input_bar_position: None,
        input_header: None,
        preview_panel: None,
        status_bar: None,
        help_panel: None,
        remote_control: None,
    });

    // Apply input header override
    if let Some(input_header_str) = &args.input_header {
        if let Ok(template) = Template::parse(input_header_str) {
            ui_spec.input_header = Some(template);
            ui_changes_needed = true;
        }
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
    cable: &Cable,
) -> ChannelPrototype {
    // Determine the base channel prototype
    let mut channel_prototype = if readable_stdin {
        create_stdin_channel(args)
    } else if let Some(prompt) = &args.autocomplete_prompt {
        debug!("Using autocomplete prompt: {:?}", prompt);
        let prototype = guess_channel_from_prompt(
            prompt,
            &config.shell_integration.commands,
            &config.shell_integration.fallback_channel,
            cable,
        );
        debug!("Using guessed channel: {:?}", prototype);
        prototype
    } else if args.channel.is_none() && args.source_command_override.is_some()
    {
        create_adhoc_channel(args)
    } else {
        let channel_name = args
            .channel
            .as_ref()
            .unwrap_or(&config.application.default_channel);
        debug!("Using channel: {:?}", channel_name);
        cable.get_channel(channel_name)
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
    use television::channels::prototypes::{
        ChannelPrototype, CommandSpec, PreviewSpec, Template,
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
            determine_channel(args, config, readable_stdin, &channels);

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

        let channel = determine_channel(&args, &config, true, &cable);

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

        let channel = determine_channel(&args, &config, true, &cable);

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

        let channel = determine_channel(
            &args,
            &config,
            false,
            &Cable::from_prototypes(vec![]),
        );

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
            ui_spec.input_header,
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
            ..Default::default()
        };
        apply_cli_overrides(&args, &mut config);

        assert_eq!(config.application.tick_rate, 100_f64);
        assert!(!config.ui.features.is_enabled(FeatureFlags::PreviewPanel));
        assert_eq!(
            config.ui.input_header,
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
            input_bar_position: None,
            input_header: Some(Template::parse("Original Header").unwrap()),
            preview_panel: Some(television::config::ui::PreviewPanelConfig {
                size: 50,
                header: Some(
                    Template::parse("Original Preview Header").unwrap(),
                ),
                footer: Some(
                    Template::parse("Original Preview Footer").unwrap(),
                ),
                scrollbar: false,
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
            ..Default::default()
        };
        let config = Config::default();

        let result_channel = determine_channel(&args, &config, false, &cable);

        // Verify that CLI arguments overrode the channel prototype's UI settings
        assert!(result_channel.ui.is_some());
        let ui_spec = result_channel.ui.as_ref().unwrap();

        assert_eq!(
            ui_spec.input_header,
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
    }
}
