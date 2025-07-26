use anyhow::Result;
use clap::Parser;
use std::env;
use std::io::{BufWriter, IsTerminal, Write, stdout};
use std::path::{Path, PathBuf};
use std::process::exit;
use television::channels::prototypes::{Metadata, SourceSpec};
use television::config::shell_integration::ShellIntegrationConfig;
use television::{
    app::{App, AppOptions},
    cable::{Cable, cable_empty_exit, load_cable},
    channels::prototypes::{
        ChannelPrototype, CommandSpec, PreviewSpec, Template, UiSpec,
    },
    cli::{
        ProcessedCli,
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

    let readable_stdin = is_readable_stdin();

    // process CLI arguments
    let cli = ProcessedCli::process(Cli::parse(), readable_stdin);
    debug!("Processed CLI: {:?}", cli);

    // load the configuration file
    debug!("Loading configuration...");
    let mut config =
        Config::new(&ConfigEnv::init()?, cli.config_file.as_deref())?;

    // load cable channels
    debug!("Loading cable channels...");
    let cable_dir = cli
        .cable_dir
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| config.application.cable_dir.clone());
    let cable = load_cable(&cable_dir).unwrap_or_default();

    // handle subcommands
    debug!("Handling subcommands...");
    if let Some(subcommand) = &cli.command {
        handle_subcommand(subcommand, &cable_dir, &config.shell_integration)?;
    }

    // optionally change the working directory
    if let Some(ref working_dir) = cli.workdir {
        set_current_dir(working_dir)
            .unwrap_or_else(|e| os_error_exit(&e.to_string()));
    }

    // TODO: [27/07] this is where I left off
    debug!("Applying CLI overrides...");
    config.apply_cli_overrides(&cli);

    // determine the channel to use based on the CLI arguments and configuration
    debug!("Determining channel...");
    let channel_prototype =
        determine_channel(&cli, &config, readable_stdin, &cable);

    CLIPBOARD.with(<_>::default);

    debug!("Creating application...");
    // Determine the effective watch interval (CLI override takes precedence)
    let watch_interval = cli.watch_interval.unwrap_or(channel_prototype.watch);
    let options = AppOptions::new(
        cli.exact,
        cli.select_1,
        cli.take_1,
        cli.take_1_fast,
        cli.no_remote,
        cli.no_preview,
        cli.preview_size,
        config.application.tick_rate,
        watch_interval,
        cli.height,
        cli.width,
        cli.inline,
    );
    let mut app = App::new(channel_prototype, config, options, cable, &cli);

    // If the user requested to show the remote control on startup, switch the
    // television into Remote Control mode before the application event loop
    // begins. This mirrors the behaviour of toggling the remote control via
    // the corresponding keybinding after launch, ensuring the panel is
    // visible from the start.
    // TODO: This is a hack, preview is not initialised yet, find a better way to do it.
    if cli.show_remote && app.television.remote_control.is_some() {
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

pub fn handle_subcommand(
    command: &Command,
    cable_dir: &Path,
    shell_integration_config: &ShellIntegrationConfig,
) -> Result<()> {
    match command {
        Command::ListChannels => {
            list_channels(cable_dir);
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
                shell_integration_config,
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

/// Creates a stdin channel prototype
fn create_stdin_channel(entry_delimiter: Option<char>) -> ChannelPrototype {
    debug!("Using stdin channel");
    // let stdin_preview = args.preview_command.as_ref().map(|preview_cmd| {
    //     PreviewSpec::new(
    //         CommandSpec::from(preview_cmd.clone()),
    //         args.preview_offset.clone(),
    //     )
    // });

    ChannelPrototype::stdin(entry_delimiter)

    // Inherit UI features from global config (which has CLI overrides applied)
    // let mut features = config.ui.features.clone();
    // if args.preview_command.is_some() {
    //     features.enable(FeatureFlags::PreviewPanel);
    // } else {
    //     features.disable(FeatureFlags::PreviewPanel);
    // }

    // Set UI specification to properly control feature visibility
    // let mut ui_spec = UiSpec::from(&config.ui);
    // ui_spec.features = Some(features);
    // prototype.ui = Some(ui_spec);
}

const CUSTOM_CHANNEL_NAME: &str = "custom channel";
/// Default header for ad-hoc channels when no custom header is provided
const DEFAULT_ADHOC_CHANNEL_HEADER: &str = "Custom Channel";

/// Creates an ad-hoc channel prototype from CLI arguments
fn create_adhoc_channel(args: &ProcessedCli) -> ChannelPrototype {
    debug!("Creating ad-hoc channel");
    let mut prototype = ChannelPrototype::new(
        Metadata {
            name: CUSTOM_CHANNEL_NAME.to_string(),
            description: None,
            requirements: vec![],
        },
        SourceSpec {
            command: CommandSpec::from(args.source_command.clone().unwrap()),
            entry_delimiter: args.source_entry_delimiter,
            ansi: args.ansi,
            display: args.source_display.clone(),
            output: args.source_output.clone(),
        },
    );

    prototype
}

/// Applies source-related CLI overrides to the channel prototype
fn apply_source_overrides(
    prototype: &mut ChannelPrototype,
    args: &ProcessedCli,
) {
    if let Some(source_cmd) = &args.source_command {
        prototype.source.command = CommandSpec::from(source_cmd.clone());
    }
    if let Some(source_display) = &args.source_display {
        prototype.source.display = Some(source_display.clone());
    }
    if let Some(source_output) = &args.source_output {
        prototype.source.output = Some(source_output.clone());
    }
    if args.ansi {
        prototype.source.ansi = true;
    }
}

/// Applies preview-related CLI overrides to the channel prototype
fn apply_preview_overrides(
    prototype: &mut ChannelPrototype,
    args: &ProcessedCli,
) {
    if let Some(preview_cmd) = &args.preview_command {
        if let Some(ref mut preview) = prototype.preview {
            preview.command = CommandSpec::from(preview_cmd.clone());
        } else {
            prototype.preview = Some(PreviewSpec::new(
                CommandSpec::from(preview_cmd.clone()),
                None,
            ));
        }
    }

    if let Some(preview_offset) = &args.preview_offset {
        if let Some(ref mut preview) = prototype.preview {
            preview.offset = Some(preview_offset.clone());
        }
    }
}

/// Applies UI-related CLI overrides to the channel prototype
fn apply_ui_overrides(prototype: &mut ChannelPrototype, args: &ProcessedCli) {
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
    args: &ProcessedCli,
    config: &Config,
    readable_stdin: bool,
    cable: &Cable,
) -> ChannelPrototype {
    // Determine the base channel prototype
    let mut channel_prototype = if readable_stdin {
        debug!("Using stdin channel");
        ChannelPrototype::stdin(args.source_entry_delimiter)
    } else if let Some(prompt) = &args.autocomplete_prompt {
        if cable.is_empty() {
            cable_empty_exit()
        }
        debug!("Using autocomplete prompt: {:?}", prompt);
        let prototype = guess_channel_from_prompt(
            prompt,
            &config.shell_integration,
            cable,
        );
        debug!("Using guessed channel: {:?}", prototype);
        prototype
    } else if args.channel.is_none() && args.source_command.is_some() {
        create_adhoc_channel(args, config)
    } else {
        if cable.is_empty() {
            cable_empty_exit()
        }
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
    use television::{
        channels::prototypes::{
            ChannelPrototype, CommandSpec, PreviewSpec, Template,
        },
        config::ui::{BorderType, InputBarConfig, Padding},
        screen::layout::InputPosition,
    };

    use super::*;

    fn assert_is_correct_channel(
        args: &ProcessedCli,
        config: &Config,
        readable_stdin: bool,
        expected_channel: &ChannelPrototype,
        cable_channels: Option<Cable>,
    ) {
        let channels: Cable =
            cable_channels.unwrap_or(Cable::from_prototypes(vec![
                ChannelPrototype::simple("files", "fd -t f"),
                ChannelPrototype::simple("dirs", "ls"),
                ChannelPrototype::simple("git", "git status"),
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
        let args = ProcessedCli::default();
        let config = Config::default();
        assert_is_correct_channel(
            &args,
            &config,
            true,
            &ChannelPrototype::simple("stdin", "cat"),
            None,
        );
    }

    #[test]
    fn test_determine_channel_stdin_disables_preview_without_command() {
        let args = ProcessedCli::default();
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
        let args = ProcessedCli {
            preview_command: Some(Template::parse("cat {}").unwrap()),
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
        let expected_channel = ChannelPrototype::simple("dirs", "ls {}");
        let args = ProcessedCli {
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
        let args = ProcessedCli {
            channel,
            ..Default::default()
        };
        let config = Config::default();
        assert_is_correct_channel(
            &args,
            &config,
            false,
            &ChannelPrototype::simple("dirs", "ls {}"),
            None,
        );
    }

    #[test]
    fn test_determine_channel_config_fallback() {
        let args = ProcessedCli {
            channel: None,
            ..Default::default()
        };
        let mut config = Config::default();
        config.application.default_channel = String::from("dirs");
        assert_is_correct_channel(
            &args,
            &config,
            false,
            &ChannelPrototype::simple("dirs", "ls"),
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

        let args = ProcessedCli {
            channel: Some(String::from("dirs")),
            preview_command: Some(preview_command),
            ..Default::default()
        };
        let config = Config::default();

        let expected_prototype = ChannelPrototype::simple("dirs", "ls")
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
        let args = ProcessedCli {
            channel: None,
            source_command: Some(Template::parse("fd -t f -H").unwrap()),
            ..Default::default()
        };
        let config = Config::default();

        let channel =
            determine_channel(&args, &config, false, &Cable::default());

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
        let args = ProcessedCli {
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
        let mut channel_prototype = ChannelPrototype::simple("test", "ls");
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
        let args = ProcessedCli {
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

        let result_channel = determine_channel(&args, &config, false, &cable);

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
        let args = ProcessedCli {
            ui_scale: Some(90),
            ..Default::default()
        };
        config.apply_cli_overrides(&args);

        assert_eq!(config.ui.ui_scale, 90);

        // Test that the config value is used when no CLI override is provided
        let mut config = Config::default();
        config.ui.ui_scale = 70;
        let args = ProcessedCli::default();
        config.apply_cli_overrides(&args);

        assert_eq!(config.ui.ui_scale, 70);
    }
}
