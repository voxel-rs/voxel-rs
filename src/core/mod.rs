pub mod messages {
    pub mod client {
        use ::block::{ChunkInfo, ChunkFragment, ChunkPos, FragmentPos};

        pub enum ToNetwork {
            NewChunk(ChunkPos),
        }

        pub enum ToInput {
            NewChunkBuffer(ChunkPos, Vec<::Vertex>),
        }

        pub enum ToMeshing {
            AllowChunk(ChunkPos),
            NewChunkFragment(ChunkPos, FragmentPos, Box<ChunkFragment>),
            NewChunkInfo(ChunkPos, ChunkInfo),
            RemoveChunk(ChunkPos),
        }
    }

    pub mod network {
        use ::block::{ChunkInfo, ChunkFragment, ChunkPos, FragmentPos};

        #[derive(Serialize, Deserialize)]
        pub enum ToClient {
            NewChunkFragment(ChunkPos, FragmentPos, Box<ChunkFragment>),
            NewChunkInfo(ChunkPos, ChunkInfo),
        }

        #[derive(Serialize, Deserialize)]
        pub enum ToServer {
            NewChunk(ChunkPos),
        }
    }
}
