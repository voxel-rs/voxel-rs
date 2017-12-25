pub mod messages {
    pub mod client {
        use ::block::{ChunkFragment, ChunkPos, FragmentPos};

        pub enum ToNetwork {
            NewChunk(ChunkPos),
        }

        pub enum ToInput {
            NewChunkBuffer(ChunkPos, Vec<::Vertex>),
        }

        pub enum ToMeshing {
            AllowChunk(ChunkPos),
            NewChunkFragment(ChunkPos, FragmentPos, Box<ChunkFragment>),
            RemoveChunk(ChunkPos),
        }
    }

    pub mod network {
        use ::block::{ChunkFragment, ChunkPos, FragmentPos};

        #[derive(Serialize, Deserialize)]
        pub enum ToClient {
            NewChunkFragment(ChunkPos, FragmentPos, Box<ChunkFragment>),
        }

        #[derive(Serialize, Deserialize)]
        pub enum ToServer {
            NewChunk(ChunkPos),
        }
    }
}
