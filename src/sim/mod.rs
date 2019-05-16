use crate::config::Config;

pub mod worldgen;
pub mod player;
pub mod chunk;
pub mod physics;

use self::chunk::map::ChunkMap;
use self::player::PlayerSet;
use self::physics::PhysicsState;

/// The entire state of the game world
pub struct World {
    /// The chunks in the world
    pub chunks : ChunkMap,
    /// The players in the world
    pub players : PlayerSet,
    /// The state of physics in the world
    pub physics : PhysicsState,
    /// The number of ticks that have occured in this world
    pub ticks : u64
}

impl World {

    /// Create a new, empty game world
    pub fn new() -> World { World {
        chunks : ChunkMap::new(),
        players : PlayerSet::new(),
        physics : PhysicsState::new(),
        ticks : 0
    }}

    /// Tick the game world forward by dt
    pub fn tick(&mut self, dt : f64, config : &Config) {

        // Stage 0: Bookkeeping
        self.ticks += 1;

        // Stage 1: Mobs and players are updated about other mobs and players
        //TODO: this

        // Stage 2: Mobs and players make their moves, edit the world.
        for p in self.players.iter_mut() {
            p.tick(dt, config, &mut self.chunks, &mut self.physics);
        }

        // Stage 3: Mobs and players act on each other
        //TODO:this

        // Stage 4: Active objects affect the physics world by loading and unloading spawn objects
        //TODO: this

        // Stage 5: The physics world ticks forwards, affecting all objects in it
        // (including the bodies of mobs and players)
        self.physics.tick(dt);

        // Stage 6: Information is synced between mobs and the physics world
        for p in self.players.iter_mut() {
            p.sync_physics(config, &mut self.physics);
        }

    }

    pub fn physics_gc(&mut self, _config : &Config) {
        //TODO: this
    }

    pub fn chunk_gc(&mut self, _config : &Config) {
        //TODO: this
    }

}
