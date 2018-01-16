pub mod messages {
    pub mod client {
        use ::block::{ChunkInfo, ChunkFragment, ChunkPos, FragmentPos};
        use ::player::PlayerPos;

        pub enum ToNetwork {
            SetPressedKeys(u8),
            SetRenderDistance(u64),
        }

        pub enum ToInput {
            NewChunkBuffer(ChunkPos, Vec<::Vertex>),
            SetPos(PlayerPos),
        }

        pub enum ToMeshing {
            AllowChunk(ChunkPos),
            NewChunkFragment(ChunkPos, FragmentPos, Box<ChunkFragment>),
            NewChunkInfo(ChunkPos, ChunkInfo),
            RemoveChunk(ChunkPos),
        }
    }

    pub mod network {
        use ::block::{ChunkInfo, ChunkPos, FragmentPos};
        use ::player::PlayerPos;

        #[derive(Serialize, Deserialize)]
        pub enum ToClient {
            NewChunkFragment(ChunkPos, FragmentPos, Vec<u8>),
            NewChunkInfo(ChunkPos, ChunkInfo),
            SetPos(PlayerPos),
        }

        #[derive(Serialize, Deserialize)]
        pub enum ToServer {
            SetPressedKeys(u8),
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
            SetPos(ConnectionID, PlayerPos),
        }

        #[derive(Debug)]
        pub enum ToGame {
            PlayerEvent(ConnectionID, ToGamePlayer),
        }

        #[derive(Debug)]
        pub enum ToGamePlayer {
            Connect,
            SetPressedKeys(u8),
            SetRenderDistance(u64),
            Disconnect,
        }
    }
}
