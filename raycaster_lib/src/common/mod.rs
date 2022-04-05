mod bound_box;
mod ray;
mod value_range;
mod viewport_box;

pub use bound_box::{BoundBox, BoundBoxIterator};
use nalgebra::{vector, Vector3};
pub use ray::Ray;
pub use value_range::ValueRange;
pub use viewport_box::{PixelBox, ViewportBox};

// Rounds up
pub fn blockify(size: Vector3<usize>, side: usize, overlap: usize) -> Vector3<usize> {
    let cells = side - overlap; // cells per block
                                // (size-1) -- number of cells
                                // (+cells-1 / cells) -- division with rounding up
    let x = size + vector![cells, cells, cells];
    let y = x - vector![2, 2, 2];
    y / cells
}

#[cfg(test)]
mod test {

    use nalgebra::vector;

    use super::*;

    #[test]
    fn blockify_3() {
        let side = 3;
        let overlap = 1;

        let size = vector![5, 5, 5];
        assert_eq!(blockify(size, side, overlap), vector![2, 2, 2]);

        let size = vector![6, 6, 6];
        assert_eq!(blockify(size, side, overlap), vector![3, 3, 3]);

        let size = vector![6, 6, 7];
        assert_eq!(blockify(size, side, overlap), vector![3, 3, 3]);

        let size = vector![6, 6, 8];
        assert_eq!(blockify(size, side, overlap), vector![3, 3, 4]);
    }

    #[test]
    fn blockify_10() {
        let side = 10;
        let overlap = 1;

        let size = vector![5, 5, 5];
        assert_eq!(blockify(size, side, overlap), vector![1, 1, 1]);

        let size = vector![11, 10, 12];
        assert_eq!(blockify(size, side, overlap), vector![2, 1, 2]);

        let size = vector![19, 20, 21];
        assert_eq!(blockify(size, side, overlap), vector![2, 3, 3]);

        let size = vector![105, 57, 67];
        assert_eq!(blockify(size, side, overlap), vector![12, 7, 8]);
    }

    #[test]
    fn blockify_11() {
        let side = 11;
        let overlap = 1;

        let size = vector![5, 5, 5];
        assert_eq!(blockify(size, side, overlap), vector![1, 1, 1]);

        let size = vector![10, 11, 12];
        assert_eq!(blockify(size, side, overlap), vector![1, 1, 2]);

        let size = vector![19, 20, 21];
        assert_eq!(blockify(size, side, overlap), vector![2, 2, 2]);

        let size = vector![105, 57, 67];
        assert_eq!(blockify(size, side, overlap), vector![11, 6, 7]);
    }
}
