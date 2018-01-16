use ::block::ChunkPos;

use ::std::collections::HashMap;

pub type PlayerPos = (f64, f64, f64);

/// A server-side player
pub struct Player {
    pub pos: PlayerPos,
    pub render_distance: u64,
    pub chunks: HashMap<ChunkPos, ()>,
    pub keys: u8,
}
