use std::path::Path;

use rexpect::error::Error;
use rexpect::process::wait::WaitStatus;
use rexpect::spawn;
use television::channels::prototypes::ChannelPrototype;
use tempfile::TempDir;

#[allow(dead_code)]
fn setup_config(content: &str) -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    std::fs::write(&config_path, content).unwrap();
    temp_dir
}

fn setup_cable_channels<D>(channels: Vec<&str>, cable_dir: D)
where
    D: AsRef<Path>,
{
    let cable_dir = cable_dir.as_ref();
    std::fs::create_dir_all(cable_dir).unwrap();
    for channel in channels {
        let name = toml::from_str::<ChannelPrototype>(channel)
            .unwrap()
            .metadata
            .name;
        let channel_path = cable_dir.join(format!("{}.toml", name));
        std::fs::write(&channel_path, channel).unwrap();
    }
}

#[test]
fn tv_version() -> Result<(), Error> {
    let mut p = spawn("./target/debug/tv --version", Some(500))?;
    p.exp_regex("television [0-9]+\\.[0-9]+\\.[0-9]+")?;

    Ok(())
}

#[test]
fn tv_help() -> Result<(), Error> {
    let mut p = spawn("./target/debug/tv --help", Some(500))?;
    p.exp_regex("A cross-platform")?;

    Ok(())
}

const BASIC_FILE_CHANNEL: &str = r#"
    [metadata]
    name = "files"

    [source]
    command = "fd -t f"
"#;

const BASIC_DIR_CHANNEL: &str = r#"
    [metadata]
    name = "dirs"

    [source]
    command = "fd -t d"
"#;

#[test]
fn tv_list_channels() -> Result<(), Error> {
    let temp_dir = TempDir::new().unwrap();
    setup_cable_channels(
        vec![BASIC_FILE_CHANNEL, BASIC_DIR_CHANNEL],
        &temp_dir,
    );

    let mut p = spawn(
        &format!(
            "./target/debug/tv --cable-dir {} list-channels ",
            temp_dir.path().display()
        ),
        Some(500),
    )?;
    p.exp_regex("files")?;
    p.exp_regex("dirs")?;

    Ok(())
}

#[test]
fn tv_init_zsh() -> Result<(), Error> {
    let p = spawn("./target/debug/tv init zsh", Some(500))?;
    // check that the process exits successfully
    if let Ok(w) = p.process.wait() {
        assert_eq!(w, WaitStatus::Exited(w.pid().unwrap(), 0));
    } else {
        panic!("Failed to wait for process");
    }

    Ok(())
}
