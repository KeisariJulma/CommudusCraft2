use std::collections::HashMap;
use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::MeshAabb;
use bevy::prelude::*;
use wgpu_types::PrimitiveTopology;
use crate::world::voxel::{Chunk, Voxel};
use crate::world::constants::{CHUNK_SIZE, TOTAL_HEIGHT, VERTICAL_CHUNK_HEIGHT};
use bevy::mesh::{Mesh, Indices};

#[derive(Clone, Copy)]
pub(crate) enum CubeFace { Top, Bottom, Left, Right, Front, Back }
impl CubeFace { fn all() -> [CubeFace; 6] { [CubeFace::Top, CubeFace::Bottom, CubeFace::Left, CubeFace::Right, CubeFace::Front, CubeFace::Back] } }

struct FaceVertices { positions: Vec<[f32;3]>, normals: Vec<[f32;3]>, indices: Vec<u32> }

fn face_vertices(face: CubeFace, x: usize, y: usize, z: usize, vertex_offset: u32) -> FaceVertices {
    let xf = x as f32; let yf = y as f32; let zf = z as f32;
    match face {
        CubeFace::Top => FaceVertices {
            positions: vec![
                [xf, yf + 1.0, zf],
                [xf + 1.0, yf + 1.0, zf],
                [xf + 1.0, yf + 1.0, zf + 1.0],
                [xf, yf + 1.0, zf + 1.0],
            ],
            normals: vec![[0.0, 1.0, 0.0]; 4],
            indices: vec![0, 2, 1, 0, 3, 2]
                .iter()
                .map(|i| i + vertex_offset)
                .collect(),
        },
        CubeFace::Bottom => FaceVertices{positions: vec![[xf,yf,zf],[xf+1.0,yf,zf],[xf+1.0,yf,zf+1.0],[xf,yf,zf+1.0]], normals: vec![[0.0,-1.0,0.0];4], indices: vec![0,2,1,0,3,2].iter().map(|i| i+vertex_offset).collect()},
        CubeFace::Left   => FaceVertices{positions: vec![[xf,yf,zf],[xf,yf+1.0,zf],[xf,yf+1.0,zf+1.0],[xf,yf,zf+1.0]], normals: vec![[-1.0,0.0,0.0];4], indices: vec![0,1,2,0,2,3].iter().map(|i| i+vertex_offset).collect()},
        CubeFace::Right  => FaceVertices{positions: vec![[xf+1.0,yf,zf],[xf+1.0,yf+1.0,zf],[xf+1.0,yf+1.0,zf+1.0],[xf+1.0,yf,zf+1.0]], normals: vec![[1.0,0.0,0.0];4], indices: vec![0,2,1,0,3,2].iter().map(|i| i+vertex_offset).collect()},
        CubeFace::Front  => FaceVertices{positions: vec![[xf,yf,zf+1.0],[xf+1.0,yf,zf+1.0],[xf+1.0,yf+1.0,zf+1.0],[xf,yf+1.0,zf+1.0]], normals: vec![[0.0,0.0,1.0];4], indices: vec![0,2,1,0,3,2].iter().map(|i| i+vertex_offset).collect()},
        CubeFace::Back   => FaceVertices{positions: vec![[xf,yf,zf],[xf+1.0,yf,zf],[xf+1.0,yf+1.0,zf],[xf,yf+1.0,zf]], normals: vec![[0.0,0.0,-1.0];4], indices: vec![0,1,2,0,2,3].iter().map(|i| i+vertex_offset).collect()},
    }
}

