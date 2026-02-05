use criterion::Criterion;
use criterion::criterion_group;
use std::hint::black_box;
use std::sync::Arc;
use television::channels::prototypes::ChannelPrototype;
use television::config::layers::ConfigLayers;
use television::frecency::Frecency;
use television::{
    action::Action,
    cable::Cable,
    cli::PostProcessedCli,
    config::{Config, ConfigEnv},
    television::Television,
};

/// Helper to create a Television instance in a steady state with matched results.
fn setup_tv() -> Television {
    let cable = Cable::from_prototypes(vec![ChannelPrototype::new(
        "files", "fd -t f",
    )]);

    let config = Config::new(&ConfigEnv::init().unwrap(), None).unwrap();
    let (tx, _) = tokio::sync::mpsc::unbounded_channel();
    let channel_prototype = cable.get_channel("files");
    let layered_config = ConfigLayers::new(
        config.clone(),
        channel_prototype.clone(),
        PostProcessedCli::default(),
    );
    let frecency = Arc::new(Frecency::new(100, &config.application.data_dir));
    let mut tv = Television::new(tx, layered_config, cable, frecency);

    // Search for a pattern and let the matcher reach steady state
    tv.find("television");
    for _ in 0..10 {
        let _ = tv.channel.results(50, 0);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    tv.update_results_picker_state();

    tv
}

/// Benchmark `update()` with a Tick action.
///
/// Tick actions affect results (the matcher may still be processing items),
/// so this exercises the full pipeline: matcher tick + snapshot + UTF32
/// conversion + entry construction + Arc allocation.
pub fn update_tick(c: &mut Criterion) {
    let mut tv = setup_tv();

    c.bench_function("update_tick", |b| {
        b.iter(|| {
            let _ = black_box(tv.update(black_box(&Action::Tick)));
        });
    });
}

/// Benchmark `update()` with an action that doesn't affect results.
///
/// Actions like `ScrollPreviewDown` only change UI state and skip the
/// expensive results pipeline entirely (only a cheap `channel.tick()` runs).
pub fn update_no_results(c: &mut Criterion) {
    let mut tv = setup_tv();

    c.bench_function("update_no_results", |b| {
        b.iter(|| {
            let _ =
                black_box(tv.update(black_box(&Action::ScrollPreviewDown)));
        });
    });
}

/// Benchmark `update()` with a navigation action.
///
/// `SelectNextEntry` changes the viewport offset, triggering the full results
/// pipeline to fetch entries for the new visible window.
pub fn update_select_next(c: &mut Criterion) {
    let mut tv = setup_tv();

    c.bench_function("update_select_next", |b| {
        b.iter(|| {
            let _ = black_box(tv.update(black_box(&Action::SelectNextEntry)));
        });
    });
}

/// Benchmark `update()` with input that changes the search pattern.
///
/// `AddInputChar` triggers pattern reparse + full results pipeline, the
/// most expensive update path.
pub fn update_add_char(c: &mut Criterion) {
    let mut tv = setup_tv();

    c.bench_function("update_add_char", |b| {
        b.iter(|| {
            // Alternate between adding and removing a char to keep pattern
            // from growing unbounded
            let _ =
                black_box(tv.update(black_box(&Action::AddInputChar('x'))));
            let _ = black_box(tv.update(black_box(&Action::DeletePrevChar)));
        });
    });
}

criterion_group!(
    benches,
    update_tick,
    update_no_results,
    update_select_next,
    update_add_char,
);
