extern crate cgmath;

use ::block::ChunkPos;
use ::config::Config;

use ::std::collections::HashMap;

use self::cgmath::{Deg, Vector3};
use self::cgmath::prelude::*;

pub type PlayerPos = (f64, f64, f64);

/// A player's inputs
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerInput {
    pub keys: u8,
    /// Yaw in degrees
    pub yaw: f32,
    /// Yaw in degrees
    pub pitch: f32,
}

/// A server-side player
pub struct Player {
    pub pos: Vector3<f32>,
    pub yaw: Deg<f32>,
    pub pitch: Deg<f32>,
    pub render_distance: u64,
    pub chunks: HashMap<ChunkPos, ()>,
    pub keys: u8,
}

impl Player {
    pub fn tick(&mut self, dt: f32, config: &Config) {
        let mut speedup = 1.0;
        if self.keys & (1 << 6) > 0
        { speedup = config.ctrl_speedup; }
        if self.keys & (1 << 0) > 0
        { self.pos += speedup * self.mv_direction(Deg(0.0)) * (config.player_speed * dt); }
        if self.keys & (1 << 1) > 0
        { self.pos += speedup * self.mv_direction(Deg(90.0)) * (config.player_speed * dt); }
        if self.keys & (1 << 2) > 0
        { self.pos += speedup * self.mv_direction(Deg(180.0)) * (config.player_speed * dt); }
        if self.keys & (1 << 3) > 0
        { self.pos += speedup * self.mv_direction(Deg(270.0)) * (config.player_speed * dt); }
        if self.keys & (1 << 4) > 0
        { self.pos.y += speedup * config.player_speed * dt; }
        if self.keys & (1 << 5) > 0
        { self.pos.y -= speedup * config.player_speed * dt; }
    }

    fn mv_direction(&self, angle: Deg<f32>) -> Vector3<f32> {
        let yaw = self.yaw + angle;
        Vector3 {
            x: -yaw.sin(),
            y: 0.0,
            z: -yaw.cos(),
        }.normalize()
    }

    pub fn get_pos(&self) -> PlayerPos {
        (self.pos[0] as f64, self.pos[1] as f64, self.pos[2] as f64)
    }

    pub fn set_input(&mut self, input: &PlayerInput) {
        self.keys = input.keys;
        self.yaw = Deg(input.yaw);
        self.pitch = Deg(input.pitch);
    }
}
