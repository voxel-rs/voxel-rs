use crate::sim::chunk::{Chunk, ChunkPos, WorldPos, BlockPos, SubIndex, InnerPos, InnerCoords};
use crate::block::BlockId;
use crate::sim::player::Player;

use hashbrown::hash_set::HashSet;

pub type PhysicsWorld = nphysics3d::world::World<f64>;
use nphysics3d::object::{ColliderHandle, BodyHandle, BodyPartHandle, RigidBodyDesc, ColliderDesc};
use nphysics3d::math::Velocity;
use ncollide3d::shape::{ShapeHandle, Cuboid};
use ncollide3d::bounding_volume::{AABB, BoundingSphere};
use nalgebra::Vector3;

lazy_static! {
    /// The shape of a chunk: 32 x 32 x 32 meters
    pub static ref CHUNK_SHAPE : ShapeHandle<f64> =
        ShapeHandle::new(Cuboid::new([32.0, 32.0, 32.0].into()));
    /// The shape of a player: 1 block at the base, 2 blocks tall
    pub static ref PLAYER_SHAPE : ShapeHandle<f64> =
        ShapeHandle::new(Cuboid::new([0.7, 1.8, 0.3].into()));
    /// A collider for the shape of a player
    pub static ref PLAYER_COLLIDER : ColliderDesc<f64> =
        ColliderDesc::new(PLAYER_SHAPE.clone());
    /// The shape of a block: 1 x 1 x 1 meters
    pub static ref BLOCK_SHAPE : ShapeHandle<f64> =
        ShapeHandle::new(Cuboid::new([1.05, 1.05, 1.05].into()));
}

/// The state of physics in the simulation
pub struct PhysicsState {
    /// The physics world
    pub world : PhysicsWorld,
    /// A list of active colliders for spawners.
    active : Vec<ColliderHandle>
}

impl PhysicsState {

    /// Create a new physics state, with no active bodies
    pub fn new() -> PhysicsState {
        let mut world = PhysicsWorld::new();
        world.set_gravity([0.0, -10.0, 0.0].into());

        let bx = Cuboid::new([10.0, 10.0, 10.0].into());
        ColliderDesc::new(ShapeHandle::new(bx))
            .build(&mut world);

        PhysicsState{ world : world, active : Vec::new() }
    }

    /// Spawn colliders, given a spawner, for a body within an AABB (if they don't already exist)
    #[allow(dead_code)]
    pub fn spawn_aabb_for<T : BVSpawner>(&mut self,
        aabb : AABB<f64>, body : BodyPartHandle, coords : T::BVCoords, spawner : &mut T) {
        let PhysicsState {
            ref mut world,
            ref mut active
        } = *self;
        spawner.spawn_aabb(coords, aabb, world, body, |handle| {active.push(handle);});
    }

    /// Spawn colliders, given a spawner, for a body within a sphere (if they don't already exist)
    #[allow(dead_code)]
    pub fn spawn_sphere_for<T : BVSpawner>(&mut self,
        sphere : BoundingSphere<f64>, body : BodyPartHandle, coords : T::BVCoords, spawner : &mut T) {
        let PhysicsState {
            ref mut world,
            ref mut active
        } = *self;
        spawner.spawn_sphere(coords, sphere, world, body, |handle| {active.push(handle);});
    }

    pub fn tick(&mut self, dt : f64) {
        self.world.set_timestep(dt);
        self.world.step();
    }

    pub fn register_player(&mut self, player : &Player) -> BodyHandle {
        if let Some(body) = player.body {
            return body;
        }
        RigidBodyDesc::new()
            .translation(player.pos)
            .mass(1.0) //TODO: this
            .velocity(Velocity::new(player.vel, Vector3::zeros()))
            .collider(&PLAYER_COLLIDER)
            .build(&mut self.world)
            .handle()
    }

    pub fn purge(&mut self) {
        self.world.remove_colliders(&self.active);
        self.active.clear();
    }

}

/// An object which contains physics objects to be spawned when an active physics object gets within
/// distance, given by a bounding volume
pub trait BVSpawner {
    type BVCoords;

    /// Spawn colliders for a body within an AABB if they don't already exist
    fn spawn_aabb<F : FnMut(ColliderHandle)>(&mut self, coords : Self::BVCoords,
        aabb : AABB<f64>, world : &mut PhysicsWorld, body : BodyPartHandle, desc : F);
    /// Spawn colliders for a body within a sphere if they don't already exist
    fn spawn_sphere<F : FnMut(ColliderHandle)>(&mut self, coords : Self::BVCoords,
        sphere : BoundingSphere<f64>, world : &mut PhysicsWorld, body : BodyPartHandle, desc : F);
}

impl BVSpawner for Chunk {
    type BVCoords = ChunkPos;

    fn spawn_aabb<F : FnMut(ColliderHandle)>(&mut self, coords : Self::BVCoords,
        aabb : AABB<f64>, world : &mut PhysicsWorld, body : BodyPartHandle, mut desc : F) {
        let mins : WorldPos = (aabb.mins().coords - coords.edge()).into();
        let maxs : WorldPos = (aabb.maxs().coords - coords.edge()).into();
        let min_blocks : BlockPos = mins.high();
        let max_blocks : BlockPos = maxs.high();
        let min_clamped = min_blocks.clamp();
        let max_clamped = max_blocks.clamp();
        for x in min_clamped.x()..max_clamped.x() {
            for y in min_clamped.y()..max_clamped.y() {
                for z in min_clamped.z()..max_clamped.z() {
                    let ic = InnerCoords::new(x, y, z).unwrap();
                    if /* !self.is_simulated(ic) &&*/ *self.get(ic) != BlockId::from(0) {
                        // Spawn a block at the appropriate position
                        //println!("Spawning collider for block @ {:?}", ic);
                        //self.set_simulated(ic, true);
                        let mut pos = coords.edge();
                        pos += Vector3::from([x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5]);
                        let collider = ColliderDesc::new(BLOCK_SHAPE.clone())
                            .translation(pos)
                            .build_with_parent(body, world)
                            .expect("Could not find body part")
                            .handle();
                        desc(collider);
                    } else {
                        //println!("Skipping collider for block @ {:?}", ic);
                    }
                }
            }
        }
    }
    fn spawn_sphere<F : FnMut(ColliderHandle)>(&mut self, coords : Self::BVCoords,
        _ : BoundingSphere<f64>, world : &mut PhysicsWorld, body : BodyPartHandle, mut desc : F) {
        //TODO: this
        if self.is_empty() {
            return;
        }
        let collider = ColliderDesc::new(CHUNK_SHAPE.clone())
            .translation(coords.center())
            .build_with_parent(body, world)
            .expect("Could not find body part")
            .handle();
        desc(collider);
    }
}
