// A region, that is, a unit of (at most) 8*8*8 = 256 chunks which gets stored to and loaded from disk
// Regions are stored as arrays, so chunks can quickly be looked up inside them.

// Physics happens at the region scale. Entities belong to a region and can interact with it and with
// the 26 neighboring regions, which a region contains the indices of.
// (and hence can be at most 8*32 = 256 m in span)

// Entities larger than the size of a region are allowed, but must be composed of multiple regions
// and run on a different layer of physics simulation, with each individual region capable of interacting
// with smaller objects / other regions and the results trickling back to the larger entity.
// An example of an entity larger than a region is the game world itself, which is *actually* just a large
// region group.

//use super::ChunkState;

use std::collections::HashMap;
use std::ops::Add;
use std::ops::Index;
use std::ops::IndexMut;
use num::Integer;

use super::ChunkPos;

pub const REGION_SIZE : usize = 8;

//TODO
pub enum RegionChunkState {
    Ungenerated
}

impl Default for RegionChunkState {
    fn default() -> RegionChunkState {
        RegionChunkState::Ungenerated
    }
}

pub struct RegionMap {
    regions : Vec<Region>,
    coord_map : HashMap<RegionPos, RegionID>
}

impl RegionMap {

    pub fn new() -> RegionMap {RegionMap {
        regions : Vec::new(),
        coord_map : HashMap::new()
    }}

    pub fn new_region(&mut self, pos : RegionPos) -> RegionID {
        let id = RegionID{ idx : self.regions.len() };
        let mut reg = Region::new(pos, Some(id));
        for (i, neighbor) in pos.get_neighbors().iter().enumerate() {
            reg.neighbors[i] = self.coord_map.get(neighbor).cloned();
        }
        self.regions.push(reg);
        self.coord_map.insert(pos, id);
        id
    }

    pub fn get_id(&self, pos : RegionPos) -> Option<RegionID> {
        self.coord_map.get(&pos).cloned()
    }

    pub fn get(&self, pos : RegionPos) -> Option<&Region> {
        if let Some(id) = self.get_id(pos) {
            Some(&self[id])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, pos : RegionPos) -> Option<&mut Region> {
        if let Some(id) = self.get_id(pos) {
            Some(&mut self[id])
        } else {
            None
        }
    }

}

impl Index<RegionID> for RegionMap {
    type Output = Region;

    fn index(&self, id : RegionID) -> &Region {
        &self.regions[id.idx]
    }
}

impl IndexMut<RegionID> for RegionMap {
    fn index_mut(&mut self, id : RegionID) -> &mut Region {
        &mut self.regions[id.idx]
    }
}

impl Index<RegionPos> for RegionMap {
    type Output = Region;

    fn index(&self, pos : RegionPos) -> &Region {
        &self[*self.coord_map.get(&pos).unwrap()]
    }
}

impl IndexMut<RegionPos> for RegionMap {
    fn index_mut(&mut self, pos : RegionPos) -> &mut Region {
        let id = *self.coord_map.get(&pos).unwrap();
        &mut self[id]
    }
}

// A region's ID
#[derive(Copy, Clone, Debug)]
pub struct RegionID {
    pub(self) idx : usize
}

// Inner region position
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct InnerRegPos([usize; 3]);

// Outer region position
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct RegionPos([i64; 3]);

impl RegionPos {

