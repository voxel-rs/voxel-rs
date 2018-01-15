use std::sync::mpsc::{Sender, Receiver};
use ::core::messages::client::{ToInput, ToMeshing};
use ::block::{BlockRegistry, Chunk, ChunkPos};
use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::Arc;
use ::{CHUNK_SIZE};
use std;

enum ChunkState {
    Received(usize, Chunk),
    Meshed(Chunk),
}

type ChunkMap = HashMap<ChunkPos, RefCell<ChunkState>>;

const ADJ_CHUNKS: [[i64; 3]; 6] = [
    [ 0,  0, -1],
    [ 0,  0,  1],
    [ 1,  0,  0],
    [-1,  0,  0],
    [ 0,  1,  0],
    [ 0, -1,  0],
];

pub fn start(rx: Receiver<ToMeshing>, input_tx: Sender<ToInput>, block_registry: Arc<BlockRegistry>) {
    let mut implementation = MeshingImpl::from_parts(rx, input_tx, block_registry);

    loop {
        implementation.process_messages();

        implementation.update_chunks();
    }
}

fn get_range(x: i64, reversed: bool) -> std::ops::Range<usize> {
    match x {
        0 => 0..CHUNK_SIZE,
        1 => if reversed { (CHUNK_SIZE-1)..CHUNK_SIZE } else { 0..1 },
        -1 => if reversed { 0..1 } else { (CHUNK_SIZE-1)..CHUNK_SIZE },
        _ => panic!("Impossible value"),
    }
}

struct MeshingImpl {
    rx: Receiver<ToMeshing>,
    input_tx: Sender<ToInput>,
    block_registry: Arc<BlockRegistry>,
    chunks: ChunkMap,
}

impl MeshingImpl {
    pub fn from_parts(
        rx: Receiver<ToMeshing>,
        input_tx: Sender<ToInput>,
        block_registry: Arc<BlockRegistry>) -> Self {
        Self {
            rx,
            input_tx,
            block_registry,
            chunks: HashMap::new(),
        }
    }

    pub fn process_messages(&mut self) {
        match self.rx.recv() {
            Ok(message) => self.handle_message(message),
            Err(_) => panic!("Error in the meshing thread"),
        }

        while let Ok(message) = self.rx.try_recv() {
            self.handle_message(message);
        }
    }

