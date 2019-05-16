pub mod worldgen;
pub mod player;
pub mod chunk;
pub mod physics;

use self::chunk::map::ChunkMap;
use self::player::PlayerSet;
use self::physics::PhysicsState;

pub struct World {
    pub chunks : ChunkMap,
    pub players : PlayerSet,
    pub physics : PhysicsState
}

impl World {

    pub fn new() -> World { World {
        chunks : ChunkMap::new(),
        players : PlayerSet::new(),
        physics : PhysicsState::new()
    }}

}
