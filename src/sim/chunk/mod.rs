use crate::block::BlockId;
use crate::CHUNK_SIZE;
use serde_derive::{Deserialize, Serialize};
use hashbrown::hash_map::HashMap;
use hashbrown::hash_map::Entry;
use hashbrown::hash_map::DefaultHashBuilder;

pub type BlockData = BlockId;
pub type ChunkFragment = [BlockData; CHUNK_SIZE];
pub type ChunkArray = [[ChunkFragment; CHUNK_SIZE]; CHUNK_SIZE];

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
    Generated(Box<ChunkArray>),
    Modified(Box<ChunkArray>, u64)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChunkContents {
    Generated(Box<ChunkArray>),
    Modified(Box<ChunkArray>, u64)
}

impl ChunkContents {

    pub fn iter(&self) -> impl Iterator<Item = &[ChunkFragment; CHUNK_SIZE]> {
        match self {
            ChunkContents::Generated(ref c) | ChunkContents::Modified(ref c, _) => c.iter()
        }
    }

    pub fn get_version(&self) -> u64 {
        match self {
            ChunkContents::Modified(_, v) => *v,
            _ => 0
        }
    }

}


impl From<ChunkContents> for ChunkState {

    fn from(status : ChunkContents) -> ChunkState {
        match status {
            ChunkContents::Generated(c) => ChunkState::Generated(c),
            ChunkContents::Modified(c, v) => ChunkState::Modified(c, v)
        }
    }

}

impl From<ChunkState> for Option<ChunkContents> {

    fn from(state : ChunkState) -> Option<ChunkContents> {
        match state {
            ChunkState::Generating => None,
            ChunkState::Generated(c) => Some(ChunkContents::Generated(c)),
            ChunkState::Modified(c, v) => Some(ChunkContents::Modified(c, v))
        }
    }

}

impl ChunkState {

    pub fn is_modified(&self) -> bool {
        match self {
            ChunkState::Modified(_, _) => true,
            _ => false
        }
    }

    pub fn get_version(&self) -> Option<u64> {
        match self {
            ChunkState::Modified(_, v) => Some(*v),
            ChunkState::Generated(_) => Some(0),
            ChunkState::Generating => None
        }
    }

    pub fn set(&mut self, block : BlockId, i_pos : InnerChunkPos) {
        let (arr, v) = match self {
            ChunkState::Generating => panic!("Can't spawn in chunk yet to be generated!"),
            ChunkState::Generated(ref mut arr) => (arr, 0),
            ChunkState::Modified(ref mut arr, v) => (arr, *v)
        };
        let x = i_pos[0] as usize;
        let y = i_pos[1] as usize;
        let z = i_pos[2] as usize;
        arr[x][y][z] = block;
        *self = ChunkState::Modified(arr.clone(), v + 1);
    }

}
