use crate::block::BlockId;
use crate::CHUNK_SIZE;
use serde_derive::{Deserialize, Serialize};

pub type BlockData = BlockId;
pub type ChunkFragment = [BlockData; CHUNK_SIZE];
pub type ChunkArray = [[ChunkFragment; CHUNK_SIZE]; CHUNK_SIZE];
pub type ChunkSidesArray = [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

mod pos;
pub use pos::*;

pub mod map;

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
