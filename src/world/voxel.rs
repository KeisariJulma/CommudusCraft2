use noise::{NoiseFn, Perlin};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use crate::world::constants::{CHUNK_SIZE, HEIGHT_ABOVE, HEIGHT_BELOW, TOTAL_HEIGHT};
use bevy::prelude::*;

#[derive(Copy, Clone, PartialEq)]
pub enum Voxel {
    Air,
    Solid,
}

#[derive(Clone)]
pub struct Chunk {
    pub data: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; TOTAL_HEIGHT],
}

#[derive(Component)]
pub struct ChunkEntity {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

#[derive(Component)]
struct ChunkPendingDespawn {
    timer: Timer,
}



impl Chunk {
    pub fn new(chunk_x: i32, chunk_z: i32, seed: u64) -> Self {
        let mut data = [[[Voxel::Air as u8; CHUNK_SIZE]; CHUNK_SIZE]; TOTAL_HEIGHT];

        // Perlin noise for terrain height
        let perlin = Perlin::new(seed as u32);
        let freq = 0.01;

        // Cave noise
        let cave_noise = Perlin::new((seed.wrapping_add(1)) as u32);
        let cave_freq = 0.1;

        // Precompute surface height map
        let mut height_map = [[0usize; CHUNK_SIZE]; CHUNK_SIZE];
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = (chunk_x * CHUNK_SIZE as i32 + x as i32) as f64;
                let world_z = (chunk_z * CHUNK_SIZE as i32 + z as i32) as f64;

                // Smooth terrain with 5-sample blur
                let center = perlin.get([world_x * freq, world_z * freq]);
                let nx = perlin.get([(world_x + 1.0) * freq, world_z * freq]);
                let px = perlin.get([(world_x - 1.0) * freq, world_z * freq]);
                let nz = perlin.get([world_x * freq, (world_z + 1.0) * freq]);
                let pz = perlin.get([world_x * freq, (world_z - 1.0) * freq]);
                let height_noise = (center + nx + px + nz + pz) / 5.0;

                let h = ((height_noise + 1.0) * 0.5 * HEIGHT_ABOVE as f64) as usize;
                height_map[x][z] = h.min(HEIGHT_ABOVE - 1);
            }
        }

        // Fill chunk data
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let surface_y = height_map[x][z];
                let world_x = (chunk_x * CHUNK_SIZE as i32 + x as i32) as f64;
                let world_z = (chunk_z * CHUNK_SIZE as i32 + z as i32) as f64;

                for y in 0..TOTAL_HEIGHT {
                    let world_y = y as i32 - HEIGHT_BELOW as i32;

                    // Underground / cave generation
                    if world_y < 0 {
                        let n = cave_noise.get([world_x * cave_freq, world_y as f64 * cave_freq, world_z * cave_freq]);
                        if n > 0.3 {
                            data[y][z][x] = Voxel::Solid as u8;
                        }
                    } else {
                        // Surface terrain
                        if world_y as usize <= surface_y {
                            data[y][z][x] = Voxel::Solid as u8;
                        }
                    }
                }
            }
        }

        Self { data }
    }
}
