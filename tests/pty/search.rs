//! Search input and matching: `--input`, `--exact`, typo resistance, `--no-sort`.

use tempfile::TempDir;

use crate::common::*;

#[test]
fn test_input_prefills_search_box() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--input", "UNIQUE16CHARID"],
    )
    .start()
    .unwrap();

    s.wait().text("UNIQUE16CHARID").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_exact_matching_enabled() {
    let pt = phantom();
    let tmp_dir = TempDir::new().unwrap();

    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--exact",
            "--input",
            "UNIQUE16CHARIDfile",
            tmp_dir.path().to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait().text("UNIQUE16CHARIDfile.txt").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_exact_matching_enabled_fails() {
    let pt = phantom();
    let tmp_dir = TempDir::new().unwrap();

    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--exact",
            "--input",
            "UNIQUE16CHARIDfl",
            tmp_dir.path().to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("UNIQUE16CHARIDfl")
        .text(" 0/1")
        .until()
        .unwrap();
    assert_frame_not_contains(&s, "UNIQUE16CHARIDfile.txt");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_typo_resistance_matches_misspelled_input() {
    let pt = phantom();
    let tmp_dir = TempDir::new().unwrap();

    std::fs::write(tmp_dir.path().join("UNIQUETYPOFILE.txt"), "").unwrap();

    // "UNIQUETYPXFILE" needs one typo to match "UNIQUETYPOFILE.txt"
    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--typo-resistance",
            "--input",
            "UNIQUETYPXFILE",
            tmp_dir.path().to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait().text("UNIQUETYPOFILE.txt").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_no_sort_preserves_source_order() {
    let pt = phantom();

    // The second entry is a stronger fuzzy match for "ab".
    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--source-command",
            "echo 'a-weak-b'; echo 'ab-strong'",
            "--input",
            "ab",
            "--no-sort",
            "--take-1",
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("a-weak-b")
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();
}
