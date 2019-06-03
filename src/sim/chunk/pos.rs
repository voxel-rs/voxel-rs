use crate::CHUNK_SIZE;
use crate::util::Face;

use std::ops::IndexMut;
use std::ops::Index;
use derive_more::{
    Add, Sub, Rem, Div, Mul, Shr, Shl, Index, IndexMut,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, ShrAssign, ShlAssign, From
};
use serde_derive::{Deserialize, Serialize};
use num::Integer;
use nalgebra::Vector3;
use std::cmp::{max, min};

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
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign,
    Serialize, Deserialize
)]
pub struct WorldPos(pub Vector3<f64>);

impl SubIndex<BlockPos> for WorldPos {
    type Remainder = InnerBlockPos;

    fn high(&self) -> BlockPos {
        [self[0] as i64, self[1] as i64, self[2] as i64].into()
    }

    fn low(&self) -> InnerBlockPos {
        let block : BlockPos = self.high();
        let inner : Vector3<f64> = [
                self[0] - (block[0] as f64),
                self[1] - (block[1] as f64),
                self[2] - (block[2] as f64)
        ].into();
        inner.into()
    }

}

impl SubIndex<ChunkPos> for WorldPos {
    type Remainder = InnerCoords;

    fn high(&self) -> ChunkPos {
        let mut ret : ChunkPos = [0, 0, 0].into();
        for i in 0..3 {
            ret[i] = self[i] as i64 / CHUNK_SIZE as i64
                - if (self[i] as i64 % CHUNK_SIZE as i64) < 0 {
                    1
                } else {
                    0
                };
        }
        ret
    }

    fn low(&self) -> InnerCoords {
        let bp : BlockPos = self.high();
        bp.low()
    }
}

#[derive(
    Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize, From,
    Add, Sub, Mul, Rem, Div, Shr, Shl,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, ShrAssign, ShlAssign
)]
pub struct BlockPos{
    pub x : i64, pub y : i64, pub z : i64
}

impl BlockPos {
    pub fn clamp(self) -> InnerCoords {
        InnerCoords::new(
            min(max(self.x, 0) as usize, CHUNK_SIZE - 1),
            min(max(self.y, 0) as usize, CHUNK_SIZE - 1),
            min(max(self.z, 0) as usize, CHUNK_SIZE - 1)
        ).unwrap()
    }
}

impl From<[i64; 3]> for BlockPos {
    fn from(pos : [i64; 3]) -> BlockPos {
        (pos[0], pos[1], pos[2]).into()
    }
}

impl Index<usize> for BlockPos {
    type Output = i64;

    fn index(&self, idx : usize) -> &i64 {
        match idx {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of bounds!")
        }
    }
}

impl IndexMut<usize> for BlockPos {
    fn index_mut(&mut self, idx : usize) -> &mut i64 {
        match idx {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index out of bounds!")
        }
    }
}

impl SubIndex<ChunkPos> for BlockPos {
    type Remainder = InnerCoords;

    fn high(&self) -> ChunkPos {
        [
            self.x.div_floor(&(CHUNK_SIZE as i64)),
            self.y.div_floor(&(CHUNK_SIZE as i64)),
            self.z.div_floor(&(CHUNK_SIZE as i64))
        ].into()
    }

    fn low(&self) -> InnerCoords {
        InnerCoords::new(
            (self.x as u8) % (CHUNK_SIZE as u8),
            (self.y as u8) % (CHUNK_SIZE as u8),
            (self.z as u8) % (CHUNK_SIZE as u8)
        ).unwrap()
    }

}

#[derive(
    PartialEq, Clone, Copy, Debug, From,
    Add, Sub, Mul, Rem, Div, Index, IndexMut,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign
)]
pub struct InnerBlockPos(Vector3<f64>);

#[derive(
    Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize, From,
    Add, Sub, Mul, Rem, Div, Shr, Shl,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, ShrAssign, ShlAssign
)]
pub struct ChunkPos {
    pub x : i64, pub y : i64, pub z : i64
}

impl ChunkPos {
    pub fn new(x : i64, y : i64, z : i64) -> ChunkPos {
        ChunkPos { x : x, y : y, z : z }
    }
    pub fn orthogonal_dist(self, other: ChunkPos) -> u64 {
        let mut maxcoord = 0;
        for i in 0..3 {
            maxcoord = i64::max(maxcoord, (other[i] - self[i]).abs());
        }
        maxcoord as u64
    }
    #[allow(dead_code)]
    pub fn center(self) -> Vector3<f64> {
        self.edge() + Vector3::from([16.0, 16.0, 16.0])
    }
    pub fn edge(self) -> Vector3<f64> {
        Vector3::from([self.x as f64 * 32.0, self.y as f64 * 32.0, self.z as f64 * 32.0])
    }
}


impl From<[i64; 3]> for ChunkPos {
    fn from(pos : [i64; 3]) -> ChunkPos {
        (pos[0], pos[1], pos[2]).into()
    }
}

impl Index<usize> for ChunkPos {
    type Output = i64;

    fn index(&self, idx : usize) -> &i64 {
        match idx {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of bounds!")
        }
    }
}

impl IndexMut<usize> for ChunkPos {
    fn index_mut(&mut self, idx : usize) -> &mut i64 {
        match idx {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index out of bounds!")
        }
    }
}

pub trait InnerPos {
    type AdjacentPos : InnerPos;

