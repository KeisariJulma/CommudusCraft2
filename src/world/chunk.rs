use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use std::collections::HashMap;

use crate::world::voxel::Chunk;
use crate::world::seed::WorldSeed;
use crate::world::mesher::build_chunk_mesh;
use crate::world::constants::{CHUNK_SIZE, RENDER_DISTANCE};

#[derive(Resource, Default)]
pub struct LoadedChunks {
    pub chunks: HashMap<(i32, i32), Entity>,
}

pub struct ChunkLoadTask {
    pub task: Task<((i32, i32), Mesh)>,
}

#[derive(Resource, Default)]
pub struct ChunkLoadQueue {
    pub tasks: Vec<ChunkLoadTask>,
}

pub fn update_chunks(
    mut commands: Commands,
    camera: Query<&Transform, With<Camera3d>>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut load_queue: ResMut<ChunkLoadQueue>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    world_seed: Res<WorldSeed>,
) {
    // -----------------------
    // 1️⃣ Player chunk position
    // -----------------------
    let camera_transform = camera.single().unwrap();
    let player_chunk_x = (camera_transform.translation.x / CHUNK_SIZE as f32).floor() as i32;
    let player_chunk_z = (camera_transform.translation.z / CHUNK_SIZE as f32).floor() as i32;

    // -----------------------
    // 2️⃣ Despawn far-away chunks
    // -----------------------
    let far_chunks: Vec<(i32, i32)> = loaded_chunks.chunks.iter()
        .filter_map(|(&pos, _)| {
            let dx = pos.0 - player_chunk_x;
            let dz = pos.1 - player_chunk_z;
            if dx.abs() > RENDER_DISTANCE as i32 || dz.abs() > RENDER_DISTANCE as i32 {
                Some(pos)
            } else {
                None
            }
        })
        .collect();

    for pos in far_chunks {
        if let Some(entity) = loaded_chunks.chunks.remove(&pos) {
            commands.entity(entity).despawn();
        }
    }

    // -----------------------
    // 3️⃣ Spawn new chunks asynchronously
    // -----------------------
    let thread_pool = AsyncComputeTaskPool::get();

    for dx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let chunk_pos = (player_chunk_x + dx as i32, player_chunk_z + dz as i32);
            if loaded_chunks.chunks.contains_key(&chunk_pos) { continue; }

            // spawn async task
            let seed = world_seed.0;
            let task = thread_pool.spawn(async move {
                let chunk = Chunk::new(chunk_pos.0, chunk_pos.1, seed);
                let mesh = build_chunk_mesh(&chunk);
                (chunk_pos, mesh)
            });

            load_queue.tasks.push(ChunkLoadTask { task });
        }
    }

    // -----------------------
    // 4️⃣ Poll finished tasks and spawn entities
    // -----------------------
    let mut finished = Vec::new();
    load_queue.tasks.retain_mut(|load_task| {
        if let Some(result) = future::block_on(future::poll_once(&mut load_task.task)) {
            finished.push(result);
            false
        } else {
            true
        }
    });

    for (chunk_pos, mesh) in finished {
        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.7, 0.9),
            ..default()
        });

        let entity = commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            Transform::from_xyz(
                chunk_pos.0 as f32 * CHUNK_SIZE as f32,
                0.0,
                chunk_pos.1 as f32 * CHUNK_SIZE as f32,
            ),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        )).id();

        loaded_chunks.chunks.insert(chunk_pos, entity);
    }
}
