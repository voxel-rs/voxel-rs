use crate::util::Face;

use serde::{Serialize, Deserialize};
use enumset::{EnumSet, EnumSetType};
use std::ops::{Index, IndexMut};
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

impl From<Face> for SimFace {
    fn from(face : Face) -> SimFace {
        match face {
            Face::Back => SimFace::Back,
            Face::Front => SimFace::Front,
            Face::Right => SimFace::Right,
            Face::Left => SimFace::Left,
            Face::Top => SimFace::Top,
            Face::Bottom => SimFace::Bottom
        }
    }
}

type SimFaces = EnumSet<SimFace>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSimArray([[[SimFaces; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);

impl ChunkSimArray {
    /// Get an empty ChunkSimArray (where no faces of any block are simulated)
    pub fn empty() -> ChunkSimArray {
        ChunkSimArray([[[SimFaces::empty(); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE])
    }
    #[allow(dead_code)]
    /// Get an iterator over the simulated faces for each block in a chunk
    pub fn iter(&self) -> impl Iterator<Item=&SimFaces>  {
        InnerIdx::indices().map(move |i| &self[i])
    }
    /// Get an iterator over slices of this ChunkSimArray
    #[allow(dead_code)]
    pub fn slices(&self) -> impl Iterator<Item=&[[SimFaces; CHUNK_SIZE]; CHUNK_SIZE]> {
        self.0.iter()
    }
    /// Get a mutable iterator over slices of this ChunkSimArray
    #[allow(dead_code)]
    pub fn slices_mut(&mut self) -> impl Iterator<Item=&mut [[SimFaces; CHUNK_SIZE]; CHUNK_SIZE]> {
        self.0.iter_mut()
    }
    /// Set this ChunkSimArray to empty
    #[allow(dead_code)]
    pub fn blank(&mut self) {
        *self = Self::empty()
    }
}

impl<T: InnerPos> Index<T> for ChunkSimArray {
    type Output = SimFaces;

    fn index(&self, pos : T) -> &SimFaces {
        &self.0[pos.x()][pos.y()][pos.z()]
    }
}

impl<T: InnerPos> IndexMut<T> for ChunkSimArray {
    fn index_mut(&mut self, pos : T) -> &mut SimFaces {
        &mut self.0[pos.x()][pos.y()][pos.z()]
    }
}

impl Index<usize> for ChunkSimArray {
    type Output = [[SimFaces; CHUNK_SIZE]; CHUNK_SIZE];

    fn index(&self, pos : usize) -> &Self::Output {
        &self.0[pos]
    }
}

impl IndexMut<usize> for ChunkSimArray {
    fn index_mut(&mut self, pos : usize) -> &mut Self::Output {
        &mut self.0[pos]
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
    /// Iterate over the slices of blocks in this chunk
    #[allow(dead_code)]
    pub fn slices(&self) -> impl Iterator<Item = &[ChunkFragment; CHUNK_SIZE]> {
        self.blocks.iter() //TODO
    }
    /// Get access to the sides array of this chunk
    #[allow(dead_code)]
    pub fn sides(&self) -> &ChunkSimArray {
        &self.sides
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
    pub fn set<T : InnerPos>(&mut self, block : BlockData, pos : T) {
        //TODO: use block registry
        // Update physics state:
        if block == BlockId::from(0) {
            // Now all neighboring blocks are visible from the appropriate side
        } else {
            // Now all neighboring blocks are invisible from the appropriate side
        }
        self.blocks[pos.x()][pos.y()][pos.z()] = block;
        self.version += 1;
    }
    /// Get the block at i_pos
    #[allow(dead_code)]
    pub fn get<T : InnerPos>(&self, pos : T) -> &BlockData {
        &self.blocks[pos.x()][pos.y()][pos.z()]
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
