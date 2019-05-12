//! Various `Block`- and `Chunk`-related data structures.
use crate::texture::TextureRegistry;
use crate::Vertex;
use serde_derive::{Deserialize, Serialize};
use derive_more::{From};

/// Block representation
pub trait Block {
    /// Append the block's vertices to the current Vertex Buffers
    /// TODO: Use the Vertex type instead of Vec<>
    fn render(&self, vertices: &mut Vec<Vertex>, adj: u8, delta: [u64; 3]);
    /// Does this block hide adjacent blocks ?
    fn is_opaque(&self) -> bool;
}

/// A block's id
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct BlockId(pub u16);

pub struct BlockRegistry {
    blocks: Vec<BlockObj>,
}

pub type BlockRef = Box<Block + Send + Sync>;

#[derive(From)]
pub enum BlockObj {
    Gas(BlockGas),
    Cube(BlockCube),
    Dynamic(BlockRef)
}

impl Block for BlockObj {
    fn render(&self, vertices: &mut Vec<Vertex>, adj: u8, delta: [u64; 3]) {
        match self {
            BlockObj::Gas(g) => g.render(vertices, adj, delta),
            BlockObj::Cube(c) => c.render(vertices, adj, delta),
            BlockObj::Dynamic(r) => r.render(vertices, adj, delta)
        }
    }
    /// Does this block hide adjacent blocks ?
    fn is_opaque(&self) -> bool {
        match self {
            BlockObj::Gas(g) => g.is_opaque(),
            BlockObj::Cube(c) => c.is_opaque(),
            BlockObj::Dynamic(r) => r.is_opaque()
        }
    }
}

pub struct BlockCube {
    uvs: [[[f32; 2]; 4]; 6],
}

pub struct BlockGas {}
impl BlockRegistry {
    pub fn new() -> BlockRegistry {
        BlockRegistry { blocks: Vec::new() }
    }

    pub fn add_block(&mut self, block: BlockObj) -> BlockId {
        self.blocks.push(block);
        BlockId::from((self.blocks.len() - 1) as u16)
    }

    pub fn get_block(&self, id: BlockId) -> &BlockObj {
        &self.blocks[id.0 as usize]
    }
}

impl From<u16> for BlockId {
    fn from(id: u16) -> Self {
        BlockId(id)
    }
}

impl Block for BlockCube {
    fn render(&self, vertices: &mut Vec<Vertex>, adj: u8, delta: [u64; 3]) {
        for face in 0..6 {
            if adj & (1 << face) > 0 {
                let side = &FACES[face as usize];
                for &pos in &FACE_ORDER {
                    let mut coords = VERTICES[side[pos]];
                    for i in 0..3 {
                        coords[i] += delta[i] as f32;
                    }
                    let uv_coords = self.uvs[face as usize][pos];
                    vertices.push(Vertex {
                        pos: [coords[0], coords[1], coords[2], 1.],
                        uv: uv_coords,
                        normal: NORMALS[face as usize].clone(),
                    });
                }
            }
        }
    }

    fn is_opaque(&self) -> bool {
        true
    }
}

/// Create a solid block with the provided textures
pub fn create_block_cube(texture_names: [&str; 6], textures: &TextureRegistry) -> BlockCube {
    let mut uvs = [[[-1.; 2]; 4]; 6];
    for i in 0..6 {
        let rect = textures.get_position(&texture_names[i]);
        for j in 0..4 {
            let (x, y) = rect.get_pos((UVS[j][0], UVS[j][1]));
            uvs[i][j][0] = x;
            uvs[i][j][1] = y;
        }
    }
    BlockCube { uvs }
}

/// Create an air block
pub fn create_block_air() -> BlockGas {
    BlockGas {}
}

impl Block for BlockGas {
    fn render(&self, _: &mut Vec<Vertex>, _: u8, _: [u64; 3]) {}

    fn is_opaque(&self) -> bool {
        false
    }
}

// ```
// 0     1
// +-----+
// |   / |
// |  /  |
// | /   |
// +-----+
// 3     2
// ```
const FACES: [[usize; 4]; 6] = [
    [5, 4, 0, 1],
    [7, 6, 2, 3],
    [6, 5, 1, 2],
    [4, 7, 3, 0],
    [6, 7, 4, 5],
    [1, 0, 3, 2],
];

const VERTICES: [[f32; 3]; 8] = [
    [0., 0., 0.],
    [1., 0., 0.],
    [1., 0., 1.],
    [0., 0., 1.],
    [0., 1., 0.],
    [1., 1., 0.],
    [1., 1., 1.],
    [0., 1., 1.],
];

const UVS: [[f32; 2]; 4] = [[0., 0.], [1., 0.], [1., 1.], [0., 1.]];

const FACE_ORDER: [usize; 6] = [0, 3, 1, 1, 3, 2];

const NORMALS: [[f32; 3]; 6] = [
    [0., 0., -1.],
    [0., 0., 1.],
    [1., 0., 0.],
    [-1., 0., 0.],
    [0., 1., 0.],
    [0., -1., 0.],
];
