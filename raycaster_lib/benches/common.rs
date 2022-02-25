pub use criterion::{criterion_group, criterion_main, Criterion};

pub use nalgebra::{point, vector, Point3, Vector3};
pub use raycaster_lib::{
    camera::PerspectiveCamera,
    render::{RenderOptions, Renderer},
    transfer_functions::skull_tf,
    volumetric::{
        from_file, parse::skull_parser, BlockVolume, BuildVolume, LinearVolume, Volume,
        VolumeMetadata,
    },
};

pub const WIDTH: usize = 512;
pub const HEIGHT: usize = 512;

pub const POSITION: Point3<f32> = point![300.0, 300.0, 300.0];
pub const DIRECTION: Vector3<f32> = vector![-1.0, -1.0, -1.0];

pub fn get_volume<V>() -> V
where
    V: Volume + BuildVolume<VolumeMetadata>,
{
    from_file("../volumes/Skull.vol", skull_parser, skull_tf).unwrap()
}
