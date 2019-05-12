// The client-side chunk data structures
use crate::client::input::BufferHandle3D;
use crate::CHUNK_SIZE;
use crate::sim::chunk::Chunk;

/// Indicates what non-void ```ChunkFragment```s a Chunk contains.
/// It is stored as 32-bit integers so that common functions are implemented.
pub type ChunkInfo = [u32; CHUNK_SIZE * CHUNK_SIZE / 32];



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
    /// All chunks loaded means that adj_chunks == 0b00111111
    pub adj_chunks: u8,
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
