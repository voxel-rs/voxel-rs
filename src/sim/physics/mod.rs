use crate::sim::chunk::Chunk;
use crate::sim::player::Player;

use hashbrown::hash_set::HashSet;

pub type PhysicsWorld = nphysics3d::world::World<f64>;
use nphysics3d::object::{ColliderHandle, BodyHandle, RigidBodyDesc, ColliderDesc};
use nphysics3d::math::Velocity;
use ncollide3d::shape::{ShapeHandle, Cuboid};
use ncollide3d::bounding_volume::{AABB, BoundingSphere};
use nalgebra::Vector3;

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
        PhysicsState{ world : PhysicsWorld::new(), active : HashSet::new() }
    }

    /// Spawn colliders, given a spawner, for a body within an AABB (if they don't already exist)
    #[allow(dead_code)]
    pub fn spawn_aabb_for<T : BVSpawner>(&mut self,
        aabb : AABB<f64>, body : BodyHandle, spawner : &T) {
        let PhysicsState {
            ref mut world,
            ref mut active
        } = *self;
        spawner.spawn_aabb(aabb, world, body, |handle| {active.insert(handle);});
    }

    /// Spawn colliders, given a spawner, for a body within a sphere (if they don't already exist)
    #[allow(dead_code)]
    pub fn spawn_sphere_for<T : BVSpawner>(&mut self,
        sphere : BoundingSphere<f64>, body : BodyHandle, spawner : &T) {
        let PhysicsState {
            ref mut world,
            ref mut active
        } = *self;
        spawner.spawn_sphere(sphere, world, body, |handle| {active.insert(handle);});
    }

    pub fn tick(&mut self, dt : f64) {
        self.world.set_timestep(dt);
        self.world.step();
    }

    pub fn register_player(&mut self, player : &Player) -> BodyHandle {
        if let Some(body) = player.body {
            return body;
        }
        // 1 block at the base, 2 blocks high
        let shape = ShapeHandle::new(Cuboid::new([1.0, 2.0, 1.0].into()));
        let collider = ColliderDesc::new(shape);
        RigidBodyDesc::new()
            .translation(player.pos)
            .mass(1.0) //TODO: this
            .velocity(Velocity::new(player.vel, Vector3::zeros()))
            .collider(&collider)
            .build(&mut self.world)
            .handle()
    }

}

/// An object which contains physics objects to be spawned when an active physics object gets within
/// distance, given by a bounding volume
pub trait BVSpawner {
    /// Spawn colliders for a body within an AABB if they don't already exist
    fn spawn_aabb<F : FnMut(ColliderHandle)>(&self,
        aabb : AABB<f64>, world : &mut PhysicsWorld, body : BodyHandle, desc : F);
    /// Spawn colliders for a body within a sphere if they don't already exist
    fn spawn_sphere<F : FnMut(ColliderHandle)>(&self,
        sphere : BoundingSphere<f64>, world : &mut PhysicsWorld, body : BodyHandle, desc : F);
}

impl BVSpawner for Chunk {
    fn spawn_aabb<F : FnMut(ColliderHandle)>(&self,
        aabb : AABB<f64>, world : &mut PhysicsWorld, body : BodyHandle, desc : F) {
        //TODO: this
    }
    fn spawn_sphere<F : FnMut(ColliderHandle)>(&self,
        sphere : BoundingSphere<f64>, world : &mut PhysicsWorld, body : BodyHandle, desc : F) {
        //TODO: this
    }
}
