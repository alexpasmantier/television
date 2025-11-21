use criterion::criterion_group;
use criterion::{Criterion, black_box};
use television::channels::prototypes::ChannelPrototype;
use television::config::layers::LayeredConfig;
use television::{
    cable::Cable,
    cli::PostProcessedCli,
    config::{Config, ConfigEnv},
    television::Television,
};
use tokio::runtime::Runtime;

/// Benchmark the full render path
/// This combines context cloning + drawing to measure end-to-end performance
pub fn render(c: &mut Criterion) {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;

    let width = 250;
    let height = 80;
    let rt = Runtime::new().unwrap();

    let cable = Cable::from_prototypes(vec![ChannelPrototype::new(
        "files", "fd -t f",
    )]);

    c.bench_function("full_render_cycle", |b| {
        b.to_async(&rt).iter_batched(
            || {
                let config =
                    Config::new(&ConfigEnv::init().unwrap(), None).unwrap();
                let backend = TestBackend::new(width, height);
                let terminal = Terminal::new(backend).unwrap();
                let (tx, _) = tokio::sync::mpsc::unbounded_channel();
                let channel_prototype = cable.get_channel("files");
                let layered_config = LayeredConfig::new(
                    config,
                    channel_prototype.clone(),
                    PostProcessedCli::default(),
                );
                let mut tv =
                    Television::new(tx, layered_config, cable.clone());
                tv.find("rust");
                for _ in 0..5 {
                    let _ = tv.channel.results(10, 0);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                tv.update_results_picker_state();
                (tv, terminal)
            },
            |(tv, mut terminal)| async move {
                // This simulates a full frame render at 60fps
                let ctx = black_box(Box::new(tv.dump_context()));
                television::draw::draw(
                    black_box(&ctx),
                    black_box(&mut terminal.get_frame()),
                    black_box(Rect::new(0, 0, width, height)),
                )
                .unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, render);
