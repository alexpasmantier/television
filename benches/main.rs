use criterion::criterion_main;

pub mod main {
    pub mod load_candidates;
    pub mod previewer;
    pub mod render;
    pub mod strings;
    pub mod ui;
}

pub use main::*;

criterion_main!(
    ui::benches,
    load_candidates::benches,
    previewer::benches,
    render::benches,
    strings::benches,
);
