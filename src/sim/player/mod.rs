//! `Player`-related data structures.
use glutin::ElementState;
use super::chunk::{map::ChunkMap, ChunkPos, InnerCoords, ChunkState, WorldPos, SubIndex};
use super::physics::PhysicsState;
use crate::block::BlockId;
use crate::config::Config;
use nalgebra::Vector3;
use serde_derive::{Deserialize, Serialize};
use enumset::{EnumSet, EnumSetType};

use nphysics3d::object::BodyHandle;
use nphysics3d::math::{Velocity, Isometry};

use ncollide3d::bounding_volume::aabb::AABB;

mod player_set;
pub use player_set::PlayerSet;
pub use player_set::PlayerId;

/// Invidual key controls
#[derive(Debug, Serialize, Deserialize, EnumSetType)]
pub enum PlayerKey {
    Forward,
    Left,
    Backward,
    Right,
    Up,
    Down,
    Control,
    Hit,
    PhysicsEnable
}

/// A player's current controls
pub type PlayerControls = EnumSet<PlayerKey>;

pub trait FromMouse {
    fn mouse(mouse_state : ElementState) -> Self;
}

impl FromMouse for PlayerControls {
    fn mouse(mouse_state : ElementState) -> PlayerControls {
        match mouse_state {
            ElementState::Pressed => PlayerKey::Hit.into(),
            _ => PlayerControls::new()
        }
    }
}

/// A player's inputs
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerInput {
    /// The keys the player has pressed down
    pub keys: PlayerControls,
    /// Yaw in degrees
    pub yaw: f64,
    /// Pitch in degrees
    pub pitch: f64,
}

/// A server-side player
pub struct Player {
    /// This player's position
    pub pos: Vector3<f64>,
    // This player's position last tick
    pub old_pos : Vector3<f64>,
    /// This player's *desired* velocity
    pub vel : Vector3<f64>,
    /// Yaw in degrees
    pub yaw: f64,
    /// Pitch in degrees
    pub pitch: f64,
    /// The render distance for this player
    pub render_distance: u64,
    /// The keys the player has pressed down
    pub keys: PlayerControls,
    /// Player ID
    pub id: PlayerId,
    /// Whether this player is active
    pub active : bool,
    /// Whether this player has physics enabled
    pub physics : bool,
    /// This player's associated body in the physics world
    pub body : Option<BodyHandle>
}

impl Player {

    pub fn new(id : PlayerId, pos : Vector3<f64>, active : bool) -> Player {
        Player {
            pos: pos,
            old_pos : pos,
            vel : [0.0, 0.0, 0.0].into(),
            yaw: 0.0,
            pitch: 0.0,
            render_distance: 0,
            keys: PlayerControls::new(),
            id : id,
            active : active,
            physics : false,
            body : None
        }
    }

    pub fn get_aabb_around(&self, rad : f64) -> AABB<f64> {
        AABB::from_half_extents(self.pos.into(), [rad, rad * 2.0, rad].into())
    }

    fn handle_hit(&mut self, _dt: f64, _config: &Config, world: &mut ChunkMap) {
        let p = self.get_pos();
        let (h, l) : (ChunkPos, InnerCoords) = p.factor();
        world.set(h, l, BlockId::from(0))
    }

    pub fn tick(&mut self, dt: f64, config: &Config, world: &mut ChunkMap, physics : &mut PhysicsState) {

        // Don't tick inactive players
        if !self.active {return;}

        let mut speedup = 1.0;
        if self.keys.contains(PlayerKey::Control) {
            speedup = config.ctrl_speedup;
            if self.physics {println!("Physics off!");}
            self.physics = false;
        }
        if self.keys.contains(PlayerKey::PhysicsEnable) {
            if !self.physics {println!("Physics on!");}
            self.physics = true;
            if let Some(body) = self.body {
                physics.world.rigid_body_mut(body)
                    .expect("Cannot load body")
                    .set_position(Isometry::new(self.pos, Vector3::zeros()))
            }
        }

        if self.body.is_none() {
            self.body = Some(physics.register_player(self));
            println!("Set body!");
        }

        if !self.physics {
            self.vel = [0.0, 0.0, 0.0].into();
        } else {
            self.vel.x /= 2.0;
            self.vel.z /= 2.0;
            if self.vel.y > 0.0 {
                self.vel.y /= 2.0
            }
        }
        //TODO: maximum speed
        if self.keys.contains(PlayerKey::Forward) {
            self.vel += speedup * self.mv_direction(0.0) * (config.player_speed);
        }
        if self.keys.contains(PlayerKey::Left) {
            self.vel += speedup * self.mv_direction(90.0) * (config.player_speed);
        }
        if self.keys.contains(PlayerKey::Backward) {
            self.vel += speedup * self.mv_direction(180.0) * (config.player_speed);
        }
        if self.keys.contains(PlayerKey::Right) {
            self.vel += speedup * self.mv_direction(270.0) * (config.player_speed);
        }
        //if !self.physics {
            if self.keys.contains(PlayerKey::Up) {
                self.vel.y += speedup * config.player_speed;
            }
            if self.keys.contains(PlayerKey::Down) {
                self.vel.y -= speedup * config.player_speed;
            }
        //}

        if self.keys.contains(PlayerKey::Hit) {
            self.handle_hit(dt, config, world);
        }

        // TODO: integrate physics
        if !self.physics {
            self.pos += self.vel * dt;
        } else if let Some(body) = self.body {
            physics.world.rigid_body_mut(body)
                .expect("Cannot load body")
                .set_velocity(Velocity::new(self.vel, Vector3::zeros()));
        }

    }

    pub fn finalize(&mut self, _config : &Config, world: &mut ChunkMap, physics : &mut PhysicsState) {
        if let Some(body) = self.body {
            let rigid_body = physics.world.rigid_body(body).expect("Cannot load body");
            if self.physics {
                self.pos = rigid_body.position().translation.vector;
                self.vel = rigid_body.velocity().linear;
            }
        }

        let chunk_pos : ChunkPos = self.get_pos().high();
        // Can't move to an unloaded chunk
        if !world.contains_key(&chunk_pos) {
            self.pos = self.old_pos;
        } else if let &ChunkState::Generating = world.get(&chunk_pos).unwrap() {
            self.pos = self.old_pos;
        } else {
            self.old_pos = self.pos;
        }
    }

    #[inline]
    fn mv_direction(&self, angle: f64) -> Vector3<f64> {
        let yaw = self.yaw + angle;
        Vector3::new(-yaw.to_radians().sin(), 0.0, -yaw.to_radians().cos()).normalize()
    }

    pub fn get_pos(&self) -> WorldPos {
        self.pos.into()
    }

    pub fn set_input(&mut self, input: &PlayerInput) {
        self.keys = input.keys;
        self.yaw = input.yaw;
        self.pitch = input.pitch;
    }
}
