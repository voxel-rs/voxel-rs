use nphysics3d::world::World;
use nphysics3d::object::ColliderDesc;

/// An object which contains physics objects to be spawned when an active physics object gets within
/// distance, given by a bounding volume
pub trait BVSpawner {
    fn spawn_aabb(&self, world : &mut World<f64>, desc : &mut Vec<ColliderDesc<f64>>);
    fn spawn_sphere(&self, world : &mut World<f64>, desc : &mut Vec<ColliderDesc<f64>>);
}
