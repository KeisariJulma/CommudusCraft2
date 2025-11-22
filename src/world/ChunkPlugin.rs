use bevy::prelude::*;
use crate::WorldSeed;use super::chunk_manager::{ChunkManager, update_chunks, poll_chunk_tasks,};

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        // Insert the ChunkManager resource
        app.insert_resource(ChunkManager::default())
            // System to update which chunks are loaded/despawned
            .add_systems(Update, update_chunks)
            // System to poll finished async tasks and update chunk entities
            .add_systems(Update, poll_chunk_tasks);
    }
}
