pub mod worldgen;
pub mod player;

use crate::block::ChunkMap;
use self::player::PlayerSet;

pub struct World {
    pub chunks : ChunkMap,
    pub players : PlayerSet
}

impl World {

    pub fn new() -> World { World {
        chunks : ChunkMap::new(),
        players : PlayerSet::new()
    }}

}
