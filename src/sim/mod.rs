use crate::config::Config;
use nphysics3d::object::BodyPartHandle;

pub mod worldgen;
pub mod player;
pub mod chunk;
pub mod physics;

use self::chunk::map::ChunkMap;
use self::player::PlayerSet;
use self::physics::PhysicsState;
use self::chunk::{ChunkState, SubIndex};

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

        // Stage 1: Active objects affect the physics world by loading and unloading spawn objects
        for p in self.players.iter_mut() {
            let chunk_pos = p.get_pos().high();
            if let Some(ChunkState::Generated(chunk)) = self.chunks.get_mut(&chunk_pos) {
                self.physics.spawn_aabb_for(
                    p.get_aabb_around(4.0),
                    BodyPartHandle::ground(),
                    chunk_pos,
                    chunk
                )
            }
        }

        // Stage 2: Mobs and players are updated about other mobs and players
        //TODO: this

        // Stage 3: Mobs and players make their moves, edit the world.
        for p in self.players.iter_mut() {
            p.tick(dt, config, &mut self.chunks, &mut self.physics);
        }

        // Stage 4: Mobs and players act on each other
        //TODO:this

        // Stage 5: The physics world ticks forwards, affecting all objects in it
        // (including the bodies of mobs and players)
        self.physics.tick(dt);

        // Stage 6: Information is synced between mobs and the physics world before the end of the tick
        for p in self.players.iter_mut() {
            p.finalize(config, &mut self.chunks, &mut self.physics);
        }

        // Stage 7: the physics world is purged of active objects
        self.physics_gc(config);

    }

    pub fn physics_gc(&mut self, _config : &Config) {
        self.physics.purge();
    }

    #[allow(dead_code)]
    pub fn chunk_gc(&mut self, _config : &Config) {
        //TODO: this
    }

}
