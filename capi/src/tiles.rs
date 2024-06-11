use crate::effects::TILES_PER_AXIS;

pub const PIXELS_PER_TILE_AXIS: usize = 8;
pub const PIXELS_PER_AXIS: usize = TILES_PER_AXIS * PIXELS_PER_TILE_AXIS;
pub const NUM_TILES: usize = TILES_PER_AXIS * TILES_PER_AXIS;
