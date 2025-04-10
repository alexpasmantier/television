use criterion::{black_box, Criterion};
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::path::PathBuf;
use television::action::Action;
use television::channels::OnAir;
use television::channels::{files::Channel, TelevisionChannel};
use television::config::{Config, ConfigEnv};
use television::television::Television;
use tokio::runtime::Runtime;

fn draw(c: &mut Criterion) {
    let width = 250;
    let height = 80;

    let rt = Runtime::new().unwrap();

    c.bench_function("draw", |b| {
        b.to_async(&rt).iter_batched(
            // FIXME: this is kind of hacky
            || {
                let config = Config::new(&ConfigEnv::init().unwrap()).unwrap();
                let backend = TestBackend::new(width, height);
                let terminal = Terminal::new(backend).unwrap();
                let (tx, _) = tokio::sync::mpsc::unbounded_channel();
                let mut channel =
                    TelevisionChannel::Files(Channel::new(vec![
                        PathBuf::from("."),
                    ]));
                channel.find("television");
                // Wait for the channel to finish loading
                let mut tv =
                    Television::new(tx, channel, config, None, false, false);
                for _ in 0..5 {
                    // tick the matcher
                    let _ = tv.channel.results(10, 0);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                tv.select_next_entry(10);
                let _ = tv.update_preview_state(
                    &tv.get_selected_entry(None).unwrap(),
                );
                let _ = tv.update(&Action::Tick);
                (tv, terminal)
            },
            // Measurement
            |(tv, mut terminal)| async move {
                television::draw::draw(
                    black_box(&tv.dump_context()),
                    black_box(&mut terminal.get_frame()),
                    black_box(Rect::new(0, 0, width, height)),
                )
                .unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion::criterion_group!(benches, draw);
criterion::criterion_main!(benches);
