pub mod main {
    pub mod draw;
    pub mod results_list_benchmark;
}
pub use main::*;

criterion::criterion_main!(results_list_benchmark::benches, draw::benches,);