    fn x(&self) -> usize;
    fn y(&self) -> usize;
    fn z(&self) -> usize;
    fn to_coords(&self) -> InnerCoords {
        (self.x() as u8, self.y() as u8, self.z() as u8).into()
    }
    fn idx(&self) -> InnerIdx {
        InnerIdx(self.x() + self.y() * CHUNK_SIZE + self.z() * CHUNK_SIZE * CHUNK_SIZE)
    }
    fn adjacent(&self, face : Face) -> Option<Self::AdjacentPos>;
}

#[derive(
    Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize,
)]
pub struct InnerCoords{
    xc : u8, yc : u8, zc : u8
}

impl InnerCoords {
    pub fn new<T : Into<usize>>(x : T, y : T, z : T) -> Option<InnerCoords> {
        let xc = x.into();
        let yc = y.into();
        let zc = z.into();
        if xc >= CHUNK_SIZE || yc >= CHUNK_SIZE || zc >= CHUNK_SIZE {
            None
        } else {
            Some(InnerCoords{ xc : xc as u8, yc : yc as u8, zc : zc as u8 })
        }
    }
}

impl<T: Into<usize>> From<(T, T, T)> for InnerCoords {
    fn from(tup : (T, T, T)) -> InnerCoords {
        InnerCoords::new(tup.0.into(), tup.1.into(), tup.2.into()).unwrap()
    }
}

impl Index<usize> for InnerCoords {
    type Output = u8;

    fn index(&self, idx : usize) -> &u8 {
        match idx {
            0 => &self.xc,
            1 => &self.yc,
            2 => &self.zc,
            _ => panic!("Index out of bounds!")
        }
    }
}

impl IndexMut<usize> for InnerCoords {
    fn index_mut(&mut self, idx : usize) -> &mut u8 {
        match idx {
            0 => &mut self.xc,
            1 => &mut self.yc,
            2 => &mut self.zc,
            _ => panic!("Index out of bounds!")
        }
    }
}

impl InnerPos for InnerCoords {
    type AdjacentPos = Self;
    fn x(&self) -> usize {self.xc as usize}
    fn y(&self) -> usize {self.yc as usize}
    fn z(&self) -> usize {self.zc as usize}
    fn adjacent(&self, face : Face) -> Option<Self::AdjacentPos> {
        match face {
            Face::Back => if self.zc == 0 {None} else {
                InnerCoords::new(self.xc, self.yc, self.zc - 1)
            },
            Face::Front => InnerCoords::new(self.xc, self.yc, self.zc + 1),
            Face::Right => InnerCoords::new(self.xc + 1, self.yc, self.zc),
            Face::Left => if self.xc == 0 {None} else {
                InnerCoords::new(self.xc - 1, self.yc, self.zc)
            },
            Face::Top => InnerCoords::new(self.xc, self.yc + 1, self.zc),
            Face::Bottom => if self.yc == 0 {None} else {
                InnerCoords::new(self.xc, self.yc - 1, self.zc)
            },
        }
    }
}

#[derive(
    Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize
)]
pub struct InnerIdx(usize);

impl InnerIdx {
    pub fn indices() -> impl Iterator<Item=InnerIdx> {
        (0..(CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE)).map(|i| InnerIdx(i))
    }
    pub fn new<T: Into<usize>>(i : T) -> Option<InnerIdx> {
        let i = i.into();
        if i < CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE {
            Some(InnerIdx(i))
        } else {
            None
        }
    }
}

impl From<usize> for InnerIdx {
    fn from(i : usize) -> InnerIdx {
        InnerIdx::new(i).unwrap()
    }
}

impl Into<usize> for InnerIdx {
    fn into(self) -> usize {
        self.0
    }
}

impl InnerPos for InnerIdx {
    type AdjacentPos = InnerIdx;
    fn x(&self) -> usize {
        self.0 % CHUNK_SIZE
    }
    fn y(&self) -> usize {
        self.0 / CHUNK_SIZE
    }
    fn z(&self) -> usize {
        self.0 / (CHUNK_SIZE * CHUNK_SIZE)
    }
    fn adjacent(&self, face : Face) -> Option<Self::AdjacentPos> {
        match face {
            Face::Back => if self.0 < CHUNK_SIZE*CHUNK_SIZE {None} else {
                Some(InnerIdx(self.0 - CHUNK_SIZE*CHUNK_SIZE))
            },
            Face::Front => InnerIdx::new(self.0 + CHUNK_SIZE*CHUNK_SIZE),
            Face::Right => if self.0 == 0 {None} else {Some(InnerIdx(self.0 - 1))},
            Face::Left => InnerIdx::new(self.0 + 1),
            Face::Top => InnerIdx::new(self.0 + CHUNK_SIZE),
            Face::Bottom =>  if self.0 < CHUNK_SIZE {None} else {
                Some(InnerIdx(self.0 - CHUNK_SIZE))
            },
        }
    }
}



#[derive(
    Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize, From,
    Add, Sub, Mul, Rem, Div, Shr, Shl,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, ShrAssign, ShlAssign
)]
pub struct FragmentPos{
    pub x : usize, pub y : usize
}

impl From<[usize; 2]> for FragmentPos {
    fn from(pos : [usize; 2]) -> FragmentPos {
        (pos[0], pos[1]).into()
    }
}

impl Index<usize> for FragmentPos {
    type Output = usize;

    fn index(&self, idx : usize) -> &usize {
        match idx {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Index out of bounds!")
        }
    }
}

impl IndexMut<usize> for FragmentPos {
    fn index_mut(&mut self, idx : usize) -> &mut usize {
        match idx {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Index out of bounds!")
        }
    }
}
