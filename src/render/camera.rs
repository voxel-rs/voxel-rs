//! Camera.

use crate::config::Config;
use crate::sim::chunk::WorldPos;
use nalgebra::{Matrix4, Perspective3, Vector3};

// TODO: Don't hardcode this
pub const MOVE_FORWARD: u32 = 17;
pub const MOVE_LEFT: u32 = 30;
pub const MOVE_BACKWARD: u32 = 31;
pub const MOVE_RIGHT: u32 = 32;
pub const MOVE_UP: u32 = 57;
pub const MOVE_DOWN: u32 = 42;
pub const CONTROL: u32 = 29;
pub const PHYSICS_ENABLE : u32 = 25; // P

pub struct Camera {
    position: Vector3<f64>,
    /// Yaw in degrees
    yaw: f64,
    /// Yaw in degrees
    pitch: f64,
    win_w: u32,
    win_h: u32,
    /// In degrees/pixel
    mouse_speed: f64,
}

impl Camera {
    pub fn new(win_w: u32, win_h: u32, config: &Config) -> Camera {
        Camera {
            position: Vector3::from([config.player_x, config.player_y, config.player_z]),
            yaw: 0.0,
            pitch: 0.0,
            win_w,
            win_h,
            mouse_speed: config.mouse_speed,
        }
    }

    // TODO: Allow mouse inverting
    pub fn update_cursor(&mut self, dx: f64, dy: f64) {
        self.yaw += -self.mouse_speed * (dx as f64);
        self.pitch += -self.mouse_speed * (dy as f64);

        // Ensure the pitch stays within [-90; 90]
        if self.pitch < -90.0 {
            self.pitch = -90.0;
        }
        if self.pitch > 90.0 {
            self.pitch = 90.0;
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
        let proj = Perspective3::new(self.get_aspect_ratio(), (45.0f64).to_radians(), 0.1, 400.0);

        let rotation = Matrix4::from_euler_angles(-self.pitch.to_radians(), 0.0, 0.0)
            * Matrix4::from_euler_angles(0.0, -self.yaw.to_radians(), 0.0);
        let translation = Matrix4::new_translation(&-self.position);

        proj.as_matrix() * rotation * translation
    }

    pub fn get_pos(&self) -> WorldPos {
        self.position.into()
    }

    pub fn set_pos(&mut self, new_pos: Vector3<f64>) {
        self.position = new_pos;
    }

    pub fn get_cam_dir(&self) -> Vector3<f64> {
        Vector3::new(
            -self.pitch.to_radians().cos() * self.yaw.to_radians().sin(),
            self.pitch.to_radians().sin(),
            -self.pitch.to_radians().cos() * self.yaw.to_radians().cos(),
        )
    }

    pub fn get_yaw_pitch(&self) -> [f64; 2] {
        [self.yaw, self.pitch]
    }
}
