use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh};
use wgpu_types::PrimitiveTopology;
use crate::world::voxel::{Chunk, Voxel};
use crate::world::constants::{CHUNK_SIZE, HEIGHT};

#[derive(Clone, Copy)]
enum CubeFace { Top, Bottom, Left, Right, Front, Back }
impl CubeFace { fn all() -> [CubeFace; 6] { [CubeFace::Top, CubeFace::Bottom, CubeFace::Left, CubeFace::Right, CubeFace::Front, CubeFace::Back] } }

struct FaceVertices { positions: Vec<[f32;3]>, normals: Vec<[f32;3]>, indices: Vec<u32> }

fn face_vertices(face: CubeFace, x: usize, y: usize, z: usize, vertex_offset: u32) -> FaceVertices {
    let xf = x as f32; let yf = y as f32; let zf = z as f32;
    match face {
        CubeFace::Top => FaceVertices{positions: vec![[xf,yf+1.0,zf],[xf+1.0,yf+1.0,zf],[xf+1.0,yf+1.0,zf+1.0],[xf,yf+1.0,zf+1.0]], normals: vec![[0.0,1.0,0.0];4], indices: vec![0,1,2,0,2,3].iter().map(|i| i+vertex_offset).collect()},
        CubeFace::Bottom => FaceVertices{positions: vec![[xf,yf,zf],[xf+1.0,yf,zf],[xf+1.0,yf,zf+1.0],[xf,yf,zf+1.0]], normals: vec![[0.0,-1.0,0.0];4], indices: vec![0,2,1,0,3,2].iter().map(|i| i+vertex_offset).collect()},
        CubeFace::Left => FaceVertices{positions: vec![[xf,yf,zf],[xf,yf+1.0,zf],[xf,yf+1.0,zf+1.0],[xf,yf,zf+1.0]], normals: vec![[-1.0,0.0,0.0];4], indices: vec![0,1,2,0,2,3].iter().map(|i| i+vertex_offset).collect()},
        CubeFace::Right => FaceVertices{positions: vec![[xf+1.0,yf,zf],[xf+1.0,yf+1.0,zf],[xf+1.0,yf+1.0,zf+1.0],[xf+1.0,yf,zf+1.0]], normals: vec![[1.0,0.0,0.0];4], indices: vec![0,2,1,0,3,2].iter().map(|i| i+vertex_offset).collect()},
        CubeFace::Front => FaceVertices{positions: vec![[xf,yf,zf+1.0],[xf+1.0,yf,zf+1.0],[xf+1.0,yf+1.0,zf+1.0],[xf,yf+1.0,zf+1.0]], normals: vec![[0.0,0.0,1.0];4], indices: vec![0,2,1,0,3,2].iter().map(|i| i+vertex_offset).collect()},
        CubeFace::Back => FaceVertices{positions: vec![[xf,yf,zf],[xf+1.0,yf,zf],[xf+1.0,yf+1.0,zf],[xf,yf+1.0,zf]], normals: vec![[0.0,0.0,-1.0];4], indices: vec![0,1,2,0,2,3].iter().map(|i| i+vertex_offset).collect()},
    }
}

fn neighbor_is_air(chunk: &Chunk, x: isize, y: isize, z: isize, face: CubeFace) -> bool {
    let (nx, ny, nz) = match face {
        CubeFace::Top    => (x, y + 1, z),
        CubeFace::Bottom => (x, y - 1, z),
        CubeFace::Left   => (x - 1, y, z),
        CubeFace::Right  => (x + 1, y, z),
        CubeFace::Front  => (x, y, z + 1),
        CubeFace::Back   => (x, y, z - 1),
    };

    // Out-of-bounds means air
    if nx < 0 || ny < 0 || nz < 0 || nx >= CHUNK_SIZE as isize || ny >= HEIGHT as isize || nz >= CHUNK_SIZE as isize {
        return true;
    }

    let idx = (ny as usize) * CHUNK_SIZE * CHUNK_SIZE
            + (nz as usize) * CHUNK_SIZE
            + (nx as usize);

    chunk.data[idx] == Voxel::Air as u8
}

pub fn build_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_offset = 0u32;

    for y in 0..HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let idx = y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x;
                if chunk.data[idx] == Voxel::Solid as u8 {
                    for face in CubeFace::all() {
                        if neighbor_is_air(chunk, x as isize, y as isize, z as isize, face) {
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
    }


    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
