pub mod main {
    pub mod draw;
    pub mod draw_results_list;
}
pub use main::*;

criterion::criterion_main!(draw_results_list::benches, draw::benches,);
