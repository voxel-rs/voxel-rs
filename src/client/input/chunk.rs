// The client-side chunk data structures
use crate::client::input::BufferHandle3D;
use crate::sim::chunk::{ChunkArray, ChunkSidesArray};
use crate::block::{Block, BlockId, BlockRegistry};
use crate::util::{Faces, Face};
use crate::CHUNK_SIZE;
use crate::Vertex;

/// Indicates what non-void ```ChunkFragment```s a Chunk contains.
/// It is stored as 32-bit integers so that common functions are implemented.
pub type ChunkInfo = [u32; CHUNK_SIZE * CHUNK_SIZE / 32];

/// Chunk type
#[derive(Clone, Debug)]
pub struct Chunk {
    /// Blocks in the chunk
    pub blocks: Box<ChunkArray>,
    /// Empty blocks adjacent to the chunk (1 is for non-opaque, 0 is for opaque)
    pub sides: Box<ChunkSidesArray>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            blocks: Box::new([[[BlockId(0); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]),
            sides: Box::new([[[Faces::new(); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]),
        }
    }

    pub fn calculate_mesh(&self, blocks: &BlockRegistry) -> Vec<Vertex> {
        let mut vec: Vec<Vertex> = Vec::new();
        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for k in 0..CHUNK_SIZE {
                    // Don't render hidden blocks
                    if self.sides[i][j][k] != Faces::all() {
                        blocks.get_block(self.blocks[i][j][k]).render(
                            &mut vec,
                            self.sides[i][j][k],
                            [i as u64, j as u64, k as u64],
                        );
                    }
                }
            }
        }
        vec
    }
}


/// Chunk information stored by the client
pub(super) struct ChunkData {
    /// The chunk data itself
    pub chunk: Chunk,
    /// How many fragments are in the chunk
    pub fragments: usize,
    /// Latest fragment version
    pub latest : u64,
    /// How many fragments have been received for the LATEST version
    pub latest_fragments : usize,
    /// Current fragment version
    pub current : u64,
    /// What adjacent chunks are loaded. This is a bit mask, and 1 means loaded.
    /// All chunks loaded means that adj_chunks == 0b00111111 (Faces::all())
    pub adj_chunks: Faces,
    /// The loaded bits
    pub chunk_info: ChunkInfo,
    /// The chunk's state
    pub state: ChunkState,
    /// Whether this chunk is hot, i.e. has been modified since last meshing
    pub hot : bool
}

/// A client chunk's state
pub(super) enum ChunkState {
    Unmeshed,
    Meshing,
    Meshed(BufferHandle3D),
}
