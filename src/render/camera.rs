extern crate cgmath;

use self::cgmath::prelude::*;
use self::cgmath::{Deg, Euler, Matrix4, Quaternion, Vector3, perspective};
use ::input::KeyboardState;

// TODO: Don't hardcode this
const MOVE_FORWARD: u32 = 17;
const MOVE_LEFT: u32 = 30;
const MOVE_BACKWARD: u32 = 31;
const MOVE_RIGHT: u32 = 32;
const MOVE_UP: u32 = 57;
const MOVE_DOWN: u32 = 42;
const CONTROL: u32 = 29;

pub struct Camera {
    position: Vector3<f32>,
    yaw: Deg<f32>,
    pitch: Deg<f32>,
    win_w: u32,
    win_h: u32,
    mouse_speed: Deg<f32>,
    player_speed: f32,
    ctrl_speedup: f32,
}

impl Camera {
    pub fn new(win_w: u32, win_h: u32, config: &::config::Config) -> Camera {
        Camera {
            position: Vector3::from([0.0, 0.0, 4.0]),
            yaw: Deg(0.0),
            pitch: Deg(0.0),
            win_w,
            win_h,
            mouse_speed: Deg(config.mouse_speed),
            player_speed: config.player_speed,
            ctrl_speedup: config.ctrl_speedup,
        }
    }

    // TODO: Allow mouse inverting
    pub fn update_cursor(&mut self, dx: f32, dy: f32) {
        self.yaw += -self.mouse_speed * (dx as f32);
        self.pitch += -self.mouse_speed * (dy as f32);

        // Ensure the pitch stays within [-90; 90]
        if self.pitch < Deg(-90.0) {
            self.pitch = Deg(-90.0);
        }
        if self.pitch > Deg(90.0) {
            self.pitch = Deg(90.0);
        }
    }

    fn get_mv_direction(&self, angle: Deg<f32>) -> Vector3<f32> {
        let yaw = self.yaw + angle;
        Vector3 {
            x: -yaw.sin(),
            y: 0.0,
            z: -yaw.cos(),
        }
    }

    pub fn tick(&mut self, dt: f32, keys: &KeyboardState) {
        let mut speedup = 1.0;
        if keys.is_key_pressed(CONTROL)
        { speedup = self.ctrl_speedup; }
        if keys.is_key_pressed(MOVE_FORWARD)
        { self.position += speedup * self.get_mv_direction(Deg(0.0)).normalize() * (self.player_speed * dt); }
        if keys.is_key_pressed(MOVE_LEFT)
        { self.position += speedup * self.get_mv_direction(Deg(90.0)).normalize() * (self.player_speed * dt); }
        if keys.is_key_pressed(MOVE_BACKWARD)
        { self.position += speedup * self.get_mv_direction(Deg(180.0)).normalize() * (self.player_speed * dt); }
        if keys.is_key_pressed(MOVE_RIGHT)
        { self.position += speedup * self.get_mv_direction(Deg(270.0)).normalize() * (self.player_speed * dt); }
        if keys.is_key_pressed(MOVE_UP)
        { self.position.y += speedup * self.player_speed * dt; }
        if keys.is_key_pressed(MOVE_DOWN)
        { self.position.y -= speedup * self.player_speed * dt; }
    }

    fn get_aspect_ratio(&self) -> f32 {
        self.win_w as f32 / self.win_h as f32
    }

    pub fn resize_window(&mut self, win_w: u32, win_h: u32) {
        self.win_w = win_w;
        self.win_h = win_h;
    }

    pub fn get_view_projection(&self) -> Matrix4<f32> {
        let proj = perspective(Deg(45.0), self.get_aspect_ratio(), 0.1, 400.0);

        let rotation = Quaternion::from(Euler { x: Deg(0.0), y: self.yaw, z: Deg(0.0) }) * Quaternion::from(Euler { x: self.pitch, y: Deg(0.0), z: Deg(0.0) });
        let translation = Matrix4::from_translation(self.position);

        proj * (translation * Matrix4::from(rotation)).invert().unwrap()
    }

    pub fn get_pos(&self) -> (f32, f32, f32) {
        (self.position[0], self.position[1], self.position[2])
    }
}
