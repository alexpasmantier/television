use criterion::criterion_main;

pub mod main {
    pub mod ui;
}

pub use main::*;

criterion_main!(ui::benches,);
