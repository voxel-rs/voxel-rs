pub mod messages {
    pub mod client {
        use ::block::{Chunk, ChunkInfo, ChunkFragment, ChunkPos, FragmentPos};
        use ::player::{PlayerInput, PlayerPos};

        pub enum ToNetwork {
            SetInput(PlayerInput),
            SetRenderDistance(u64),
        }

        pub enum ToInput {
            NewChunkBuffer(ChunkPos, Vec<::Vertex>),
            NewChunkFragment(ChunkPos, FragmentPos, Box<ChunkFragment>),
            NewChunkInfo(ChunkPos, ChunkInfo),
            SetPos(PlayerPos),
        }

        pub enum ToMeshing {
            ComputeChunkMesh(ChunkPos, Chunk),
        }
    }

    pub mod network {
        use ::block::{ChunkInfo, ChunkPos, FragmentPos};
        use ::player::{PlayerInput, PlayerPos};

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

    pub mod server {
        extern crate cobalt;

        use ::block::{ChunkArray, ChunkPos};
        use ::player::{PlayerInput, PlayerPos};

        use self::cobalt::ConnectionID;


        pub enum ToNetwork {
            NewChunk(ConnectionID, ChunkPos, Box<ChunkArray>),
            SetPos(ConnectionID, PlayerPos),
        }

        #[derive(Debug)]
        pub enum ToGame {
            PlayerEvent(ConnectionID, ToGamePlayer),
        }

        #[derive(Debug)]
        pub enum ToGamePlayer {
            Connect,
            SetInput(PlayerInput),
            SetRenderDistance(u64),
            Disconnect,
        }
    }
}
