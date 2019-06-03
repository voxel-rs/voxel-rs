//! The meshing thread computes chunk meshes from `ChunkArray`s.
//! It it used to offload computation-intensive operations from the input thread.

use crate::util::Faces;
use crate::{
    block::{BlockRegistry, Block},
    core::messages::client::{ToInput, ToMeshing},
    CHUNK_SIZE,
};
use super::input::chunk::Chunk;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

const ADJ_CHUNKS: [[i64; 3]; 6] = [
    [0, 0, -1],
    [0, 0, 1],
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
];

pub fn start(
    rx: Receiver<ToMeshing>,
    input_tx: Sender<ToInput>,
    block_registry: Arc<BlockRegistry>,
) {
    let mut implementation = MeshingImpl::from_parts(rx, input_tx, block_registry);

    loop {
        implementation.process_message();

        //implementation.update_chunks();
    }
}

struct MeshingImpl {
    rx: Receiver<ToMeshing>,
    input_tx: Sender<ToInput>,
    block_registry: Arc<BlockRegistry>,
}

impl MeshingImpl {
    pub fn from_parts(
        rx: Receiver<ToMeshing>,
        input_tx: Sender<ToInput>,
        block_registry: Arc<BlockRegistry>,
    ) -> Self {
        Self {
            rx,
            input_tx,
            block_registry,
        }
    }

    fn process_message(&mut self) {
        if let Ok(message) = self.rx.recv() {
            match message {
                ToMeshing::ComputeChunkMesh(pos, mut chunk) => {
                    self.calculate_chunk_sides(&mut chunk);
                    let mesh = chunk.calculate_mesh(&self.block_registry);
                    self.input_tx
                        .send(ToInput::NewChunkBuffer(pos, mesh))
                        .unwrap();
                }
            }
        }
    }

    fn calculate_chunk_sides(&self, chunk: &mut Chunk) {
        let blocks = &chunk.blocks;
        // Update the sides using the data from the chunk
        let sz = CHUNK_SIZE as i64;
        for i in 0..sz {
            for j in 0..sz {
                for k in 0..sz {
                    for side in Faces::all().iter() {
                        let adj = ADJ_CHUNKS[side as usize];
                        let (x, y, z) = (i + adj[0], j + adj[1], k + adj[2]);
                        if 0 <= x && x < sz && 0 <= y && y < sz && 0 <= z && z < sz {
                            if !self
                                .block_registry
                                .get_block(blocks[x as usize][y as usize][z as usize])
                                .is_opaque()
                            {
                                chunk.sides[i as usize][j as usize][k as usize] |= side;
                            }
                        }
                    }
                }
            }
        }
    }
}
