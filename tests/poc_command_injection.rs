use rustc_hash::FxHashSet;
use television::channels::entry::Entry;
use television::channels::prototypes::Template;
use television::utils::command::{format_command, shell_command};

const MARKER: &str = "INJECTED_BY_POC_MARKER";

#[test]
fn crafted_filename_cannot_break_out_of_single_quotes_in_action_command() {
    let payload = format!("x'; printf {}; #'", MARKER);
    let mut entries = FxHashSet::default();
    entries.insert(Entry::new(payload.clone()));

    // Mirrors the bundled "edit" action template from cable/unix/files.toml:
    // command = "${EDITOR:-vim} {}"
    let template = Template::parse("echo {}").unwrap();
    let formatted = format_command(&entries, &template, "\n").unwrap();
    eprintln!("formatted command: {}", formatted);

    let envs: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut cmd = shell_command(&formatted, false, &envs, None);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    eprintln!("captured stdout: {}", stdout);
    // Before the fix, `printf` ran as a separate command and the marker
    // appeared on its own line with no surrounding payload text. After the
    // fix, the whole payload is echoed back literally as one argument, so
    // the marker only ever appears embedded inside the untouched payload.
    assert_eq!(
        stdout.trim_end(),
        payload,
        "echo should print the filename back literally, unexecuted, got: {}",
        stdout
    );
}

#[test]
fn normal_filename_is_unaffected() {
    let mut entries = FxHashSet::default();
    entries.insert(Entry::new("normal_file.txt".to_string()));
    let template = Template::parse("echo {}").unwrap();
    let formatted = format_command(&entries, &template, "\n").unwrap();
    assert_eq!(formatted, "echo 'normal_file.txt'");
}