pub fn neighbor_is_air(
    loaded_chunks: &HashMap<(i32, i32), Chunk>,
    chunk_x: i32,
    chunk_z: i32,
    x: isize,
    y: isize,
    z: isize,
    face: CubeFace,
) -> bool {
    let (nx, ny, nz) = match face {
        CubeFace::Top    => (x, y + 1, z),
        CubeFace::Bottom => (x, y - 1, z),
        CubeFace::Left   => (x - 1, y, z),
        CubeFace::Right  => (x + 1, y, z),
        CubeFace::Front  => (x, y, z + 1),
        CubeFace::Back   => (x, y, z - 1),
    };

    // If above or below chunk bounds in y, treat as air
    if ny < 0 || ny >= TOTAL_HEIGHT as isize {
        return true;
    }

    // Determine which chunk the neighbor is in
    let mut neighbor_chunk_x = chunk_x;
    let mut neighbor_chunk_z = chunk_z;
    let mut local_x = nx;
    let mut local_z = nz;

    if nx < 0 {
        neighbor_chunk_x -= 1;
        local_x += CHUNK_SIZE as isize;
    } else if nx >= CHUNK_SIZE as isize {
        neighbor_chunk_x += 1;
        local_x -= CHUNK_SIZE as isize;
    }

    if nz < 0 {
        neighbor_chunk_z -= 1;
        local_z += CHUNK_SIZE as isize;
    } else if nz >= CHUNK_SIZE as isize {
        neighbor_chunk_z += 1;
        local_z -= CHUNK_SIZE as isize;
    }

    // Get the neighbor chunk
    if let Some(neighbor_chunk) = loaded_chunks.get(&(neighbor_chunk_x, neighbor_chunk_z)) {
        let lx = local_x as usize;
        let ly = ny as usize;
        let lz = local_z as usize;

        neighbor_chunk.data[ly][lz][lx] == Voxel::Air as u8
    } else {
        // Neighbor chunk not loaded yet = treat as air
        true
    }
}



pub(crate) fn build_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_offset = 0u32;

    for y in 0..TOTAL_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.data[y][z][x] != Voxel::Solid as u8 {
                    continue;
                }

                for face in CubeFace::all() {
                    // Check if neighbor inside the chunk is air
                    let neighbor_air = match face {
                        CubeFace::Top => y + 1 >= TOTAL_HEIGHT || chunk.data[y + 1][z][x] == Voxel::Air as u8,
                        CubeFace::Bottom => y == 0 || chunk.data[y - 1][z][x] == Voxel::Air as u8,
                        CubeFace::Left => x == 0 || chunk.data[y][z][x - 1] == Voxel::Air as u8,
                        CubeFace::Right => x + 1 >= CHUNK_SIZE || chunk.data[y][z][x + 1] == Voxel::Air as u8,
                        CubeFace::Front => z + 1 >= CHUNK_SIZE || chunk.data[y][z + 1][x] == Voxel::Air as u8,
                        CubeFace::Back => z == 0 || chunk.data[y][z - 1][x] == Voxel::Air as u8,
                    };

                    if neighbor_air {
                        let fv = face_vertices(face, x, y, z, vertex_offset);
                        vertex_offset += fv.positions.len() as u32;
                        positions.extend(fv.positions);
                        normals.extend(fv.normals);
                        indices.extend(fv.indices);
                    }
                }
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));

    mesh.compute_aabb();
    mesh
}




pub(crate) fn build_vertical_chunk_mesh(chunk: &Chunk, y_offset: usize) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_offset = 0u32;

    // Define vertical slice bounds
    let start_y = y_offset;
    let end_y = (y_offset + VERTICAL_CHUNK_HEIGHT).min(TOTAL_HEIGHT);

    for y in start_y..end_y {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.data[y][z][x] != Voxel::Solid as u8 {
                    continue;
                }

                for face in CubeFace::all() {
                    // Check if neighbor inside the chunk is air
                    let neighbor_air = match face {
                        CubeFace::Top => y + 1 >= TOTAL_HEIGHT || chunk.data[y + 1][z][x] == Voxel::Air as u8,
                        CubeFace::Bottom => y == 0 || chunk.data[y - 1][z][x] == Voxel::Air as u8,
                        CubeFace::Left => x == 0 || chunk.data[y][z][x - 1] == Voxel::Air as u8,
                        CubeFace::Right => x + 1 >= CHUNK_SIZE || chunk.data[y][z][x + 1] == Voxel::Air as u8,
                        CubeFace::Front => z + 1 >= CHUNK_SIZE || chunk.data[y][z + 1][x] == Voxel::Air as u8,
                        CubeFace::Back => z == 0 || chunk.data[y][z - 1][x] == Voxel::Air as u8,
                    };

                    if neighbor_air {
                        // Make vertex positions relative to this vertical chunk
                        let fv = face_vertices(face, x, y - y_offset, z, vertex_offset);
                        vertex_offset += fv.positions.len() as u32;
                        positions.extend(fv.positions);
                        normals.extend(fv.normals);
                        indices.extend(fv.indices);
                    }
                }
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));

    mesh.compute_aabb();
    mesh
}

