use criterion::{Criterion, black_box, criterion_group};
use rustc_hash::FxHashMap;
use television::{
    channels::{
        entry::Entry,
        prototypes::{CommandSpec, Template},
    },
    previewer::try_preview,
};
use tokio::sync::mpsc;

fn make_command(cmd: &str) -> CommandSpec {
    let template = Template::parse(cmd).unwrap();
    CommandSpec::new(vec![template], false, FxHashMap::default())
}

fn bench_preview(
    c: &mut Criterion,
    name: &str,
    entry_name: &str,
    command: &str,
) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function(name, |b| {
        b.to_async(&rt).iter(|| async {
            let entry = black_box(Entry::new(entry_name.to_string()));
            let command = black_box(make_command(command));
            let (tx, mut rx) = mpsc::unbounded_channel();

            try_preview(command, 0, None, None, None, entry, tx, None)
                .await
                .unwrap();

            let _ = rx.recv().await;
        });
    });
}

pub fn preview_ascii(c: &mut Criterion) {
    bench_preview(
        c,
        "preview_ascii",
        "test_file.txt",
        "echo \"Hello, World! This is a simple ASCII preview.\"",
    );
}

pub fn preview_ansi_colors(c: &mut Criterion) {
    bench_preview(
        c,
        "preview_ansi_colors",
        "colored_file.txt",
        r#"echo -e "\x1b[31mRed text\x1b[0m \x1b[32mGreen text\x1b[0m \x1b[34mBlue text\x1b[0m""#,
    );
}

pub fn preview_unicode(c: &mut Criterion) {
    bench_preview(
        c,
        "preview_unicode",
        "unicode.txt",
        "echo \"Hello 世界 🌍 こんにちは 안녕하세요 नमस्ते!\"",
    );
}

pub fn preview_with_tabs(c: &mut Criterion) {
    bench_preview(
        c,
        "preview_with_tabs",
        "code.rs",
        "echo -e \"fn main() {\\n\\tprintln!(\\\"Hello\\\");\\n}\"",
    );
}

pub fn preview_multiline(c: &mut Criterion) {
    let text = (1..=500)
        .map(|i| {
            format!("Line {}: The quick brown fox jumps over the lazy dog", i)
        })
        .collect::<Vec<_>>()
        .join("\\n");

    bench_preview(
        c,
        "preview_multiline",
        "large_file.txt",
        &format!("echo -e \"{}\"", text),
    );
}

pub fn preview_large_ansi(c: &mut Criterion) {
    let text = (1..=1000)
        .map(|i| {
            format!(
                "\\x1b[31mLine {}:\\x1b[0m The quick brown fox jumps over the lazy dog",
                i
            )
        })
        .collect::<Vec<_>>()
        .join("\\n");

    bench_preview(
        c,
        "preview_large_ansi",
        "large_ansi_file.txt",
        &format!("echo -e \"{}\"", text),
    );
}

criterion_group!(
    benches,
    preview_ascii,
    preview_ansi_colors,
    preview_unicode,
    preview_with_tabs,
    preview_multiline,
    preview_large_ansi,
);
