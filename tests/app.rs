//! This module tests the inner `App` struct of the `television` crate.

use std::{collections::HashSet, path::PathBuf, time::Duration};

use television::{
    action::Action,
    app::App,
    cable::Cable,
    channels::prototypes::ChannelPrototype,
    cli::{ChannelCli, PostProcessedCli},
    config::{default_config_from_file, layers::LayeredConfig},
};
use tokio::{
    task::JoinHandle,
    time::{sleep, timeout},
};

/// Default timeout for tests.
///
/// This is kept quite high to avoid flakiness in CI.
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1000);

/// Sets up an app with a file channel and default config.
///
/// Returns a tuple containing the app's `JoinHandle` and the action channel's
/// sender.
///
/// The app is started in a separate task and can be interacted with by sending
/// actions to the action channel.
fn setup_app(
    channel_prototype: Option<ChannelPrototype>,
    select_1: bool,
    exact: bool,
) -> (
    JoinHandle<television::app::AppOutput>,
    tokio::sync::mpsc::UnboundedSender<Action>,
) {
    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("target_dir");
    std::env::set_current_dir(&target_dir).unwrap();

    let chan: ChannelPrototype = channel_prototype
        .unwrap_or(ChannelPrototype::new("files", "find . -type f"));
    let mut config = default_config_from_file().unwrap();
    // this speeds up the tests
    config.application.tick_rate = 100;

    let layered_config = LayeredConfig::new(
        config,
        chan,
        PostProcessedCli {
            channel: ChannelCli {
                select_1,
                exact,
                ..ChannelCli::default()
            },
            ..PostProcessedCli::default()
        },
    );
    let mut app = App::new(
        layered_config,
        Cable::from_prototypes(vec![
            ChannelPrototype::new("files", "find . -type f"),
            ChannelPrototype::new("dirs", "find . -type d"),
            ChannelPrototype::new("env", "printenv"),
        ]),
    );

    // retrieve the app's action channel handle to send a quit action
    let tx = app.action_tx.clone();

    // start the app in a separate task
    let f = tokio::spawn(async move { app.run_headless().await.unwrap() });

    // let the app spin up
    std::thread::sleep(Duration::from_millis(100));

    (f, tx)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_does_quit() {
    let (f, tx) = setup_app(None, false, false);

    // send a quit action to the app
    tx.send(Action::Quit).unwrap();

    // assert that the app quits within a default timeout
    sleep(DEFAULT_TIMEOUT).await;
    assert!(f.is_finished());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_starts_normally() {
    let (f, _) = setup_app(None, false, false);

    // assert that the app is still running after the default timeout
    sleep(DEFAULT_TIMEOUT).await;
    assert!(!f.is_finished());

    f.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_basic_search() {
    let (f, tx) = setup_app(None, false, false);

    // send actions to the app
    for c in "file1".chars() {
        tx.send(Action::AddInputChar(c)).unwrap();
        sleep(Duration::from_millis(100)).await;
    }
    tx.send(Action::ConfirmSelection).unwrap();

    // check the output with a timeout
    let output = timeout(DEFAULT_TIMEOUT, f)
        .await
        .expect("app did not finish within the default timeout")
        .unwrap();

    assert!(output.selected_entries.is_some());
    assert_eq!(
        &output.selected_entries.unwrap().drain().next().unwrap().raw,
        "./file1.txt"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_basic_search_multiselect() {
    let (f, tx) = setup_app(None, false, false);

    // send actions to the app
    for c in "file".chars() {
        tx.send(Action::AddInputChar(c)).unwrap();
    }

    // select both files
    tx.send(Action::ToggleSelectionDown).unwrap();
    sleep(Duration::from_millis(50)).await;
    tx.send(Action::ToggleSelectionDown).unwrap();
    sleep(Duration::from_millis(50)).await;
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
            .map(|e| &e.raw)
            .collect::<HashSet<_>>(),
        HashSet::from([
            &"./file1.txt".to_string(),
            &"./file2.txt".to_string()
        ])
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_exact_search_multiselect() {
    let (f, tx) = setup_app(None, false, true);

    // send actions to the app
    for c in "fie".chars() {
        tx.send(Action::AddInputChar(c)).unwrap();
    }

    tx.send(Action::ConfirmSelection).unwrap();

    // check the output with a timeout
    let output = timeout(DEFAULT_TIMEOUT, f)
        .await
        .expect("app did not finish within the default timeout")
        .unwrap();

    let selected_entries = output.selected_entries.clone();
    assert!(selected_entries.is_some());
    // should contain a single entry with the prompt
    assert!(!selected_entries.as_ref().unwrap().is_empty());
    assert_eq!(selected_entries.unwrap().drain().next().unwrap().raw, "fie");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_exact_search_positive() {
    let (f, tx) = setup_app(None, false, true);

    // send actions to the app
    for c in "file".chars() {
        tx.send(Action::AddInputChar(c)).unwrap();
    }

    // select both files
    tx.send(Action::ToggleSelectionDown).unwrap();
    sleep(Duration::from_millis(50)).await;
    tx.send(Action::ToggleSelectionDown).unwrap();
    sleep(Duration::from_millis(50)).await;
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
            .map(|e| &e.raw)
            .collect::<HashSet<_>>(),
        HashSet::from([
            &"./file1.txt".to_string(),
            &"./file2.txt".to_string()
        ])
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_exits_when_select_1_and_only_one_result() {
    let prototype = ChannelPrototype::new("some_channel", "echo file1.txt");
    let (f, tx) = setup_app(Some(prototype), true, false);

    // tick a few times to get the results
    for _ in 0..=10 {
        tx.send(Action::Tick).unwrap();
    }

    // check the output with a timeout
    // Note: we don't need to send a confirm action here, as the app should
    // exit automatically when there's only one result
    let output = timeout(DEFAULT_TIMEOUT, f)
        .await
        .expect("app did not finish within the default timeout")
        .unwrap();

    assert!(output.selected_entries.is_some());
    assert_eq!(
        &output.selected_entries.unwrap().drain().next().unwrap().raw,
        "file1.txt"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_app_does_not_exit_when_select_1_and_more_than_one_result() {
    let prototype =
        ChannelPrototype::new("some_channel", "echo 'file1.txt\nfile2.txt'");
    let (f, tx) = setup_app(Some(prototype), true, false);

    // tick a few times to get the results
    for _ in 0..=10 {
        tx.send(Action::Tick).unwrap();
    }

    // check that the app is still running after the default timeout
    let output = timeout(DEFAULT_TIMEOUT, f).await;

    assert!(output.is_err());
}
