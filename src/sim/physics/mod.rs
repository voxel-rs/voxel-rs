use crate::sim::chunk::{Chunk, ChunkPos};
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
        ShapeHandle::new(Cuboid::new([1.0, 2.0, 1.0].into()));
    /// A collider for the shape of a player
    pub static ref PLAYER_COLLIDER : ColliderDesc<f64> =
        ColliderDesc::new(PLAYER_SHAPE.clone());
    /// The shape of a block: 1 x 1 x 1 meters
    pub static ref BLOCK_SHAPE : ShapeHandle<f64> =
        ShapeHandle::new(Cuboid::new([1.0, 1.0, 1.0].into()));
}

/// The state of physics in the simulation
pub struct PhysicsState {
    /// The physics world
    pub world : PhysicsWorld,
    /// A list of active colliders for spawners.
    /// "Garbage collected" by checking if colliders from other bodies are nearby.
    /// TODO: think of a way to garbage collect colliders for nearby bodies known not to collide
    /// with each other, other than maybe a collision group...
    active : HashSet<ColliderHandle>
}

impl PhysicsState {

    /// Create a new physics state, with no active bodies
    pub fn new() -> PhysicsState {
        let mut world = PhysicsWorld::new();
        world.set_gravity([0.0, -10.0, 0.0].into());

        let bx = Cuboid::new([10.0, 10.0, 10.0].into());
        ColliderDesc::new(ShapeHandle::new(bx))
            .build(&mut world);

        PhysicsState{ world : world, active : HashSet::new() }
    }

    /// Spawn colliders, given a spawner, for a body within an AABB (if they don't already exist)
    #[allow(dead_code)]
    pub fn spawn_aabb_for<T : BVSpawner>(&mut self,
        aabb : AABB<f64>, body : BodyPartHandle, coords : T::BVCoords, spawner : &T) {
        let PhysicsState {
            ref mut world,
            ref mut active
        } = *self;
        spawner.spawn_aabb(coords, aabb, world, body, |handle| {active.insert(handle);});
    }

    /// Spawn colliders, given a spawner, for a body within a sphere (if they don't already exist)
    #[allow(dead_code)]
    pub fn spawn_sphere_for<T : BVSpawner>(&mut self,
        sphere : BoundingSphere<f64>, body : BodyPartHandle, coords : T::BVCoords, spawner : &T) {
        let PhysicsState {
            ref mut world,
            ref mut active
        } = *self;
        spawner.spawn_sphere(coords, sphere, world, body, |handle| {active.insert(handle);});
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

}

/// An object which contains physics objects to be spawned when an active physics object gets within
/// distance, given by a bounding volume
pub trait BVSpawner {
    type BVCoords;

    /// Spawn colliders for a body within an AABB if they don't already exist
    fn spawn_aabb<F : FnMut(ColliderHandle)>(&self, coords : Self::BVCoords,
        aabb : AABB<f64>, world : &mut PhysicsWorld, body : BodyPartHandle, desc : F);
    /// Spawn colliders for a body within a sphere if they don't already exist
    fn spawn_sphere<F : FnMut(ColliderHandle)>(&self, coords : Self::BVCoords,
        sphere : BoundingSphere<f64>, world : &mut PhysicsWorld, body : BodyPartHandle, desc : F);
}

impl BVSpawner for Chunk {
    type BVCoords = ChunkPos;

    fn spawn_aabb<F : FnMut(ColliderHandle)>(&self, coords : Self::BVCoords,
        _aabb : AABB<f64>, world : &mut PhysicsWorld, body : BodyPartHandle, mut desc : F) {
        //TODO: this (current implementation just a test)
        if self.is_empty() {
            return;
        }
        // Check if something is inside the desired collider:
        
        let collider = ColliderDesc::new(CHUNK_SHAPE.clone())
            .translation(coords.center())
            .build_with_parent(body, world)
            .expect("Could not find body part")
            .handle();
        desc(collider);
    }
    fn spawn_sphere<F : FnMut(ColliderHandle)>(&self, coords : Self::BVCoords,
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
