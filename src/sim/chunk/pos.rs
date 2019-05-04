use crate::CHUNK_SIZE;

use std::ops::IndexMut;
use std::ops::Index;
use derive_more::{
    Add, Sub, Rem, Div, Mul, Shr, Shl, Index, IndexMut,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, ShrAssign, ShlAssign, From
};
use serde_derive::{Deserialize, Serialize};
use num::Integer;
use nalgebra::Vector3;

pub trait SubIndex<T> {
    type Remainder;
    fn high(&self) -> T;
    fn low(&self) -> Self::Remainder;
    fn factor(&self) -> (T, Self::Remainder) {
        (self.high(), self.low())
    }
}

#[derive(
    PartialEq, Clone, Copy, Debug, From,
    Add, Sub, Mul, Rem, Div, Index, IndexMut,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign
)]
pub struct WorldPos(Vector3<f64>);

impl SubIndex<BlockPos> for WorldPos {
    type Remainder = InnerBlockPos;

    fn high(&self) -> BlockPos {
        [self[0] as i64, self[1] as i64, self[2] as i64].into()
    }

    fn low(&self) -> InnerBlockPos {
        let block = self.high();
        let inner : Vector3<f64> = [
                self[0] - (block[0] as f64),
                self[1] - (block[1] as f64),
                self[2] - (block[2] as f64)
        ].into();
        inner.into()
    }

}

#[derive(
    Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize,
    Add, Sub, Mul, Rem, Div, Shr, Shl,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, ShrAssign, ShlAssign
)]
pub struct BlockPos(pub i64, pub i64, pub i64);

impl From<[i64; 3]> for BlockPos {
    fn from(pos : [i64; 3]) -> BlockPos {
        BlockPos(pos[0], pos[1], pos[2])
    }
}

impl Index<usize> for BlockPos {
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

impl IndexMut<usize> for BlockPos {
    fn index_mut(&mut self, idx : usize) -> &mut i64 {
        match idx {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("Index out of bounds!")
        }
    }
}

impl SubIndex<ChunkPos> for BlockPos {
    type Remainder = InnerChunkPos;

    fn high(&self) -> ChunkPos {
        [
            self.0.div_floor(&(CHUNK_SIZE as i64)),
            self.1.div_floor(&(CHUNK_SIZE as i64)),
            self.2.div_floor(&(CHUNK_SIZE as i64))
        ].into()
    }

    fn low(&self) -> InnerChunkPos {
        [
            (self.0 as u8) % (CHUNK_SIZE as u8),
            (self.1 as u8) % (CHUNK_SIZE as u8),
            (self.2 as u8) % (CHUNK_SIZE as u8)
        ].into()
    }

}

#[derive(
    PartialEq, Clone, Copy, Debug, From,
    Add, Sub, Mul, Rem, Div, Index, IndexMut,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign
)]
pub struct InnerBlockPos(Vector3<f64>);

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

impl From<[u8; 3]> for InnerChunkPos {
    fn from(pos : [u8; 3]) -> InnerChunkPos {
        InnerChunkPos(pos[0], pos[1], pos[2])
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
