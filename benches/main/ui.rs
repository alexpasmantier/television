use criterion::Criterion;
use criterion::criterion_group;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use std::hint::black_box;
use std::sync::Arc;
use television::channels::prototypes::ChannelPrototype;
use television::config::layers::ConfigLayers;
use television::frecency::Frecency;
use television::picker::Movement;
use television::{
    action::Action,
    cable::Cable,
    cli::PostProcessedCli,
    config::{Config, ConfigEnv},
    television::Television,
};
use tokio::runtime::Runtime;

#[allow(clippy::missing_panics_doc)]
pub fn draw(c: &mut Criterion) {
    let width = 250;
    let height = 80;

    let rt = Runtime::new().unwrap();

    let cable = Cable::from_prototypes(vec![ChannelPrototype::new(
        "files", "fd -t f",
    )]);

    c.bench_function("draw", |b| {
        b.to_async(&rt).iter_batched(
            // FIXME: this is kind of hacky
            || {
                let config =
                    Config::new(&ConfigEnv::init().unwrap(), None).unwrap();
                let backend = TestBackend::new(width, height);
                let terminal = Terminal::new(backend).unwrap();
                let (tx, _) = tokio::sync::mpsc::unbounded_channel();
                let channel_prototype = cable.get_channel("files");
                let layered_config = ConfigLayers::new(
                    config.clone(),
                    channel_prototype.clone(),
                    PostProcessedCli::default(),
                );
                let frecency =
                    Arc::new(Frecency::new(100, &config.application.data_dir));
                // Wait for the channel to finish loading
                let mut tv = Television::new(
                    tx,
                    layered_config,
                    cable.clone(),
                    frecency,
                );
                tv.find("television");
                for _ in 0..5 {
                    // tick the matcher
                    let _ = tv.channel.results(10, 0);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                tv.move_cursor(Movement::Next, 10);
                let selected_entry = tv.get_selected_entry();
                let _ = tv.update_preview_state(&selected_entry);
                let _ = tv.update(&Action::Tick);
                (tv, terminal)
            },
            // Measurement
            |(tv, mut terminal)| async move {
                television::draw::draw(
                    black_box(tv.dump_context()),
                    black_box(&mut terminal.get_frame()),
                    black_box(Rect::new(0, 0, width, height)),
                )
                .unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, draw);
