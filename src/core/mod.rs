pub mod messages {
    pub mod client {
        use ::block::{ChunkArray, ChunkPos};

        pub enum ToNetwork {
            NewChunk(ChunkPos),
        }

        pub enum ToInput {
            NewChunkBuffer(ChunkPos, Vec<::Vertex>),
        }

        pub enum ToMeshing {
            AllowChunk(ChunkPos),
            NewChunk(ChunkPos, Box<ChunkArray>),
            RemoveChunk(ChunkPos),
        }
    }

    pub mod network {
        use ::block::{ChunkArray, ChunkPos};

        pub enum ToClient {
            NewChunk(ChunkPos, Box<ChunkArray>),
        }

        pub enum ToServer {
            NewChunk(ChunkPos),
        }
    }
}