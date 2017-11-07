use std::sync::mpsc::{Sender, Receiver};
use ::core::messages::client::{ToInput, ToMeshing};
use ::block::{BlockRegistry, Chunk, ChunkPos};
use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::Arc;
use ::{CHUNK_SIZE};
use std;

enum ChunkState {
    Pending,
    Received(bool, Chunk),
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
    let mut chunks: ChunkMap = HashMap::new();

    loop {
        println!("Meshing: new loop run");

        // Receive all pending updates
        match rx.recv() {
            Ok(message) => handle_message(&mut chunks, message),
            Err(_) => return,
        }

        while let Ok(message) = rx.try_recv() {
            handle_message(&mut chunks, message);
        }

        // Update chunks
        'all_chunks: for (pos, state) in &chunks {
            if let ChunkState::Received(ref mut meshed @ false, ref mut chunk) = *state.borrow_mut() {
                for &adj in &ADJ_CHUNKS {
                    match chunks.get(&ChunkPos(pos.0 + adj[0], pos.1 + adj[1], pos.2 + adj[2])) {
                        Some(ref cell) => if let ChunkState::Pending = *cell.borrow() {
                            continue 'all_chunks;
                        },
                        _ => continue 'all_chunks,
                    }
                }

                {
                    let sides = &mut chunk.sides;
                    let blocks = &chunk.blocks;
                    for side in 0..6 {
                        let adj = ADJ_CHUNKS[side];
                        if let ChunkState::Received(_, ref c) = *chunks.get(&ChunkPos(pos.0 + adj[0], pos.1 + adj[1], pos.2 + adj[2])).unwrap().borrow_mut() {
                            for (i1, i2) in get_range(adj[0], false).zip(get_range(adj[0], true)) {
                                for (j1, j2) in get_range(adj[1], false).zip(get_range(adj[1], true)) {
                                    for (k1, k2) in get_range(adj[2], false).zip(get_range(adj[2], true)) {
                                        if !block_registry.get_block(c.blocks[i1][j1][k1]).is_opaque() {
                                            sides[i2][j2][k2] ^= 1 << side;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let sz = CHUNK_SIZE as i64;
                    for i in 0..sz {
                        for j in 0..sz {
                            for k in 0..sz {
                                for side in 0..6 {
                                    let adj = ADJ_CHUNKS[side];
                                    let (x, y, z) = (i + adj[0], j + adj[1], k + adj[2]);
                                    if 0 <= x && x < sz && 0 <= y && y < sz && 0 <= z && z < sz {
                                        if !block_registry.get_block(blocks[x as usize][y as usize][z as usize]).is_opaque() {
                                            sides[i as usize][j as usize][k as usize] ^= 1 << side;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                input_tx.send(ToInput::NewChunkBuffer(pos.clone(), chunk.calculate_mesh(&block_registry))).unwrap();
                println!("Meshing: updated chunk @ {:?}", pos);
                *meshed = true;
            }
        }
    }
}

fn handle_message(chunks: &mut ChunkMap, message: ToMeshing) {
    match message {
        ToMeshing::AllowChunk(pos) => {
            chunks.insert(pos, RefCell::new(ChunkState::Pending));
        },
        // TODO: Ensure the chunk has been allowed
        ToMeshing::NewChunk(pos, chunk) => {
            chunks.insert(pos, RefCell::new(ChunkState::Received(false, Chunk {
                blocks: chunk,
                sides: Box::new([[[0xFF; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]),
            })));
        },
        ToMeshing::RemoveChunk(pos) => {
            chunks.remove(&pos);
        },
    }
    println!("Meshing: processed message");
}

fn get_range(x: i64, reversed: bool) -> std::ops::Range<usize> {
    match x {
        0 => 0..CHUNK_SIZE,
        1 => if reversed { (CHUNK_SIZE-1)..CHUNK_SIZE } else { 0..1 },
        -1 => if reversed { 0..1 } else { (CHUNK_SIZE-1)..CHUNK_SIZE },
        _ => panic!("Impossible value"),
    }
}