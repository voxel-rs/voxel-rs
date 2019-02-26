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
            NewChunkFragment(ChunkPos, FragmentPos, Box<ChunkFragment>),
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
            NewChunkFragment(ChunkPos, FragmentPos, Vec<u8>),
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
        use crate::block::{ChunkArray, ChunkPos};
        use crate::player::{PlayerInput, PlayerPos};

        use cobalt::ConnectionID;

        pub enum ToNetwork {
            NewChunk(ConnectionID, ChunkPos, Box<ChunkArray>),
            SetPos(ConnectionID, PlayerPos),
        }

        #[derive(Debug)]
        pub enum ToGame {
            PlayerEvent(ConnectionID, ToGamePlayer),
            NewChunk(ChunkPos, Box<ChunkArray>),
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
