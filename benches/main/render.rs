use criterion::Criterion;
use criterion::criterion_group;
use std::hint::black_box;
use std::sync::Arc;
use television::channels::prototypes::ChannelPrototype;
use television::config::layers::ConfigLayers;
use television::frecency::Frecency;
use television::{
    cable::Cable,
    cli::PostProcessedCli,
    config::{Config, ConfigEnv},
    television::Television,
};

/// Benchmark a render cycle (context dump + drawing)
pub fn render(c: &mut Criterion) {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;

    let width = 250;
    let height = 80;

    let cable = Cable::from_prototypes(vec![ChannelPrototype::new(
        "files", "fd -t f",
    )]);

    let config = Config::new(&ConfigEnv::init().unwrap(), None).unwrap();
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let (tx, _) = tokio::sync::mpsc::unbounded_channel();
    let channel_prototype = cable.get_channel("files");
    let layered_config = ConfigLayers::new(
        config.clone(),
        channel_prototype.clone(),
        PostProcessedCli::default(),
    );
    let frecency = Arc::new(Frecency::new(100, &config.application.data_dir));
    let mut tv = Television::new(tx, layered_config, cable.clone(), frecency);
    tv.find("visio");
    // just make sure we're in a steady state
    for _ in 0..5 {
        let _ = tv.channel.results(50, 0);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    tv.update_results_picker_state();

    c.bench_function("render_cycle", |b| {
        b.iter(|| {
            let ctx = black_box(Box::new(tv.dump_context()));
            television::draw::draw(
                black_box(*ctx),
                black_box(&mut terminal.get_frame()),
                black_box(Rect::new(0, 0, width, height)),
            )
            .unwrap();
        });
    });
}

criterion_group!(benches, render);
