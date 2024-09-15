use anyhow::Result;

mod action;
mod app;
mod cli;
mod command;
mod config;
mod display;
mod events;
mod finders;
mod logging;
mod pickers;
mod previewers;
mod sorters;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::parse();
    config::init(&cli)?;
    logging::initialize_logging()?;
    tui::init_panic_hook();

    if cli.print_default_config {
        println!("{}", toml::to_string_pretty(config::get())?);
        return Ok(());
    }

    let tui = tui::init()?;
    let events = events::Events::new();
    app::App::new().run(tui, events).await?;
    tui::restore()?;
    Ok(())
}
