use crate::block::BlockId;
use crate::CHUNK_SIZE;
use serde_derive::{Deserialize, Serialize};
use derive_more::From;

pub type BlockData = BlockId;
pub type ChunkFragment = [BlockData; CHUNK_SIZE];
pub type ChunkArray = [[ChunkFragment; CHUNK_SIZE]; CHUNK_SIZE];
pub type ChunkSidesArray = [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

mod pos;
pub use pos::*;

pub mod map;

#[derive(From, Debug, Serialize, Deserialize, Clone)]
pub enum ChunkState {
    Generating,
    Generated(Chunk)
}

/// A server-side chunk
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chunk {
    /// An array containing the blocks of this chunk
    blocks : Box<ChunkArray>,
    /// The version number of this chunk
    version : u64
}

impl Chunk {
    /// Iterate over the blocks in this chunk
    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &[ChunkFragment; CHUNK_SIZE]> {
        self.blocks.iter()
    }
    /// What version is this chunk (version 0 is freshly generated)
    pub fn get_version(&self) -> u64 {
        self.version
    }
    /// Was this chunk modified since it was generated
    pub fn is_modified(&self) -> bool {
        self.get_version() != 0
    }
    /// Clone this chunk's contents
    pub fn clone_contents(&self) -> ChunkContents {
        ChunkContents(self.blocks.clone(), self.version)
    }
    /// Move this chunk's contents out
    #[allow(dead_code)]
    pub fn contents(self) -> ChunkContents {
        ChunkContents(self.blocks, self.version)
    }
    /// Set the block at i_pos to block
    pub fn set(&mut self, block : BlockId, i_pos : InnerChunkPos) {
        let x = i_pos[0] as usize;
        let y = i_pos[1] as usize;
        let z = i_pos[2] as usize;
        self.blocks[x][y][z] = block;
        self.version += 1;
    }
}

impl From<ChunkContents> for Chunk {
    fn from(c : ChunkContents) -> Chunk {
        Chunk{ blocks : c.0, version : c.1 }
    }
}

/// The contents of a chunk
#[derive(From, Debug, Serialize, Deserialize, Clone)]
pub struct ChunkContents(pub Box<ChunkArray>, pub u64);

impl ChunkContents {
    /// Iterate over the blocks in this chunk
    pub fn iter(&self) -> impl Iterator<Item = &[ChunkFragment; CHUNK_SIZE]> {
        self.0.iter()
    }
    /// What version is this chunk (version 0 is freshly generated)
    pub fn get_version(&self) -> u64 {
        self.1
    }
}

impl ChunkState {
    /// If this chunk was generated, was this chunk modified since it was generated
    pub fn is_modified(&self) -> bool {
        match self {
            ChunkState::Generated(g) => g.is_modified(),
            ChunkState::Generating => false
        }
    }

    /// If this chunk has been generated, what version is this chunk (version 0 is freshly generated)
    pub fn get_version(&self) -> Option<u64> {
        match self {
            ChunkState::Generated(g) => Some(g.get_version()),
            ChunkState::Generating => None
        }
    }

    /// Set the block at i_pos to block if the chunk has been generated
    /// TODO: if the chunk has NOT yet been generated, store a "Modified" variant until it is
    pub fn set(&mut self, block : BlockId, i_pos : InnerChunkPos) {
        match self {
            ChunkState::Generating => panic!("Can't spawn in chunk yet to be generated!"),
            ChunkState::Generated(g) => g.set(block, i_pos)
        };
    }

    /// Check if this chunk has been generated
    pub fn is_generated(&self) -> bool {
        match self {
            ChunkState::Generating => false,
            ChunkState::Generated(_) => true
        }
    }

    /// Update with newly-generated chunk if not already generated. Return whether an update occured
    pub fn update_worldgen<T : Into<Chunk>>(&mut self, chunk : T) -> bool {
        if !self.is_generated() {
            *self = ChunkState::Generated(chunk.into());
            true
        } else {false}
    }

}
