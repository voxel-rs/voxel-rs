#[macro_use]
extern crate gfx;
#[macro_use]
extern crate lazy_static;
extern crate num;

// MUST BE A MULTIPLE OF 8 !
const CHUNK_SIZE: usize = 32;

// TODO: refactor ?
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
mod render;
mod server;
mod simple;
mod texture;
mod util;
mod sim;

fn main() {
    client::input::start();
}
