use criterion::criterion_group;
use criterion::{BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use television::channels::entry_processor::{
    AnsiProcessor, DisplayProcessor, PlainProcessor,
};
use television::channels::prototypes::SourceSpec;
use television::matcher::{
    Matcher,
    config::{Config, SortStrategy},
};
use tokio::runtime::Runtime;

pub fn load_candidates_by_size(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("load_candidates");

    let sizes = vec![10_000, 100_000, 1_000_000];

    for size in sizes {
        group.throughput(Throughput::Elements(size));

        group.bench_with_input(
            BenchmarkId::new("default_delimiter", size),
            &size,
            |b, &size| {
                b.to_async(&rt).iter(|| async move {
                    // Generate a command that produces `size` lines
                    let command_str =
                        format!("seq 1 {} | sed 's/.*/entry_&/'", size);

                    let source_spec: SourceSpec = toml::from_str(&format!(
                        r#"
                        command = "{}"
                        "#,
                        command_str
                    ))
                    .unwrap();

                    // Plain mode uses Matcher<()> for memory efficiency
                    let mut matcher = Matcher::<()>::new(
                        &Config::default(),
                        SortStrategy::Score,
                    );
                    let injector = matcher.injector();

                    television::channels::channel::load_candidates(
                        black_box(source_spec.command),
                        black_box(source_spec.entry_delimiter),
                        black_box(0),
                        black_box(PlainProcessor),
                        injector,
                    )
                    .await;

                    // Ensure matcher has processed entries
                    matcher.tick();
                });
            },
        );
    }

    group.finish();
}

pub fn load_candidates_with_ansi(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let size = 100_000;

    let mut group = c.benchmark_group("load_candidates_ansi");
    group.throughput(Throughput::Elements(size));

    group.bench_function("no_ansi", |b| {
        b.to_async(&rt).iter(|| async {
            let source_spec: SourceSpec = toml::from_str(&format!(
                r#"
                command = "seq 1 {} | sed 's/.*/entry_&/'"
                ansi = false
                "#,
                size
            ))
            .unwrap();

            // Plain mode uses Matcher<()>
            let mut matcher =
                Matcher::<()>::new(&Config::default(), SortStrategy::Score);
            let injector = matcher.injector();

            television::channels::channel::load_candidates(
                black_box(source_spec.command),
                black_box(source_spec.entry_delimiter),
                black_box(0),
                black_box(PlainProcessor),
                injector,
            )
            .await;

            matcher.tick();
        });
    });

    group.bench_function("with_ansi", |b| {
        b.to_async(&rt).iter(|| async {
            // Use colored output to generate ANSI codes
            let source_spec: SourceSpec = toml::from_str(&format!(
                r#"
                command = "seq 1 {} | sed 's/.*/\\x1b[31mentry_&\\x1b[0m/'"
                ansi = true
                "#,
                size
            ))
            .unwrap();

            // ANSI mode uses Matcher<String> to store original
            let mut matcher = Matcher::<String>::new(
                &Config::default(),
                SortStrategy::Score,
            );
            let injector = matcher.injector();

            television::channels::channel::load_candidates(
                black_box(source_spec.command),
                black_box(source_spec.entry_delimiter),
                black_box(0),
                black_box(AnsiProcessor),
                injector,
            )
            .await;

            matcher.tick();
        });
    });

    group.finish();
}

pub fn load_candidates_with_display_template(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let size = 100_000;

    let mut group = c.benchmark_group("load_candidates_display");
    group.throughput(Throughput::Elements(size));

    group.bench_function("no_template", |b| {
        b.to_async(&rt).iter(|| async {
            let source_spec: SourceSpec = toml::from_str(&format!(
                r#"
                command = "seq 1 {} | sed 's/.*/entry_&/'"
                "#,
                size
            ))
            .unwrap();

            // Plain mode uses Matcher<()>
            let mut matcher =
                Matcher::<()>::new(&Config::default(), SortStrategy::Score);
            let injector = matcher.injector();

            television::channels::channel::load_candidates(
                black_box(source_spec.command),
                black_box(source_spec.entry_delimiter),
                black_box(0),
                black_box(PlainProcessor),
                injector,
            )
            .await;

            matcher.tick();
        });
    });

    group.bench_function("with_template", |b| {
        b.to_async(&rt).iter(|| async {
            let source_spec: SourceSpec = toml::from_str(&format!(
                r#"
                command = "seq 1 {} | sed 's/.*/entry_&/'"
                display = "{{}} - displayed"
                "#,
                size
            ))
            .unwrap();

            // Display mode uses Matcher<String> to store original
            let mut matcher = Matcher::<String>::new(
                &Config::default(),
                SortStrategy::Score,
            );
            let injector = matcher.injector();

            television::channels::channel::load_candidates(
                black_box(source_spec.command),
                black_box(source_spec.entry_delimiter),
                black_box(0),
                black_box(DisplayProcessor {
                    template: source_spec.display.unwrap(),
                }),
                injector,
            )
            .await;

            matcher.tick();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    load_candidates_by_size,
    load_candidates_with_ansi,
    load_candidates_with_display_template,
);