    pub fn get_neighbors(self) -> [RegionPos; 26] {
        [
            self + [-1, -1, -1].into(),
            self + [-1, -1, 0].into(),
            self + [-1, -1, 1].into(),
            self + [-1, 0, -1].into(),
            self + [-1, 0, 0].into(),
            self + [-1, 0, 1].into(),
            self + [-1, 1, -1].into(),
            self + [-1, 1, 0].into(),
            self + [-1, 1, 1].into(),
            self + [0, -1, -1].into(),
            self + [0, -1, 0].into(),
            self + [0, -1, 1].into(),
            self + [0, 0, -1].into(),
            //Don't count this position as it's own neighbor: self + [0, 0, 0].into(),
            self + [0, 0, 1].into(),
            self + [0, 1, -1].into(),
            self + [0, 1, 0].into(),
            self + [0, 1, 1].into(),
            self + [1, -1, -1].into(),
            self + [1, -1, 0].into(),
            self + [1, -1, 1].into(),
            self + [1, 0, -1].into(),
            self + [1, 0, 0].into(),
            self + [1, 0, 1].into(),
            self + [1, 1, -1].into(),
            self + [1, 1, 0].into(),
            self + [1, 1, 1].into()
        ]
    }

    pub fn from_chunk(pos : ChunkPos) -> RegionPos {
        let sz = REGION_SIZE as i64;
        [pos[0].div_floor(&sz), pos[1].div_floor(&sz), pos[2].div_floor(&sz)].into()
    }

}

impl From<[i64; 3]> for RegionPos {
    fn from(pos : [i64; 3]) -> RegionPos {
        RegionPos(pos)
    }
}

impl From<[usize; 3]> for InnerRegPos {
    fn from(pos : [usize; 3]) -> InnerRegPos {
        InnerRegPos(pos)
    }
}

impl Index<usize> for RegionPos {
    type Output = i64;

    fn index(&self, idx : usize) -> &i64 {
        let RegionPos(arr) = self;
        &arr[idx]
    }
}

impl IndexMut<usize> for RegionPos {
    fn index_mut(&mut self, idx : usize) -> &mut i64 {
        let RegionPos(arr) = self;
        &mut arr[idx]
    }
}

impl InnerRegPos {

    // Euclidean mod
    fn em(pos : i64) -> usize {
        let fl = pos.mod_floor(&(REGION_SIZE as i64));
        (if fl >= 0 {
            fl
        } else {
            fl + REGION_SIZE as i64
        } as usize)
    }

    pub fn from_chunk(pos : ChunkPos) -> InnerRegPos {
        [Self::em(pos[0]), Self::em(pos[1]), Self::em(pos[2])]
        .into()
    }

}

impl Index<usize> for InnerRegPos {
    type Output = usize;

    fn index(&self, idx : usize) -> &usize {
        let InnerRegPos(arr) = self;
        &arr[idx]
    }
}

impl IndexMut<usize> for InnerRegPos {
    fn index_mut(&mut self, idx : usize) -> &mut usize {
        let InnerRegPos(arr) = self;
        &mut arr[idx]
    }
}


impl Add for RegionPos {
    type Output = RegionPos;

    fn add(self, other : RegionPos) -> RegionPos {
        [
            self[0] + other[0],
            self[1] + other[1],
            self[2] + other[2]
        ].into()
    }
}

pub struct Region {
    chunks : [[[RegionChunkState; REGION_SIZE]; REGION_SIZE]; REGION_SIZE],
    modified : u16, // Modified chunk count
    generated : u16, // Generated chunk count
    pub(self) neighbors : [Option<RegionID>; 26],
    position : RegionPos,
    id : Option<RegionID>,
    //TODO: entities
}

impl Region {

    pub fn new(pos : RegionPos, id : Option<RegionID>) -> Region {
        Region {
            chunks : Default::default(),
            modified : 0,
            generated : 0,
            neighbors : Default::default(),
            position : pos,
            id : id
        }
    }

}

impl Index<InnerRegPos> for Region {
    type Output = RegionChunkState;

    fn index(&self, pos : InnerRegPos) -> &RegionChunkState {
        &self.chunks[pos[0]][pos[1]][pos[2]]
    }
}

impl IndexMut<InnerRegPos> for Region {

    fn index_mut(&mut self, pos : InnerRegPos) -> &mut RegionChunkState {
        &mut self.chunks[pos[0]][pos[1]][pos[2]]
    }

}
