use nalgebra::{vector, Vector3};

use crate::config::Config;

// Describe header
#[derive(Debug)]
pub enum HeaderFormat {
    Default,
}

// Order of samples in resulting array
// Generally, the fastest growing axis is Z axis
#[derive(Debug)]
pub enum SampleOrder {
    // Samples ordered by axis (x,y,z)
    Linear,

    // Samples ordered by blocks
    // Blocks are ordered lineary
    // and data inside blocks is also ordered lineary
    Z(u8), // todo parametrize overlap
}

pub struct DimIterator<SRC>
where
    SRC: Iterator<Item = Vector3<u32>>,
{
    it: SRC,
}

impl<SRC> Iterator for DimIterator<SRC>
where
    SRC: Iterator<Item = Vector3<u32>>,
{
    type Item = Vector3<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        self.it.next()
    }
}

pub struct LinearCoordIterator {
    dims: Vector3<u32>,
    state: Vector3<u32>,
    done: bool,
    started: bool,
}

impl LinearCoordIterator {
    pub fn from_dims(dims: Vector3<u32>) -> LinearCoordIterator {
        LinearCoordIterator {
            dims,
            state: vector![0, 0, 0],
            done: false,
            started: false,
        }
    }
}

impl Iterator for LinearCoordIterator {
    type Item = Vector3<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if !self.started {
            self.started = true;
            return Some(self.state);
        }
        self.state.z += 1;
        if self.state.z == self.dims.z {
            self.state.z = 0;
            self.state.y += 1;
        }
        if self.state.y == self.dims.y {
            self.state.y = 0;
            self.state.x += 1;
        }
        if self.state.x == self.dims.x {
            self.state.x = 0;
            self.done = true;
            return None;
        }

        Some(self.state)
    }
}

// Could be also implemented with two nested LinearCoordIterators
pub struct ZCoordIterator {
    // state
    current_block: Vector3<usize>,
    current_offset: usize,
    // dims
    block_side: usize,
    dims_block: Vector3<usize>,
}

impl Iterator for ZCoordIterator {
    type Item = Vector3<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn lin_order() {
        let order = LinearCoordIterator::from_dims(vector![2, 3, 4]);

        #[rustfmt::skip]
        let expected = &[
            vector![0,0,0],vector![0,0,1],vector![0,0,2],vector![0,0,3],
            vector![0,1,0],vector![0,1,1],vector![0,1,2],vector![0,1,3],
            vector![0,2,0],vector![0,2,1],vector![0,2,2],vector![0,2,3],
            // next slice
            vector![1,0,0],vector![1,0,1],vector![1,0,2],vector![1,0,3],
            vector![1,1,0],vector![1,1,1],vector![1,1,2],vector![1,1,3],
            vector![1,2,0],vector![1,2,1],vector![1,2,2],vector![1,2,3],
        ];

        let order_collect: Vec<_> = order.collect();

        assert_eq!(order_collect.len(), 2 * 3 * 4);
        assert_eq!(order_collect.len(), expected.len());

        order_collect
            .iter()
            .zip(expected.iter())
            .for_each(|(act, exp)| assert_eq!(act, exp));
    }
}
