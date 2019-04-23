//! `Player`-related data structures.

use glutin::ElementState;
use std::ops::BitOrAssign;
use crate::block::{ChunkMap, ChunkPos, ChunkState, BlockId};
use crate::config::Config;
use nalgebra::Vector3;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::BitOr;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PlayerPos(pub [f64; 3]);

// Invidual key controls
#[derive(Debug, Clone, Copy)]
pub enum PlayerKey {
    Forward,
    Left,
    Backward,
    Right,
    Up,
    Down,
    Control,
    Hit
}

// A player's currentcontrols
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PlayerControls {
    keys : u8
}

impl PlayerControls {
    fn key_bitmap(addr : u32) -> PlayerControls {
        PlayerControls{
            keys : (1 << addr) as u8
        }
    }
    fn get_bitmap(self, addr : u32) -> bool {
        (self.keys & ((1 << addr) as u8)) > 0
    }

    pub fn none() -> PlayerControls {
        PlayerControls { keys : 0 }
    }

    pub fn pressed(self, key : PlayerKey) -> bool {
        self.get_bitmap(key as u32)
    }

    pub fn mouse(mouse_state : ElementState) -> PlayerControls {
        match mouse_state {
            ElementState::Pressed => PlayerKey::Hit.into(),
            _ => PlayerControls::none()
        }
    }
}

impl From<PlayerKey> for PlayerControls {
    fn from(key : PlayerKey) -> PlayerControls {
        return PlayerControls::key_bitmap(key as u32);
    }
}

impl BitOr for PlayerControls {
    type Output = PlayerControls;

    fn bitor(self, other : PlayerControls) -> PlayerControls {
        PlayerControls {
            keys : self.keys | other.keys
        }
    }
}

impl BitOrAssign for PlayerControls {
    fn bitor_assign(&mut self, rhs : Self) {
        self.keys |= rhs.keys;
    }
}

/// A player's inputs
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerInput {
    pub keys: PlayerControls,
    /// Yaw in degrees
    pub yaw: f64,
    /// Pitch in degrees
    pub pitch: f64,
}

/// A server-side player
pub struct Player {
    pub pos: Vector3<f64>,
    /// Yaw in degrees
    pub yaw: f64,
    /// Pitch in degrees
    pub pitch: f64,
    pub render_distance: u64,
    pub chunks: HashMap<ChunkPos, ()>,
    pub keys: PlayerControls,
}

impl Player {

    fn handle_hit(&mut self, _dt: f64, _config: &Config, world: &mut ChunkMap) {
        let inner_pos = self.get_pos().inner_chunk_pos();
        world.set(&self.get_pos().chunk_pos(), BlockId::from(0), inner_pos[0], inner_pos[1], inner_pos[2])
    }

    pub fn tick(&mut self, dt: f64, config: &Config, world: &mut ChunkMap) {
        let mut speedup = 1.0;
        if self.keys.pressed(PlayerKey::Control) {
            speedup = config.ctrl_speedup;
        }

        let old_pos = self.pos.clone();
        if self.keys.pressed(PlayerKey::Forward) {
            self.pos += speedup * self.mv_direction(0.0) * (config.player_speed * dt);
        }
        if self.keys.pressed(PlayerKey::Left) {
            self.pos += speedup * self.mv_direction(90.0) * (config.player_speed * dt);
        }
        if self.keys.pressed(PlayerKey::Backward) {
            self.pos += speedup * self.mv_direction(180.0) * (config.player_speed * dt);
        }
        if self.keys.pressed(PlayerKey::Right) {
            self.pos += speedup * self.mv_direction(270.0) * (config.player_speed * dt);
        }
        if self.keys.pressed(PlayerKey::Up) {
            self.pos.y += speedup * config.player_speed * dt;
        }
        if self.keys.pressed(PlayerKey::Down) {
            self.pos.y -= speedup * config.player_speed * dt;
        }

        if self.keys.pressed(PlayerKey::Hit) {
            self.handle_hit(dt, config, world);
        }

        let chunk_pos = self.get_pos().chunk_pos();
        // Can't move to an unloaded chunk
        if !world.contains_key(&chunk_pos) {
            self.pos = old_pos;
        } else if let &ChunkState::Generating = world.get(&chunk_pos).unwrap() {
            self.pos = old_pos;
        }
    }

    fn mv_direction(&self, angle: f64) -> Vector3<f64> {
        let yaw = self.yaw + angle;
        Vector3::new(-yaw.to_radians().sin(), 0.0, -yaw.to_radians().cos()).normalize()
    }

    pub fn get_pos(&self) -> PlayerPos {
        PlayerPos(self.pos.into())
    }

    pub fn set_input(&mut self, input: &PlayerInput) {
        self.keys = input.keys;
        self.yaw = input.yaw;
        self.pitch = input.pitch;
    }
}

impl PlayerPos {
    pub fn chunk_pos(self) -> ChunkPos {
        use crate::CHUNK_SIZE;
        let mut ret = [0; 3];
        for i in 0..3 {
            ret[i] = self.0[i] as i64 / CHUNK_SIZE as i64
                - if (self.0[i] as i64 % CHUNK_SIZE as i64) < 0 {
                    1
                } else {
                    0
                };
        }
        ChunkPos(ret)
    }
    pub fn inner_chunk_pos(self) -> [usize; 3] {
        use crate::CHUNK_SIZE;
        let mut x = self.0[0] as i64 % CHUNK_SIZE as i64;
        let mut y = self.0[1] as i64 % CHUNK_SIZE as i64;
        let mut z = self.0[2] as i64 % CHUNK_SIZE as i64;
        if x < 0 {
            x = CHUNK_SIZE as i64 + x;
        }
        if y < 0 {
            y = CHUNK_SIZE as i64 + y;
        }
        if z < 0 {
            z = CHUNK_SIZE as i64 + z;
        }
        [x as usize, y as usize, z as usize]
    }
}
