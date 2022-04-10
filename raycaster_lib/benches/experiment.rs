// Microbenchmarks for development

mod common;
use common::*;
use raycaster_lib::{render::Renderer, PerspectiveCamera};

fn get_ui_from_usize(c: &mut Criterion) {
    return;

    let volume = get_volume();
    let camera = PerspectiveCamera::new(POSITION, DIRECTION);

    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let mut renderer = Renderer::<LinearVolume>::new(volume, render_options);

    c.bench_function("get blocktype from usize position", |b| {
        b.iter(|| {
            // unused test
        });
    });
}

fn get_ui_from_float(c: &mut Criterion) {
    return;

    let volume = get_volume();
    let camera = PerspectiveCamera::new(POSITION, DIRECTION);

    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let mut renderer = Renderer::<LinearVolume>::new(volume, render_options);

    c.bench_function("get blocktype from float position", |b| {
        b.iter(|| {
            // unused test
        });
    });
}

criterion_group!(get_ei_fl_vs_usize, get_ui_from_float, get_ui_from_usize);
criterion_main!(get_ei_fl_vs_usize);
