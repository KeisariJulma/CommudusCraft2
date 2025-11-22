use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use std::collections::HashMap;

use crate::world::voxel::{Chunk, Voxel};
use crate::world::constants::{CHUNK_SIZE, HEIGHT_BELOW, RENDER_DISTANCE, TOTAL_HEIGHT, VERTICAL_CHUNK_HEIGHT};
use crate::utils::light::Fullbright;
use crate::world::mesher::{build_chunk_mesh, build_vertical_chunk_mesh};
use crate::WorldSeed;


#[derive(Resource, Default)]
pub struct ChunkManager {
    pub loaded_chunks: HashMap<(i32, i32, i32), Entity>, // now includes vertical layer
    pub load_queue: Vec<ChunkLoadTask>,
}


pub struct ChunkLoadTask {
    pub task: Task<((i32, i32, i32), Chunk, Mesh)>,
}

#[derive(Component)]
pub struct ChunkComponent(pub Chunk);

impl Chunk {
    pub fn empty() -> Self {
        Self {
            data: [[[Voxel::Air as u8; CHUNK_SIZE]; CHUNK_SIZE]; TOTAL_HEIGHT],
        }
    }
}

/*

/// Spawn a placeholder chunk and enqueue async mesh generation
fn spawn_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_pos: (i32, i32),
    seed: u64,
    fullbright: bool,
    xray: bool,
    chunk_manager: &mut ChunkManager,
) {
    let chunk_world_x = chunk_pos.0 as f32 * CHUNK_SIZE as f32;
    let chunk_world_z = chunk_pos.1 as f32 * CHUNK_SIZE as f32;

    // place chunk at correct Y = world base
    let chunk_world_y = -(HEIGHT_BELOW as f32); // -32

    let mut mesh = Mesh::from(Cuboid::new(
        CHUNK_SIZE as f32,
        TOTAL_HEIGHT as f32,
        CHUNK_SIZE as f32,
    ));

    // shift AABB upward so bottom matches -HEIGHT_BELOW (-32)
    let shift_y = (TOTAL_HEIGHT as f32 / 2.0) - (HEIGHT_BELOW as f32);
    mesh.translate_by(Vec3::new(0.0, shift_y, 0.0));

    let placeholder_mesh = meshes.add(mesh);

    let placeholder_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),
        unlit: fullbright,
        alpha_mode: if xray { AlphaMode::Blend } else { AlphaMode::Opaque },
        ..default()
    });

    let entity = commands
        .spawn((
            ChunkComponent(Chunk::empty()),
            Mesh3d(placeholder_mesh),
            MeshMaterial3d(placeholder_material),
            Transform::from_xyz(chunk_world_x, chunk_world_y, chunk_world_z),
            GlobalTransform::default(),
            Visibility::default(),
        ))
        .id();

    chunk_manager.loaded_chunks.insert(chunk_pos, entity);

    // Async chunk generation
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        let chunk = Chunk::new(chunk_pos.0, chunk_pos.1, seed);
        let mesh = build_chunk_mesh(&chunk);
        (chunk_pos, chunk, mesh)
    });
    chunk_manager.load_queue.push(ChunkLoadTask { task });
}*/

/// Update chunks around the camera
pub(crate) fn update_chunks(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera: Query<&Transform, With<Camera3d>>,
    world_seed: Res<WorldSeed>,
    fullbright: Res<Fullbright>,
) {
    let camera_transform = match camera.single() {
        Ok(t) => t,
        Err(_) => return,
    };

    let camera_pos = camera_transform.translation;
    let player_chunk_x = (camera_pos.x / CHUNK_SIZE as f32).floor() as i32;
    let player_chunk_z = (camera_pos.z / CHUNK_SIZE as f32).floor() as i32;

    const VIEW_DISTANCE: i32 = RENDER_DISTANCE as i32;
    const VERTICAL_BUFFER: f32 = 20.0; // in blocks
    let visible_distance = VIEW_DISTANCE as f32 * CHUNK_SIZE as f32;

    // -----------------------------
    // Despawn far chunks
    // -----------------------------
    let mut to_despawn = Vec::new();
    for (&(cx, cz, layer), &entity) in chunk_manager.loaded_chunks.iter() {
        let chunk_world_x = cx as f32 * CHUNK_SIZE as f32;
        let chunk_world_z = cz as f32 * CHUNK_SIZE as f32;
        let chunk_world_y = -(HEIGHT_BELOW as f32) + layer as f32 * VERTICAL_CHUNK_HEIGHT as f32;

        let horizontal_dist_sq = (chunk_world_x - camera_pos.x).powi(2)
            + (chunk_world_z - camera_pos.z).powi(2);
        let vertical_dist = (chunk_world_y - camera_pos.y).abs();

        if horizontal_dist_sq > visible_distance.powi(2)
            || vertical_dist > VERTICAL_BUFFER * CHUNK_SIZE as f32
        {
            to_despawn.push((cx, cz, layer));
        }
    }

    for key in to_despawn {
        if let Some(entity) = chunk_manager.loaded_chunks.remove(&key) {
            commands.entity(entity).despawn();
        }
    }

    // -----------------------------
    // Spawn new chunks
    // -----------------------------
    let num_vertical_chunks = (TOTAL_HEIGHT as f32 / VERTICAL_CHUNK_HEIGHT as f32).ceil() as i32;

    for dx in -VIEW_DISTANCE..=VIEW_DISTANCE {
        for dz in -VIEW_DISTANCE..=VIEW_DISTANCE {
            let base_chunk_x = player_chunk_x + dx;
            let base_chunk_z = player_chunk_z + dz;

            // Only spawn vertical layers that are missing
            for layer in 0..num_vertical_chunks {
                let key = (base_chunk_x, base_chunk_z, layer);
                if chunk_manager.loaded_chunks.contains_key(&key) {
                    continue;
                }

                let chunk_world_x = base_chunk_x as f32 * CHUNK_SIZE as f32;
                let chunk_world_z = base_chunk_z as f32 * CHUNK_SIZE as f32;
                let horizontal_dist_sq = (chunk_world_x - camera_pos.x).powi(2)
                    + (chunk_world_z - camera_pos.z).powi(2);

                if horizontal_dist_sq > visible_distance.powi(2) {
                    continue;
                }

                // Spawn vertical chunk
                if !chunk_manager.loaded_chunks.contains_key(&key) {
                    spawn_vertical_chunk_layer(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        (base_chunk_x, base_chunk_z),
                        layer,
                        world_seed.0,
                        fullbright.0,
                        false,
                        &mut chunk_manager,
                    );
                }

                break; // spawn per (x,z) only once; all vertical layers handled inside
            }
        }
    }
}





