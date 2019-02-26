//! `Player`-related data structures.

use crate::block::ChunkPos;
use crate::config::Config;
use nalgebra::Vector3;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PlayerPos(pub [f64; 3]);

/// A player's inputs
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerInput {
    pub keys: u8,
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
    pub keys: u8,
}

impl Player {
    pub fn tick(&mut self, dt: f64, config: &Config) {
        let mut speedup = 1.0;
        if self.keys & (1 << 6) > 0 {
            speedup = config.ctrl_speedup;
        }
        if self.keys & (1 << 0) > 0 {
            self.pos += speedup * self.mv_direction(0.0) * (config.player_speed * dt);
        }
        if self.keys & (1 << 1) > 0 {
            self.pos += speedup * self.mv_direction(90.0) * (config.player_speed * dt);
        }
        if self.keys & (1 << 2) > 0 {
            self.pos += speedup * self.mv_direction(180.0) * (config.player_speed * dt);
        }
        if self.keys & (1 << 3) > 0 {
            self.pos += speedup * self.mv_direction(270.0) * (config.player_speed * dt);
        }
        if self.keys & (1 << 4) > 0 {
            self.pos.y += speedup * config.player_speed * dt;
        }
        if self.keys & (1 << 5) > 0 {
            self.pos.y -= speedup * config.player_speed * dt;
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
}
