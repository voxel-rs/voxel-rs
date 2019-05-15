pub mod worldgen;
pub mod player;
pub mod chunk;
pub mod physics;

use self::chunk::map::ChunkMap;
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