    fn handle_message(&mut self, message: ToMeshing) {
        match message {
            ToMeshing::AllowChunk(pos) => {
                //println!("Meshing: allowed chunk @ {:?}", pos);
                self.chunks.insert(pos, RefCell::new(ChunkState::Received(0, Chunk {
                    blocks: Box::new([[[::block::BlockId::from(0); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]),
                    sides: Box::new([[[0xFF; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]),
                })));
            },
            // TODO: Ensure the chunk has been allowed
            ToMeshing::NewChunkFragment(pos, fpos, frag) => {
                if let Some(state) = self.chunks.get(&pos) {
                    if let ChunkState::Received(ref mut fragment_count, ref mut chunk) = *state.borrow_mut() {
                        *fragment_count += 1;
                        if *fragment_count == CHUNK_SIZE*CHUNK_SIZE {
                            //println!("Meshing: new full chunk !");
                        }
                        chunk.blocks[fpos.0][fpos.1] = (*frag).clone();
                    }
                }
            },
            // Chunk is already initialized to air, so we just need to increase the fragment count!
            ToMeshing::NewChunkInfo(pos, info) => {
                if let Some(state) = self.chunks.get(&pos) {
                    if let ChunkState::Received(ref mut fragment_count, _) = *state.borrow_mut() {
                        let mut void_fragments = 0;
                        for byte in info.iter() {
                            void_fragments += byte.count_ones() as usize;
                        }
                        *fragment_count += void_fragments;
                        if *fragment_count == CHUNK_SIZE*CHUNK_SIZE {
                            //println!("Meshing: new full chunk !");
                        }
                    }
                }
            }
            ToMeshing::RemoveChunk(pos) => {
                //println!("Meshing: removed chunk @ {:?}", pos);
                self.chunks.remove(&pos);
            },
        }
    }

    pub fn update_chunks(&mut self) {
        'all_chunks: for (pos, state) in self.chunks.iter() {
            let mut chunk_copy: Chunk;
            if let ChunkState::Received(fragment_count, ref mut chunk) = *state.borrow_mut() { // Chunk has not been meshed yet.
                if fragment_count < CHUNK_SIZE*CHUNK_SIZE {
                    // Chunk hasn't yet been fully received.
                    continue;
                }
                for &adj in &ADJ_CHUNKS {
                    match self.chunks.get(&ChunkPos(pos.0 + adj[0], pos.1 + adj[1], pos.2 + adj[2])) {
                        Some(ref cell) => {
                            if let ChunkState::Received(fragment_count, _) = *cell.borrow() {
                                if fragment_count < CHUNK_SIZE*CHUNK_SIZE {
                                    continue 'all_chunks;
                                }
                            }
                        },
                        _ => continue 'all_chunks,
                    }
                }
                // At this point, the chunk has been fully received, and the 6 adjacent chunks too.
                // => Do meshing

                self.calculate_chunk_sides(pos.clone(), chunk);
                self.input_tx.send(ToInput::NewChunkBuffer(pos.clone(), chunk.calculate_mesh(&self.block_registry))).unwrap();
                //println!("Meshing: updated chunk @ {:?}", pos);
                chunk_copy = chunk.clone();
            }
            else {
                continue;
            }

            ::std::mem::swap(&mut *state.borrow_mut(), &mut ChunkState::Meshed(chunk_copy));
        }
    }

    fn calculate_chunk_sides(&self, pos: ChunkPos, chunk: &mut Chunk) {
        let sides = &mut chunk.sides;
        let blocks = &chunk.blocks;
        // Update information on the 6 faces of the chunk using the adjacent chunks
        for side in 0..6 {
            let adj = ADJ_CHUNKS[side];
            match *self.chunks.get(&ChunkPos(pos.0 + adj[0], pos.1 + adj[1], pos.2 + adj[2])).unwrap().borrow() {
                ChunkState::Received(_, ref c) => for (i1, i2) in get_range(adj[0], false).zip(get_range(adj[0], true)) {
                    for (j1, j2) in get_range(adj[1], false).zip(get_range(adj[1], true)) {
                        for (k1, k2) in get_range(adj[2], false).zip(get_range(adj[2], true)) {
                            if !self.block_registry.get_block(c.blocks[i1][j1][k1]).is_opaque() {
                                sides[i2][j2][k2] ^= 1 << side;
                            }
                        }
                    }
                },
                ChunkState::Meshed(ref c) => for (i1, i2) in get_range(adj[0], false).zip(get_range(adj[0], true)) {
                    for (j1, j2) in get_range(adj[1], false).zip(get_range(adj[1], true)) {
                        for (k1, k2) in get_range(adj[2], false).zip(get_range(adj[2], true)) {
                            if !self.block_registry.get_block(c.blocks[i1][j1][k1]).is_opaque() {
                                sides[i2][j2][k2] ^= 1 << side;
                            }
                        }
                    }
                },
            }
        }
        // Update the sides using the data from the chunk
        let sz = CHUNK_SIZE as i64;
        for i in 0..sz {
            for j in 0..sz {
                for k in 0..sz {
                    for side in 0..6 {
                        let adj = ADJ_CHUNKS[side];
                        let (x, y, z) = (i + adj[0], j + adj[1], k + adj[2]);
                        if 0 <= x && x < sz && 0 <= y && y < sz && 0 <= z && z < sz {
                            if !self.block_registry.get_block(blocks[x as usize][y as usize][z as usize]).is_opaque() {
                                sides[i as usize][j as usize][k as usize] ^= 1 << side;
                            }
                        }
                    }
                }
            }
        }
    }
}
