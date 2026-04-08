mod common;

use common::*;

/// Really just a sanity check
#[test]
fn tv_version() {
    let mut tester = PtyTester::new();
    let mut child = tester.spawn_command(tv_with_args(&["--version"]));

    tester.assert_raw_output_contains("television");
    tester.assert_exit_ok(&mut child, DEFAULT_DELAY);
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

    // Use timeout-based assertion to wait for the full output.
    // Checking for the last channel alphabetically ensures the entire list
    // has been printed.
    tester.assert_raw_output_contains_with_timeout(
        cable_dir_filenames.iter().max().unwrap(),
        std::time::Duration::from_secs(2),
    );

    tester.assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
/// This simply tests that the command exits successfully.
fn tv_init_zsh() {
    let mut tester = PtyTester::new();
    let mut child = tester
        .spawn_command(tv_local_config_and_cable_with_args(&["init", "zsh"]));

    tester.assert_exit_ok(&mut child, DEFAULT_DELAY);
}
