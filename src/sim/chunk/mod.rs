use crate::block::{Block, BlockId, BlockRegistry};
use crate::CHUNK_SIZE;
use crate::Vertex;
use serde_derive::{Deserialize, Serialize};
use hashbrown::hash_map::HashMap;
use hashbrown::hash_map::Entry;
use hashbrown::hash_map::DefaultHashBuilder;

pub type BlockData = BlockId;
pub type ChunkFragment = [BlockData; CHUNK_SIZE];
pub type ChunkArray = [[ChunkFragment; CHUNK_SIZE]; CHUNK_SIZE];
pub type ChunkSidesArray = [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

mod pos;
pub use pos::*;

pub struct ChunkMap{
    map : HashMap<ChunkPos, ChunkState>
}

impl ChunkMap {

    pub fn new() -> ChunkMap {
        ChunkMap {
            map : HashMap::default()
        }
    }

    pub fn get_mut(&mut self, pos : &ChunkPos) -> Option<&mut ChunkState> {
        self.map.get_mut(pos)
    }

    pub fn get(&self, pos : &ChunkPos) -> Option<&ChunkState> {
        self.map.get(pos)
    }

    pub fn entry(&mut self, pos : ChunkPos) -> Entry<ChunkPos, ChunkState, DefaultHashBuilder> {
        self.map.entry(pos)
    }

    pub fn retain<F : FnMut(&ChunkPos, &mut ChunkState) -> bool>(&mut self, f : F) {
        self.map.retain(f)
    }

    pub fn contains_key(&self, pos : &ChunkPos) -> bool {
        return self.map.contains_key(pos);
    }

    pub fn set(&mut self, pos : ChunkPos, i_pos : InnerChunkPos, block : BlockId) {
        match self.get_mut(&pos) {
            None => {print!("Failed to set {:?} : {:?} to {:?}!\n", pos, i_pos, block);},
            Some(ref mut state) => {
                print!("Setting {:?} : {:?} to {:?}!\n", pos, i_pos, block);
                state.set(block, i_pos);
            }
        }

    }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChunkState {
    Generating,
    Generated(Box<ChunkArray>, u64)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChunkContents(pub Box<ChunkArray>, pub u64);

impl ChunkContents {

    pub fn iter(&self) -> impl Iterator<Item = &[ChunkFragment; CHUNK_SIZE]> {
        self.0.iter()
    }

    pub fn get_version(&self) -> u64 {
        self.1
    }

}


impl From<ChunkContents> for ChunkState {

    fn from(status : ChunkContents) -> ChunkState {
        ChunkState::Generated(status.0, status.1)
    }

}

impl From<ChunkState> for Option<ChunkContents> {

    fn from(state : ChunkState) -> Option<ChunkContents> {
        match state {
            ChunkState::Generating => None,
            ChunkState::Generated(c, v) => Some(ChunkContents(c, v))
        }
    }

}

impl ChunkState {

    pub fn is_modified(&self) -> bool {
        match self {
            ChunkState::Generated(_, v) => *v != 0,
            _ => false
        }
    }

    pub fn get_version(&self) -> Option<u64> {
        match self {
            ChunkState::Generated(_, v) => Some(*v),
            ChunkState::Generating => None
        }
    }

    pub fn set(&mut self, block : BlockId, i_pos : InnerChunkPos) {
        let (arr, v) = match self {
            ChunkState::Generating => panic!("Can't spawn in chunk yet to be generated!"),
            ChunkState::Generated(ref mut arr, ref mut v) => (arr, v)
        };
        let x = i_pos[0] as usize;
        let y = i_pos[1] as usize;
        let z = i_pos[2] as usize;
        arr[x][y][z] = block;
        *v += 1;
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
