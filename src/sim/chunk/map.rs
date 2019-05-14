use super::{ChunkPos, InnerChunkPos, ChunkState};
use crate::block::BlockId;

use hashbrown::hash_map::HashMap;
use hashbrown::hash_map::Entry;
use hashbrown::hash_map::DefaultHashBuilder;

#[derive(Clone)]
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
