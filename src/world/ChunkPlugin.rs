use bevy::prelude::*;
use crate::world::chunk::{update_chunks, ChunkLoadQueue, LoadedChunks};

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        // Insert resources first
        app.insert_resource(LoadedChunks::default());
        app.insert_resource(ChunkLoadQueue::default());

        // Then add the system
        app.add_systems(Update, update_chunks);
    }
}
