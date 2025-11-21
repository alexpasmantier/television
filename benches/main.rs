use criterion::criterion_main;

pub mod main {
    pub mod load_candidates;
    pub mod render;
    pub mod ui;
}

pub use main::*;

criterion_main!(ui::benches, load_candidates::benches, render::benches,);
