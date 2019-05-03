use crate::CHUNK_SIZE;

use std::ops::IndexMut;
use std::ops::Index;
use derive_more::{
    Add, Sub, Rem, Div, Mul, Shr, Shl,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, ShrAssign, ShlAssign
};
use serde_derive::{Deserialize, Serialize};

#[derive(
    Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize,
    Add, Sub, Mul, Rem, Div, Shr, Shl,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, ShrAssign, ShlAssign
)]
pub struct ChunkPos(pub i64, pub i64, pub i64);

impl ChunkPos {
    pub fn orthogonal_dist(self, other: ChunkPos) -> u64 {
        let mut maxcoord = 0;
        for i in 0..3 {
            maxcoord = i64::max(maxcoord, (other[i] - self[i]).abs());
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
        ChunkPos(pos[0], pos[1], pos[2])
    }
}

impl Index<usize> for ChunkPos {
    type Output = i64;

    fn index(&self, idx : usize) -> &i64 {
        match idx {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Index out of bounds!")
        }
    }
}

impl IndexMut<usize> for ChunkPos {
    fn index_mut(&mut self, idx : usize) -> &mut i64 {
        match idx {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("Index out of bounds!")
        }
    }
}

#[derive(
    Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize,
    Add, Sub, Mul, Rem, Div, Shr, Shl,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, ShrAssign, ShlAssign
)]
pub struct InnerChunkPos(pub u8, pub u8, pub u8);

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
            InnerChunkPos::coord_to_inner(coords[0]),
            InnerChunkPos::coord_to_inner(coords[1]),
            InnerChunkPos::coord_to_inner(coords[2])
        )
    }
}

impl Index<usize> for InnerChunkPos {
    type Output = u8;

    fn index(&self, idx : usize) -> &u8 {
        match idx {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Index out of bounds!")
        }
    }
}

impl IndexMut<usize> for InnerChunkPos {
    fn index_mut(&mut self, idx : usize) -> &mut u8 {
        match idx {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("Index out of bounds!")
        }
    }
}

#[derive(
    Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize,
    Add, Sub, Mul, Rem, Div, Shr, Shl,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, ShrAssign, ShlAssign
)]
pub struct FragmentPos(pub usize, pub usize);

impl Index<usize> for FragmentPos {
    type Output = usize;

    fn index(&self, idx : usize) -> &usize {
        match idx {
            0 => &self.0,
            1 => &self.1,
            _ => panic!("Index out of bounds!")
        }
    }
}

impl IndexMut<usize> for FragmentPos {
    fn index_mut(&mut self, idx : usize) -> &mut usize {
        match idx {
            0 => &mut self.0,
            1 => &mut self.1,
            _ => panic!("Index out of bounds!")
        }
    }
}
