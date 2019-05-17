use serde::{Serialize, Deserialize};
use enumset::{EnumSet, EnumSetType};
use derive_more::{Index, IndexMut};
use super::*;

#[derive(Debug, EnumSetType, Serialize, Deserialize)]
pub enum SimFace {
    Back = 0,
    Front = 1,
    Right = 2,
    Left = 3,
    Top = 4,
    Bottom = 5,
    This = 6
}

type SimFaces = EnumSet<SimFace>;

#[derive(Debug, Clone, Serialize, Deserialize, Index, IndexMut)]
pub struct ChunkSimArray([[[SimFaces; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);

impl ChunkSimArray {
    pub fn empty() -> ChunkSimArray {
        ChunkSimArray([[[SimFaces::empty(); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum ChunkData {
    Full(Box<ChunkArray>),
    Empty
}

#[allow(dead_code)]
impl ChunkData {
    pub fn empty_full() -> ChunkData {
        ChunkData::Full(Box::new([[[BlockId::from(0); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]))
    }
    pub fn is_empty_variant(&self) -> bool {
        match self {
            ChunkData::Full(_) => false,
            ChunkData::Empty => true
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            ChunkData::Full(f) => {
                for slice in f.iter() {
                    for line in slice {
                        for block in line {
                            if *block != BlockId::from(0) {
                                return false;
                            }
                        }
                    }
                }
                return true;
            },
            ChunkData::Empty => true
        }
    }
    pub fn fill(&mut self) -> &mut Box<ChunkArray> {
        if self.is_empty() {
            *self = Self::empty_full();
        }
        if let ChunkData::Full(ref mut f) = self {
            return f;
        }
        unreachable!();
    }
    pub fn compress(&mut self) -> bool {
        if self.is_empty() {
            *self = ChunkData::Empty;
            true
        } else {
            false
        }
    }
}

/// A server-side chunk
#[derive(Clone)]
pub struct Chunk {
    /// An array containing the blocks of this chunk
    blocks : Box<ChunkArray>,
    sides : Box<ChunkSimArray>,
    /// The version number of this chunk
    version : u64
}

impl Chunk {
    /// Iterate over the blocks in this chunk
    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &[ChunkFragment; CHUNK_SIZE]> {
        self.blocks.iter() //TODO
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
    pub fn set(&mut self, block : BlockId, i_pos : InnerCoords) {
        let x = i_pos[0] as usize;
        let y = i_pos[1] as usize;
        let z = i_pos[2] as usize;
        self.blocks[x][y][z] = block;
        self.version += 1;
    }
    /// Check whether this chunk is empty
    pub fn is_empty(&self) -> bool {
        for slice in self.blocks.iter() {
            for line in slice {
                for block in line {
                    if *block != BlockId::from(0) {
                        return false;
                    }
                }
            }
        }
        return true;
    }
}

impl From<ChunkContents> for Chunk {
    fn from(c : ChunkContents) -> Chunk {
        Chunk{ blocks : c.0, sides : Box::new(ChunkSimArray::empty()), version : c.1 }
    }
}
