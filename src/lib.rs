mod camera;
mod ray;
pub mod render;
pub mod volumetric;

pub use camera::Camera;
pub use render::{RenderOptions, Renderer};
pub use volumetric::vol_reader;
pub use volumetric::EmptyIndexes;

use crate::volumetric::LinearVolume;

pub fn render_frame(width: usize, height: usize) -> Vec<u8> {
    let camera = Camera::new(width, height);
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
        empty_index: false,
        multi_thread: false,
    });

    renderer.render_to_buffer();

    renderer.get_buffer()
}
