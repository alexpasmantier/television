use std::{collections::HashSet, path::PathBuf, time::Duration};

use television::{
    action::Action, app::App, channels::TelevisionChannel,
    config::default_config_from_file,
};
use tokio::{task::JoinHandle, time::timeout};

/// Default timeout for tests.
///
/// This is kept quite high to avoid flakiness in CI.
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(500);

/// Sets up an app with a file channel and default config.
///
/// Returns a tuple containing the app's `JoinHandle` and the action channel's
/// sender.
///
/// The app is started in a separate task and can be interacted with by sending
/// actions to the action channel.
fn setup_app() -> (
    JoinHandle<television::app::AppOutput>,
    tokio::sync::mpsc::UnboundedSender<Action>,
) {
    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("target_dir");
    std::env::set_current_dir(&target_dir).unwrap();
    let channel = TelevisionChannel::Files(
        television::channels::files::Channel::new(vec![target_dir]),
    );
    let config = default_config_from_file().unwrap();
    let input = None;

    let mut app = App::new(channel, config, input);

    // retrieve the app's action channel handle in order to send a quit action
    let tx = app.action_tx.clone();

    // start the app in a separate task
    let f = tokio::spawn(async move { app.run_headless().await.unwrap() });

    // let the app spin up
    std::thread::sleep(Duration::from_millis(200));

    (f, tx)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_does_quit() {
    let (f, tx) = setup_app();

    // send a quit action to the app
    tx.send(Action::Quit).unwrap();

    // assert that the app quits within a default timeout
    std::thread::sleep(DEFAULT_TIMEOUT / 4);
    assert!(f.is_finished());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_starts_normally() {
    let (f, _) = setup_app();

    // assert that the app is still running after the default timeout
    std::thread::sleep(DEFAULT_TIMEOUT / 4);
    assert!(!f.is_finished());

    f.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_basic_search() {
    let (f, tx) = setup_app();

    // send actions to the app
    for c in "file1".chars() {
        tx.send(Action::AddInputChar(c)).unwrap();
    }
    tx.send(Action::ConfirmSelection).unwrap();

    // check the output with a timeout
    let output = timeout(DEFAULT_TIMEOUT, f)
        .await
        .expect("app did not finish within the default timeout")
        .unwrap();

    assert!(output.selected_entries.is_some());
    assert_eq!(
        &output
            .selected_entries
            .unwrap()
            .drain()
            .next()
            .unwrap()
            .name,
        "file1.txt"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_basic_search_multiselect() {
    let (f, tx) = setup_app();

    // send actions to the app
    for c in "file".chars() {
        tx.send(Action::AddInputChar(c)).unwrap();
    }

    // select both files
    tx.send(Action::ToggleSelectionDown).unwrap();
    std::thread::sleep(Duration::from_millis(50));
    tx.send(Action::ToggleSelectionDown).unwrap();
    std::thread::sleep(Duration::from_millis(50));
    tx.send(Action::ConfirmSelection).unwrap();

    // check the output with a timeout
    let output = timeout(DEFAULT_TIMEOUT, f)
        .await
        .expect("app did not finish within the default timeout")
        .unwrap();

    assert!(output.selected_entries.is_some());
    assert_eq!(
        output
            .selected_entries
            .as_ref()
            .unwrap()
            .iter()
            .map(|e| &e.name)
            .collect::<HashSet<_>>(),
        HashSet::from([&"file1.txt".to_string(), &"file2.txt".to_string()])
    );
}
