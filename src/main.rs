#[macro_use]
extern crate gfx;

const CHUNK_SIZE: usize = 32;
const RENDER_DIST: i64 = 5;

// TODO: refactor ?
type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    constant Transform {
        view_proj: [[f32; 4]; 4] = "u_ViewProj",
        model: [[f32; 4]; 4] = "u_Model",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        image: gfx::TextureSampler<[f32; 4]> = "t_Image",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

mod block;
mod core;
mod input;
mod render;
mod texture;
mod threads;

fn main() {
    threads::input::start();
}
