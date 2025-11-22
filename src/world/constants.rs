
pub(crate) const CHUNK_SIZE: usize = 20;

pub const HEIGHT_ABOVE: usize = 64; // blocks above 0
pub const HEIGHT_BELOW: usize = 32; // blocks below 0 (caves)
pub const TOTAL_HEIGHT: usize = HEIGHT_ABOVE + HEIGHT_BELOW;


pub const VERTICAL_CHUNK_HEIGHT: usize = 32; // y size per vertical chunk
pub const RENDER_DISTANCE: i64 = 20;


const LOD_NEAR: f32 = 50.0;  // full detail
const LOD_MID: f32 = 150.0;  // reduced detail
const LOD_FAR: f32 = 300.0;  // very low detail / placeholder
