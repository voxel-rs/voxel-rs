#[macro_use]
extern crate gfx;
#[macro_use]
extern crate serde_derive;
extern crate cgmath;

// MUST BE A MULTIPLE OF 8 !
const CHUNK_SIZE: usize = 32;
const ICHUNK_SIZE: i64 = CHUNK_SIZE as i64;

// TODO: refactor ?
/*const PLAYER_WIDTH: f64 = 0.6;
const PLAYER_HEIGHT: f64 = 1.8;
const PLAYER_EYES: f64 = 1.6;*/
const PLAYER_WIDTH: f64 = 0.0;
const PLAYER_EYES: f64 = 0.0;
const CAMERA_OFFSET: cgmath::Vector3<f64> = cgmath::Vector3 { x: PLAYER_WIDTH/2.0, y: PLAYER_EYES, z: PLAYER_WIDTH/2.0 };

type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
        normal: [f32; 3] = "a_Normal",
    }

    constant Transform {
        view_proj: [[f32; 4]; 4] = "u_ViewProj",
        model: [[f32; 4]; 4] = "u_Model",
    }

    constant PlayerData {
        direction: [f32; 3] = "u_Direction",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        player_data: gfx::ConstantBuffer<PlayerData> = "PlayerData",
        image: gfx::TextureSampler<[f32; 4]> = "t_Image",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

mod block;
mod client;
mod config;
mod core;
mod input;
mod network;
mod player;
mod render;
mod server;
mod texture;
mod util;

fn main() {
    client::input::start();
}
