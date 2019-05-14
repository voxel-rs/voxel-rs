use super::*;

/// A server-side chunk
#[derive(Clone)]
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
