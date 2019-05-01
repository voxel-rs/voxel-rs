use std::ops::Add;
use std::ops::IndexMut;
use std::ops::Index;
use crate::block::BlockId;
use std::collections::hash_map::Entry;
use crate::CHUNK_SIZE;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

pub type BlockData = BlockId;
pub type ChunkFragment = [BlockData; CHUNK_SIZE];
pub type ChunkArray = [[ChunkFragment; CHUNK_SIZE]; CHUNK_SIZE];

pub mod region;

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
    Ungenerated,
    Generating,
    Generated(Box<ChunkArray>),
    Modified(Box<ChunkArray>, u64)
}

impl Default for ChunkState {
    fn default() -> ChunkState {
        ChunkState::Ungenerated
    }
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
            ChunkState::Generating | ChunkState::Ungenerated => None,
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

    pub fn set(&mut self, block : BlockId, i_pos : InnerChunkPos) {
        let (arr, v) = match self {
            ChunkState::Generating | ChunkState::Ungenerated
            => panic!("Can't spawn in chunk yet to be generated!"),
            ChunkState::Generated(ref mut arr) => (arr, 0),
            ChunkState::Modified(ref mut arr, v) => (arr, *v)
        };
        let x = i_pos.0[0] as usize;
        let y = i_pos.0[1] as usize;
        let z = i_pos.0[2] as usize;
        arr[x][y][z] = block;
        *self = ChunkState::Modified(arr.clone(), v + 1);
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
    /*
    pub fn get_adjacent(self) -> [ChunkPos; 6] {
        let x = self.0[0];
        let y = self.0[1];
        let z = self.0[2];
        [
            ChunkPos([x + 1, y, z]),
            ChunkPos([x, y + 1, z]),
            ChunkPos([x, y, z + 1]),
            ChunkPos([x - 1, y, z]),
            ChunkPos([x, y - 1, z]),
            ChunkPos([x, y, z - 1])
        ]
    }
    */
}

impl From<[i64; 3]> for ChunkPos {
    fn from(pos : [i64; 3]) -> ChunkPos {
        ChunkPos(pos)
    }
}

impl Index<usize> for ChunkPos {
    type Output = i64;

    fn index(&self, idx : usize) -> &i64 {
        let ChunkPos(arr) = self;
        &arr[idx]
    }
}

impl IndexMut<usize> for ChunkPos {
    fn index_mut(&mut self, idx : usize) -> &mut i64 {
        let ChunkPos(arr) = self;
        &mut arr[idx]
    }
}


impl Add for ChunkPos {
    type Output = ChunkPos;

    fn add(self, other : ChunkPos) -> ChunkPos {
        [
            self[0] + other[0],
            self[1] + other[1],
            self[2] + other[2]
        ].into()
    }
}


// TODO: Struct instead ?
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ChunkPos(pub [i64; 3]);

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FragmentPos(pub [usize; 2]);
