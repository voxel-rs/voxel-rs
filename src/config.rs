extern crate serde;
extern crate toml;

use std::path::Path;
use std::fs::OpenOptions;
use std::io::{Read, Write};

/// Temporary way to configure the engine
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_player_x")]
    pub player_x: f32,
    #[serde(default = "default_player_y")]
    pub player_y: f32,
    #[serde(default = "default_player_z")]
    pub player_z: f32,
    #[serde(default = "default_mouse_speed")]
    pub mouse_speed: f32,
    #[serde(default = "default_player_speed")]
    pub player_speed: f32,
    #[serde(default = "default_ctrl_speedup")]
    pub ctrl_speedup: f32,
    #[serde(default = "default_render_distance")]
    pub render_distance: i64,
    #[serde(default = "default_tick_rate")]
    pub tick_rate: u64,
}

fn default_player_x() -> f32 {
    0.0
}

fn default_player_y() -> f32 {
    -100.0
}

fn default_player_z() -> f32 {
    0.0
}

fn default_mouse_speed() -> f32 {
    0.2
}

fn default_player_speed() -> f32 {
    5.0
}

fn default_ctrl_speedup() -> f32 {
    15.0
}

fn default_render_distance() -> i64 {
    5
}

fn default_tick_rate() -> u64 {
    2500
}

pub fn load_config(path: &Path) -> Config {
    let mut config_file = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(true)
                            .open(path).unwrap();
    // Read file
    let mut buf = "".to_owned();
    config_file.read_to_string(&mut buf).unwrap();

    let config: Config = toml::from_str(&buf).unwrap();

    // Write file
    let mut config_file = OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(path).unwrap();
    config_file.write_all(toml::to_string(&config).unwrap().as_bytes()).unwrap();
    config
}
