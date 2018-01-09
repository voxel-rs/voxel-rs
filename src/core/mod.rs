pub mod messages {
    pub mod client {
        use ::block::{ChunkInfo, ChunkFragment, ChunkPos, FragmentPos};
        use ::player::PlayerPos;

        pub enum ToNetwork {
            SetPos(PlayerPos),
            SetRenderDistance(u64),
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
        use ::player::PlayerPos;

        #[derive(Serialize, Deserialize)]
        pub enum ToClient {
            NewChunkFragment(ChunkPos, FragmentPos, Box<ChunkFragment>),
            NewChunkInfo(ChunkPos, ChunkInfo),
        }

        #[derive(Serialize, Deserialize)]
        pub enum ToServer {
            SetPosition(PlayerPos),
            SetRenderDistance(u64),
        }
    }

    pub mod server {
        extern crate cobalt;

        use ::block::{ChunkArray, ChunkPos};
        use ::player::PlayerPos;

        use self::cobalt::ConnectionID;
        

        pub enum ToNetwork {
            NewChunk(ConnectionID, ChunkPos, Box<ChunkArray>),
        }

        pub enum ToGame {
            PlayerEvent(ConnectionID, ToGamePlayer),
        }

        pub enum ToGamePlayer {
            Connect,
            SetPos(PlayerPos),
            SetRenderDistance(u64),
            Disconnect,
        }
    }
}
