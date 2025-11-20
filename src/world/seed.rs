use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

#[derive(Resource)]
pub struct WorldSeed(pub u64);

impl Default for WorldSeed {
    fn default() -> Self {
        Self(12345) // default seed, can be random or user-provided
    }
}
