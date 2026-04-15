mod common;

use common::*;

/// Really just a sanity check
#[test]
fn tv_version() {
    let pt = phantom();
    let s = tv_with_args(&pt, &["--version"]).start().unwrap();

    s.wait().text("television").until().unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests the `tv list-channels` command.
///
/// We expect this to list all available channels in the cable directory.
#[test]
fn tv_list_channels() {
    let pt = phantom();
    let s = tv_local_config_and_cable_with_args(&pt, &["list-channels"])
        .start()
        .unwrap();

    // Check what's in the cable directory
    let cable_dir_filenames = std::fs::read_dir(DEFAULT_CABLE_DIR)
        .expect("Failed to read cable directory")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            // this is pretty lazy and can be improved later on
            entry.path().extension().and_then(|ext| {
                if ext == "toml" {
                    entry
                        .path()
                        .file_stem()
                        .and_then(|stem| stem.to_str().map(String::from))
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    // Checking for the last channel alphabetically ensures the entire list
    // has been printed.
    s.wait()
        .text(cable_dir_filenames.iter().max().unwrap())
        .timeout_ms(2000)
        .until()
        .unwrap();

    s.wait().exit_code(0).until().unwrap();
}

#[test]
/// This simply tests that the command exits successfully.
fn tv_init_zsh() {
    let pt = phantom();
    let s = tv_local_config_and_cable_with_args(&pt, &["init", "zsh"])
        .start()
        .unwrap();

    s.wait().exit_code(0).until().unwrap();
}
