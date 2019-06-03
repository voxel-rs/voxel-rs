//! _Core_ data types. For now it only contains the different messages.

pub mod messages {
    /// Client-to-client messages.
    pub mod client {
        use crate::sim::chunk::{ChunkFragment, ChunkPos, FragmentPos, WorldPos};
        use crate::client::input::chunk::{Chunk, ChunkInfo};
        use crate::sim::player::{PlayerInput};
        use crate::Vertex;

        pub enum ToNetwork {
            SetInput(PlayerInput),
            SetRenderDistance(u64),
        }

        pub enum ToInput {
            NewChunkBuffer(ChunkPos, Vec<Vertex>),
            NewChunkFragment(ChunkPos, FragmentPos, Box<ChunkFragment>, u64),
            NewChunkInfo(ChunkPos, ChunkInfo, u64),
            SetPos(WorldPos),
        }

        pub enum ToMeshing {
            ComputeChunkMesh(ChunkPos, Chunk),
        }
    }

    /// Client-to-server and server-to-client messages.
    pub mod network {
        use crate::sim::chunk::{ChunkPos, FragmentPos, WorldPos};
        use crate::sim::player::PlayerInput;
        use crate::client::input::chunk::{ChunkInfo};
        use serde_derive::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        pub enum ToClient {
            NewChunkFragment(ChunkPos, FragmentPos, Vec<u8>, u64),
            NewChunkInfo(ChunkPos, ChunkInfo, u64),
            SetPos(WorldPos),
        }

        #[derive(Serialize, Deserialize)]
        pub enum ToServer {
            SetInput(PlayerInput),
            SetRenderDistance(u64),
        }
    }

    /// Server-to-server messages.
    pub mod server {
        use crate::sim::chunk::{ChunkContents, ChunkPos, WorldPos, Chunk};
        use crate::network::ConnectionId;
        use crate::sim::player::PlayerInput;

        pub enum ToNetwork {
            NewChunk(ConnectionId, ChunkPos, ChunkContents),
            SetPos(ConnectionId, WorldPos),
        }

        #[derive(Debug)]
        pub enum ToGame {
            PlayerEvent(ConnectionId, ToGamePlayer),
            NewChunk(ChunkPos, Chunk, bool),
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
