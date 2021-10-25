use criterion::{black_box, criterion_group, criterion_main, Criterion};

use nalgebra::vector;
use raycaster_lib::render::Renderer;
use raycaster_lib::volumetric::{vol_reader, LinearVolume};
use raycaster_lib::{render_frame, Camera, RenderOptions};

fn full_render(c: &mut Criterion) {
    c.bench_function("file read, alloc, render 512x512", |b| {
        b.iter(|| render_frame(black_box(512), black_box(512)));
    });
}

fn pure_render(c: &mut Criterion) {
    c.bench_function("render 512x512", |b| {
        let camera = Camera::new(512, 512);
        let read_result = vol_reader::from_file("Skull.vol");

        let volume_b = match read_result {
            Ok(vol) => vol,
            Err(message) => {
                eprint!("{}", message);
                std::process::exit(1);
            }
        };

        let volume = volume_b.build();

        let mut renderer = Renderer::<LinearVolume>::new(volume, camera);
        renderer.set_render_options(RenderOptions {
            ray_termination: true,
            empty_index: true,
            multi_thread: false,
        });

        let mut buffer: Vec<u8> = vec![0; 512 * 512 * 3];

        b.iter(|| renderer.render_to_buffer());
    });
}

fn float_to_int_vector(c: &mut Criterion) {
    let index_block_size = 4;
    let pos = vector![5.1f32, 74.2, 111.7];
    c.bench_function("vec float to int", |b| {
        b.iter(|| black_box(pos).map(|f| (black_box(f) as usize)) / black_box(index_block_size));
    });
}

criterion_group!(benches, pure_render);
//criterion_group!(benches, float_to_int_vector);
criterion_main!(benches);