/// Poll finished async chunk tasks and update placeholder entities
pub(crate) fn poll_chunk_tasks(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    fullbright: Res<Fullbright>,
) {
    let mut finished = Vec::new();

    // Poll async tasks
    chunk_manager.load_queue.retain_mut(|load_task| {
        if let Some(result) = future::block_on(future::poll_once(&mut load_task.task)) {
            finished.push(result);
            false
        } else {
            true
        }
    });

    for (chunk_pos, chunk, mesh) in finished {
        // chunk_pos = (i32, i32, layer)
        if let Some(&entity) = chunk_manager.loaded_chunks.get(&chunk_pos) {
            let mesh_handle = meshes.add(mesh);
            let material_handle = materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.8, 0.5),
                unlit: fullbright.0,
                emissive: if fullbright.0 {
                    LinearRgba::from(Color::WHITE)
                } else {
                    LinearRgba::from(Color::BLACK)
                },
                ..default()
            });

            commands.entity(entity)
                .insert(ChunkComponent(chunk))
                .insert(Mesh3d(mesh_handle))
                .insert(MeshMaterial3d(material_handle));
        }
    }
}




fn spawn_vertical_chunk_layer(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_pos: (i32, i32),
    layer: i32,
    seed: u64,
    fullbright: bool,
    xray: bool,
    chunk_manager: &mut ChunkManager,
) {
    let chunk_world_x = chunk_pos.0 as f32 * CHUNK_SIZE as f32;
    let chunk_world_z = chunk_pos.1 as f32 * CHUNK_SIZE as f32;

    // world Y position for this vertical slice
    let chunk_world_y = -(HEIGHT_BELOW as f32) + layer as f32 * VERTICAL_CHUNK_HEIGHT as f32;

    // Placeholder mesh for this slice only
    let mut mesh = Mesh::from(Cuboid::new(
        CHUNK_SIZE as f32,
        VERTICAL_CHUNK_HEIGHT as f32,
        CHUNK_SIZE as f32,
    ));
    mesh.translate_by(Vec3::new(0.0, VERTICAL_CHUNK_HEIGHT as f32 / 2.0, 0.0));
    let placeholder_mesh = meshes.add(mesh);

    let placeholder_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.0), // fully transparent
        alpha_mode: AlphaMode::Blend,                // enable transparency
        unlit: true,                                 // ignore lighting
        ..default()
    });



    let entity = commands.spawn((
        ChunkComponent(Chunk::empty()),
        Mesh3d(placeholder_mesh),
        MeshMaterial3d(placeholder_material),
        Transform::from_xyz(chunk_world_x, chunk_world_y, chunk_world_z),
        GlobalTransform::default(),
        Visibility::default(),
    )).id();

    chunk_manager.loaded_chunks.insert((chunk_pos.0, chunk_pos.1, layer), entity);

    // Async mesh generation for this vertical slice
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        let chunk = Chunk::new(chunk_pos.0, chunk_pos.1, seed);
        let mesh = build_vertical_chunk_mesh(&chunk, layer as usize * VERTICAL_CHUNK_HEIGHT);
        ((chunk_pos.0, chunk_pos.1, layer), chunk, mesh)
    });

    chunk_manager.load_queue.push(ChunkLoadTask { task });
}
