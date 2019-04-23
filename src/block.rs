//! Various `Block`- and `Chunk`-related data structures.

use std::collections::hash_map::Entry;
use crate::texture::TextureRegistry;
use crate::{Vertex, CHUNK_SIZE};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

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
    blocks: Vec<BlockRef>,
}

pub type ChunkFragment = [BlockId; CHUNK_SIZE];
pub type ChunkArray = [[ChunkFragment; CHUNK_SIZE]; CHUNK_SIZE];
pub type ChunkSidesArray = [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
pub type BlockRef = Box<Block + Send + Sync>;
/// Indicates what non-void ```ChunkFragment```s a Chunk contains.
/// It is stored as 32-bit integers so that common functions are implemented.
pub type ChunkInfo = [u32; CHUNK_SIZE * CHUNK_SIZE / 32];

#[derive(Debug, Copy, Clone)]
pub struct InnerChunkPos([u8; 3]);

impl InnerChunkPos {
    fn coord_to_inner(coord : f64) -> u8 {
        let mod_coord = coord as i64 % CHUNK_SIZE as i64;
        (if mod_coord < 0 {
            CHUNK_SIZE as i64 + mod_coord
        } else {
            mod_coord
        }) as u8
    }
    pub fn from_coords(coords : [f64; 3]) -> InnerChunkPos {
        InnerChunkPos(
            [
                InnerChunkPos::coord_to_inner(coords[0]),
                InnerChunkPos::coord_to_inner(coords[1]),
                InnerChunkPos::coord_to_inner(coords[2])
            ]
        )
    }
}

pub struct ChunkMap{
    map : HashMap<ChunkPos, ChunkState>,
    hot : HashMap<ChunkPos, ()>
}

impl ChunkMap {

    pub fn new() -> ChunkMap {
        ChunkMap {
            map : HashMap::new(),
            hot : HashMap::new()
        }
    }

    pub fn get_mut(&mut self, pos : &ChunkPos) -> Option<&mut ChunkState> {
        self.map.get_mut(pos)
    }

    pub fn get(&self, pos : &ChunkPos) -> Option<&ChunkState> {
        self.map.get(pos)
    }

    pub fn entry(&mut self, pos : ChunkPos) -> Entry<ChunkPos, ChunkState> {
        self.map.entry(pos)
    }

    pub fn retain<F : FnMut(&ChunkPos, &mut ChunkState) -> bool>(&mut self, f : F) {
        self.map.retain(f)
    }

    pub fn is_hot(&self, pos : &ChunkPos) -> bool {
        self.hot.get(pos) != None
    }

    pub fn heat(&mut self, pos : ChunkPos) {
        self.hot.insert(pos, ());
    }

    pub fn cool_all(&mut self) {
        self.hot.clear()
    }

    pub fn contains_key(&self, pos : &ChunkPos) -> bool {
        return self.map.contains_key(pos);
    }

    pub fn set(&mut self, pos : ChunkPos, i_pos : InnerChunkPos, block : BlockId) {
        let mut heated = false;
        match self.get_mut(&pos) {
            None => {print!("Failed to set {:?} : {:?} to {:?}!\n", pos, i_pos, block);},
            Some(ref mut state) => {
                print!("Setting {:?} : {:?} to {:?}!\n", pos, i_pos, block);
                state.set(block, i_pos);
                heated = true;
            }
        }
        if heated {
            self.heat(pos);
        }

    }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChunkState {
    Generating,
    Generated(Box<ChunkArray>),
    Modified(Box<ChunkArray>)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChunkContents {
    Generated(Box<ChunkArray>),
    Modified(Box<ChunkArray>)
}

impl ChunkContents {

    pub fn iter(&self) -> impl Iterator<Item = &[ChunkFragment; CHUNK_SIZE]> {
        match self {
            ChunkContents::Generated(ref c) | ChunkContents::Modified(ref c) => c.iter()
        }
    }

}

impl From<ChunkContents> for ChunkState {

    fn from(status : ChunkContents) -> ChunkState {
        match status {
            ChunkContents::Generated(c) => ChunkState::Generated(c),
            ChunkContents::Modified(c) => ChunkState::Modified(c)
        }
    }

}

impl From<ChunkState> for Option<ChunkContents> {

    fn from(state : ChunkState) -> Option<ChunkContents> {
        match state {
            ChunkState::Generating => None,
            ChunkState::Generated(c) => Some(ChunkContents::Generated(c)),
            ChunkState::Modified(c) => Some(ChunkContents::Modified(c))
        }
    }

}

impl ChunkState {

    pub fn is_modified(&self) -> bool {
        match self {
            ChunkState::Modified(_) => true,
            _ => false
        }
    }

    pub fn set(&mut self, block : BlockId, _i_pos : InnerChunkPos) {
        match self {
            ChunkState::Generating => panic!("Can't spawn in chunk yet to be generated!"),
            ChunkState::Generated(ref mut arr) | ChunkState::Modified(ref mut arr) => {
                //let x = i_pos.0[0] as usize;
                //let y = i_pos.0[1] as usize;
                //let z = i_pos.0[2] as usize;
                for i in arr.iter() {
                    for j in i.iter() {
                        for k in j.iter() {
                            if *k != BlockId::from(0) {
                                print!("Nonzero block ID {:?} detected!\n", k);
                            }
                        }
                    }
                }
                *self = ChunkState::Modified(Box::new([[[BlockId::from(0); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]))
                //arr[x][y][z] = block;
            }
        }
    }

}


/// Chunk type
#[derive(Clone, Debug)]
pub struct Chunk {
    /// Blocks in the chunk
    pub blocks: Box<ChunkArray>,
    /// Empty blocks adjacent to the chunk (1 is for non-opaque, 0 is for opaque)
    pub sides: Box<ChunkSidesArray>,
}

// TODO: Struct instead ?
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ChunkPos(pub [i64; 3]);

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FragmentPos(pub [usize; 2]);

pub struct BlockCube {
    uvs: [[[f32; 2]; 4]; 6],
}

pub struct BlockAir {}
impl BlockRegistry {
    pub fn new() -> BlockRegistry {
        BlockRegistry { blocks: Vec::new() }
    }

    pub fn add_block(&mut self, block: BlockRef) -> BlockId {
        self.blocks.push(block);
        BlockId::from((self.blocks.len() - 1) as u16)
    }

    pub fn get_block(&self, id: BlockId) -> &BlockRef {
        &self.blocks[id.0 as usize]
    }
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            blocks: Box::new([[[BlockId(0); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]),
            sides: Box::new([[[0b00000000; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]),
        }
    }

    pub fn calculate_mesh(&self, blocks: &BlockRegistry) -> Vec<Vertex> {
        let mut vec: Vec<Vertex> = Vec::new();
        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for k in 0..CHUNK_SIZE {
                    // Don't render hidden blocks
                    if self.sides[i][j][k] != 0xFF {
                        blocks.get_block(self.blocks[i][j][k]).render(
                            &mut vec,
                            self.sides[i][j][k],
                            [i as u64, j as u64, k as u64],
                        );
                    }
                }
            }
        }
        vec
    }
}

impl ChunkPos {
    pub fn orthogonal_dist(self, other: ChunkPos) -> u64 {
        let mut maxcoord = 0;
        for i in 0..3 {
            maxcoord = i64::max(maxcoord, (other.0[i] - self.0[i]).abs());
        }
        maxcoord as u64
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
pub fn create_block_air() -> BlockAir {
    BlockAir {}
}

impl Block for BlockAir {
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
