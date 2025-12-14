use criterion::{Criterion, black_box, criterion_group};
use television::utils::strings::{
    ReplaceNonPrintableConfig, replace_non_printable_bulk,
};

/// Benchmark for pure ASCII text (most common case)
pub fn replace_non_printable_ascii(c: &mut Criterion) {
    let input = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                  Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
                  Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.";
    let config = ReplaceNonPrintableConfig::default();

    c.bench_function("replace_non_printable_ascii", |b| {
        b.iter(|| {
            replace_non_printable_bulk(black_box(input), black_box(&config))
        });
    });
}

/// Benchmark for text with tabs (triggers tab expansion)
pub fn replace_non_printable_with_tabs(c: &mut Criterion) {
    let input = "fn main() {\n\tprintln!(\"Hello, world!\");\n\tlet x = 42;\n\treturn x;\n}";
    let config = ReplaceNonPrintableConfig::default();

    c.bench_function("replace_non_printable_with_tabs", |b| {
        b.iter(|| {
            replace_non_printable_bulk(black_box(input), black_box(&config))
        });
    });
}

/// Benchmark for text with control characters
pub fn replace_non_printable_with_control_chars(c: &mut Criterion) {
    let input = "Hello\x00World\x01Test\x7FMore\x1Ftext\u{FEFF}here";
    let config = ReplaceNonPrintableConfig::default();

    c.bench_function("replace_non_printable_with_control_chars", |b| {
        b.iter(|| {
            replace_non_printable_bulk(black_box(input), black_box(&config))
        });
    });
}

/// Benchmark for Unicode text (CJK, emoji, etc.)
pub fn replace_non_printable_unicode(c: &mut Criterion) {
    let input = "Hello 世界 🌍 こんにちは 안녕하세요 สวัสดี नमस्ते!";
    let config = ReplaceNonPrintableConfig::default();

    c.bench_function("replace_non_printable_unicode", |b| {
        b.iter(|| {
            replace_non_printable_bulk(black_box(input), black_box(&config))
        });
    });
}

/// Benchmark for mixed content (realistic scenario)
pub fn replace_non_printable_mixed(c: &mut Criterion) {
    let input = "src/main.rs:42:    fn process_data() {\n\
                 \tlet items = vec![1, 2, 3];\n\
                 \t// Process 世界 items\n\
                 \tfor item in items {\n\
                 \t\tprintln!(\"Item: {}\", item);\n\
                 \t}\n\
                 }";
    let config = ReplaceNonPrintableConfig::default();

    c.bench_function("replace_non_printable_mixed", |b| {
        b.iter(|| {
            replace_non_printable_bulk(black_box(input), black_box(&config))
        });
    });
}

/// Benchmark for large ASCII text (stress test)
pub fn replace_non_printable_large_ascii(c: &mut Criterion) {
    let line = "The quick brown fox jumps over the lazy dog. ";
    let input = line.repeat(100);
    let config = ReplaceNonPrintableConfig::default();

    c.bench_function("replace_non_printable_large_ascii", |b| {
        b.iter(|| {
            replace_non_printable_bulk(black_box(&input), black_box(&config))
        });
    });
}

/// Benchmark for text with Nerd Font icons (tests NF optimization)
pub fn replace_non_printable_nerd_fonts(c: &mut Criterion) {
    // Using actual Nerd Font characters in the ranges we optimized
    let input = " file.rs  folder  test.txt ";
    let config = ReplaceNonPrintableConfig::default();

    c.bench_function("replace_non_printable_nerd_fonts", |b| {
        b.iter(|| {
            replace_non_printable_bulk(black_box(input), black_box(&config))
        });
    });
}

criterion_group!(
    benches,
    // Original implementation
    replace_non_printable_ascii,
    replace_non_printable_with_tabs,
    replace_non_printable_with_control_chars,
    replace_non_printable_unicode,
    replace_non_printable_mixed,
    replace_non_printable_large_ascii,
    replace_non_printable_nerd_fonts,
);
