use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use crate::world::constants::{CHUNK_SIZE, HEIGHT};

#[derive(Copy, Clone, PartialEq)]
pub enum Voxel {
    Air,
    Solid,
}

pub struct Chunk {
    pub data: Vec<u8>, // or voxels
}


impl Chunk {
    pub fn new(_x: i32, _z: i32, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed ^ ((_x as u64) << 32 | _z as u64));
        let mut data = vec![0u8; CHUNK_SIZE * HEIGHT * CHUNK_SIZE];

        for i in 0..CHUNK_SIZE {
            for k in 0..CHUNK_SIZE {
                let height = (rng.random::<f32>() * HEIGHT as f32) as usize;
                for y in 0..height {
                    data[(y * CHUNK_SIZE * CHUNK_SIZE) + (k * CHUNK_SIZE) + i] = 1;
                }
            }
        }

        Self { data }
    }
}

