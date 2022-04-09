use criterion::{criterion_group, criterion_main, Criterion};
use render_benchmarks::{
    linear::{render_linear, render_linear_ei, render_linear_ert, render_linear_ert_ei},
    multi_thread::{render_parallel_mem, render_parallel_stream},
};

mod common;
mod render_benchmarks;

criterion_group! {
    name = sequential_linear;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = render_linear, render_linear_ert, render_linear_ei, render_linear_ert_ei
}

criterion_group! {
    name = parallel;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = render_parallel_mem, render_parallel_stream
}

criterion_main!(sequential_linear, parallel);
//criterion_main!(parallel);
