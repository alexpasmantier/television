mod common;

use common::*;

/// Really just a sanity check
#[test]
fn tv_version() {
    let mut tester = PtyTester::new();
    let mut child = tester.spawn_command(tv_with_args(&["--version"]));

    tester.assert_raw_output_contains("television");
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests the `tv list-channels` command.
///
/// We expect this to list all available channels in the cable directory.
#[test]
fn tv_list_channels() {
    let mut tester = PtyTester::new();
    let mut child =
        tester.spawn_command(tv_local_config_and_cable_with_args(&[
            "list-channels",
        ]));

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

    // Check if the output contains all channel names
    let output = tester.read_raw_output();
    for channel in cable_dir_filenames {
        assert!(
            output.contains(&channel),
            "Channel '{}' not found in output: {}",
            channel,
            output
        );
    }

    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

#[test]
/// This simply tests that the command exits successfully.
fn tv_init_zsh() {
    let mut tester = PtyTester::new();
    let mut child = tester
        .spawn_command(tv_local_config_and_cable_with_args(&["init", "zsh"]));

    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}
