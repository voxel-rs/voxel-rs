extern crate cgmath;

use self::cgmath::prelude::*;
use self::cgmath::{Deg, Euler, Matrix4, Quaternion, Vector3, perspective};
use ::player::PlayerPos;

// TODO: Don't hardcode this
pub const MOVE_FORWARD: u32 = 17;
pub const MOVE_LEFT: u32 = 30;
pub const MOVE_BACKWARD: u32 = 31;
pub const MOVE_RIGHT: u32 = 32;
pub const MOVE_UP: u32 = 57;
pub const MOVE_DOWN: u32 = 42;
pub const CONTROL: u32 = 29;

pub struct Camera {
    position: Vector3<f64>,
    yaw: Deg<f64>,
    pitch: Deg<f64>,
    win_w: u32,
    win_h: u32,
    mouse_speed: Deg<f64>,
}

impl Camera {
    pub fn new(win_w: u32, win_h: u32, config: &::config::Config) -> Camera {
        Camera {
            position: Vector3::from([config.player_x, config.player_y, config.player_z]),
            yaw: Deg(0.0),
            pitch: Deg(0.0),
            win_w,
            win_h,
            mouse_speed: Deg(config.mouse_speed),
        }
    }

    // TODO: Allow mouse inverting
    pub fn update_cursor(&mut self, dx: f64, dy: f64) {
        self.yaw += -self.mouse_speed * (dx as f64);
        self.pitch += -self.mouse_speed * (dy as f64);

        // Ensure the pitch stays within [-90; 90]
        if self.pitch < Deg(-90.0) {
            self.pitch = Deg(-90.0);
        }
        if self.pitch > Deg(90.0) {
            self.pitch = Deg(90.0);
        }
    }

    fn get_aspect_ratio(&self) -> f64 {
        self.win_w as f64 / self.win_h as f64
    }

    pub fn resize_window(&mut self, win_w: u32, win_h: u32) {
        self.win_w = win_w;
        self.win_h = win_h;
    }

    pub fn get_view_projection(&self) -> Matrix4<f64> {
        let proj = perspective(Deg(45.0), self.get_aspect_ratio(), 0.1, 400.0);

        let rotation = Quaternion::from(Euler { x: Deg(0.0), y: self.yaw, z: Deg(0.0) }) * Quaternion::from(Euler { x: self.pitch, y: Deg(0.0), z: Deg(0.0) });
        let translation = Matrix4::from_translation(self.position);

        proj * (translation * Matrix4::from(rotation)).invert().unwrap()
    }

    pub fn get_pos(&self) -> PlayerPos {
        PlayerPos(self.position.into())
    }

    pub fn set_pos(&mut self, new_pos: [f64; 3]) {
        self.position = new_pos.into();
    }

    pub fn get_cam_dir(&self) -> Vector3<f64> {
        Vector3 {
            x: -self.pitch.cos() * self.yaw.sin(),
            y:  self.pitch.sin(),
            z: -self.pitch.cos() * self.yaw.cos(),
        }
    }

    pub fn get_yaw_pitch(&self) -> [f64; 2] {
        [self.yaw.0, self.pitch.0]
    }
}
