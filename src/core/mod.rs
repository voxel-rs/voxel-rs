//! _Core_ data types. For now it only contains the different messages.

pub mod messages {
    /// Client-to-client messages.
    pub mod client {
        use crate::block::{Chunk, ChunkFragment, ChunkInfo, ChunkPos, FragmentPos};
        use crate::player::{PlayerInput, PlayerPos};
        use crate::Vertex;

        pub enum ToNetwork {
            SetInput(PlayerInput),
            SetRenderDistance(u64),
        }

        pub enum ToInput {
            NewChunkBuffer(ChunkPos, Vec<Vertex>),
            NewChunkFragment(ChunkPos, FragmentPos, Box<ChunkFragment>, bool),
            NewChunkInfo(ChunkPos, ChunkInfo),
            SetPos(PlayerPos),
        }

        pub enum ToMeshing {
            ComputeChunkMesh(ChunkPos, Chunk),
        }
    }

    /// Client-to-server and server-to-client messages.
    pub mod network {
        use crate::block::{ChunkInfo, ChunkPos, FragmentPos};
        use crate::player::{PlayerInput, PlayerPos};
        use serde_derive::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        pub enum ToClient {
            NewChunkFragment(ChunkPos, FragmentPos, Vec<u8>, bool),
            NewChunkInfo(ChunkPos, ChunkInfo),
            SetPos(PlayerPos),
        }

        #[derive(Serialize, Deserialize)]
        pub enum ToServer {
            SetInput(PlayerInput),
            SetRenderDistance(u64),
        }
    }

    /// Server-to-server messages.
    pub mod server {
        use crate::block::ChunkContents;
        use crate::block::ChunkPos;
        use crate::network::ConnectionId;
        use crate::player::{PlayerInput, PlayerPos};

        pub enum ToNetwork {
            NewChunk(ConnectionId, ChunkPos, ChunkContents, bool),
            SetPos(ConnectionId, PlayerPos),
        }

        #[derive(Debug)]
        pub enum ToGame {
            PlayerEvent(ConnectionId, ToGamePlayer),
            NewChunk(ChunkPos, ChunkContents, bool),
        }

        #[derive(Debug)]
        pub enum ToGamePlayer {
            Connect,
            SetInput(PlayerInput),
            SetRenderDistance(u64),
            Disconnect,
        }

        pub enum ToWorldgen {
            GenerateChunk(ChunkPos),
        }
    }
}
