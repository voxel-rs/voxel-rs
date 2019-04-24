use super::*;

use gfx::Device;
use nalgebra::{convert, Matrix4, Vector3};

impl InputImpl {
    /// Process queued chunk messages
    pub fn process_chunk_messages(&mut self) {
        for message in self.pending_messages.drain(..) {
            match message {
                ToInput::NewChunkFragment(pos, fpos, frag, modified) => {
                    if let Some(data) = self.game_state.chunks.get(&pos) {
                        let mut data = &mut *data.borrow_mut();
                        let index = fpos.0[0] * 32 + fpos.0[1];
                        // New fragment
                        let not_loaded = data.chunk_info[index / 32] & (1 << (index % 32)) == 0;

                        // If the chunk has been modified, mark it as hot
                        data.hot |= modified;

                        if not_loaded || modified {
                            data.chunk_info[index / 32] |= 1 << (index % 32);
                            data.chunk.blocks[fpos.0[0]][fpos.0[1]] = *frag;
                            if not_loaded {data.fragments += 1;}
                            // TODO: check that the chunk is in render_distance but NOT in (render_distance; rander_distance+1]
                            // Update adjacent chunks too
                            Self::check_finalize_chunk(
                                pos,
                                data,
                                &self.game_state.chunks,
                                &self.game_registries.block_registry,
                                modified
                            );
                        }
                    }
                }
                ToInput::NewChunkInfo(pos, info) => {
                    if let Some(data) = self.game_state.chunks.get(&pos) {
                        let mut data = &mut *data.borrow_mut();
                        for (from, to) in info.iter().zip(data.chunk_info.iter_mut()) {
                            data.fragments -= to.count_ones() as usize;
                            *to |= *from;
                            data.fragments += to.count_ones() as usize;
                        }
                        // Update adjacent chunks
                        Self::check_finalize_chunk(
                            pos,
                            data,
                            &self.game_state.chunks,
                            &self.game_registries.block_registry,
                            false //TODO
                        );
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    /// Check if the given chunk has been fully received, and update the adjacent chunk's sides if so
    fn check_finalize_chunk(
        pos: ChunkPos,
        data: &mut ChunkData,
        chunks: &HashMap<ChunkPos, RefCell<ChunkData>>,
        br: &BlockRegistry,
        modified : bool
    ) {
        if data.fragments == CHUNK_SIZE * CHUNK_SIZE {
            for face in 0..6 {
                let adj = ADJ_CHUNKS[face];
                let mut pos = pos;
                for i in 0..3 {
                    pos.0[i] += adj[i];
                }
                if let Some(c) = chunks.get(&pos) {
                    let mut adj_chunk = c.borrow_mut();
                    if adj_chunk.adj_chunks & (1 << face) == 0 {
                        adj_chunk.adj_chunks |= 1 << face;
                        // We update that adjacent chunk's sides with the current chunk !
                        Self::update_side(
                            // It should be the opposite face from the adjacent chunk's POV, so we XOR 1 to flip the last bit
                            face ^ 1,
                            &data,
                            &mut adj_chunk,
                            br,
                            modified
                        );
                    }
                } else {
                    println!("Warning: LOST INFORMATION. Chunk {:?} is not loaded!", pos);
                }
            }
        }
    }

    /// Add close chunks to the HashMap, drop far chunks, mesh ready chunks
    pub fn fetch_close_chunks(&mut self) {
        let player_chunk = self.input_state.camera.get_pos().chunk_pos();

        // Fetch new close chunks
        // render_distance+2 because we need to store information about the adjacent chunks
        // even if they are not yet loaded. We use +2 instead of +1 to have a small margin,
        // just in case the packets don't arrive in the right order.
        let render_dist = self.config.render_distance + 2;
        for i in -render_dist..(render_dist + 1) {
            for j in -render_dist..(render_dist + 1) {
                for k in -render_dist..(render_dist + 1) {
                    let mut pos = ChunkPos([i, j, k]);
                    for x in 0..3 {
                        pos.0[x] += player_chunk.0[x];
                    }
                    self.game_state
                        .chunks
                        .entry(pos.clone())
                        .or_insert_with(|| {
                            RefCell::new(ChunkData {
                                chunk: Chunk::new(),
                                fragments: 0,
                                adj_chunks: 0,
                                chunk_info: [0; CHUNK_SIZE * CHUNK_SIZE / 32],
                                state: ChunkState::Unmeshed,
                                hot: false
                            })
                        });
                }
            }
        }

        // Trash far chunks
        let render_dist = (self.config.render_distance + 2) as u64;
        self.game_state
            .chunks
            .retain(|pos, _| pos.orthogonal_dist(player_chunk) <= render_dist);

        // Start meshing for new chunks
        for (pos, chunk) in self.game_state.chunks.iter() {
            let mut c = chunk.borrow_mut();
            // TODO: FRAGMENT_COUNT const
            if c.adj_chunks == 0b00111111 && c.fragments == CHUNK_SIZE * CHUNK_SIZE {
                let mut update_state = false;
                if let ChunkState::Unmeshed = c.state {
                    update_state = true;
                    self.meshing_tx
                        .send(ToMeshing::ComputeChunkMesh(*pos, c.chunk.clone()))
                        .unwrap();
                }
                if update_state {
                    c.state = ChunkState::Meshing;
                }
            }
        }
    }

    /// Helper for the chunk sides, used to process 1 value or the full chunk size depending on the adjacency (-1, 0, +1).
    /// Reversed means the internal faces of the chunk.
    fn get_range(x: i64, reversed: bool) -> std::ops::Range<usize> {
        match x {
            0 => 0..CHUNK_SIZE,
            1 => {
                if reversed {
                    (CHUNK_SIZE - 1)..CHUNK_SIZE
                } else {
                    0..1
                }
            }
            -1 => {
                if reversed {
                    0..1
                } else {
                    (CHUNK_SIZE - 1)..CHUNK_SIZE
                }
            }
            _ => panic!("Impossible value"),
        }
    }

    /// Update side [face] in the [sides] of some chunk with its adjacent chunk [c].
    fn update_side(face: usize, cd: &ChunkData, a: &mut ChunkData, br: &BlockRegistry, modified : bool) {
        let adj = ADJ_CHUNKS[face];

        let c = &cd.chunk;
        let sides = &mut a.chunk.sides;

        for (int_x, ext_x) in Self::get_range(adj[0], true).zip(Self::get_range(adj[0], false)) {
            for (int_y, ext_y) in Self::get_range(adj[1], true).zip(Self::get_range(adj[1], false))
            {
                for (int_z, ext_z) in
                    Self::get_range(adj[2], true).zip(Self::get_range(adj[2], false))
                {
                    if !br.get_block(c.blocks[ext_x][ext_y][ext_z]).is_opaque() {
                        sides[int_x][int_y][int_z] |= 1 << face;
                        a.hot |= modified; // Needs a re-rendering
                    }
                }
            }
        }
    }

    /// Draw a frame.
    pub fn render(&mut self) {
        let state = &mut self.rendering_state;

        // The transform buffer
        let mut transform = Transform {
            view_proj: convert::<Matrix4<f64>, Matrix4<f32>>(
                self.input_state.camera.get_view_projection(),
            )
            .into(),
            model: [[0.0; 4]; 4],
        };

        // The player data buffer
        let player_data = PlayerData {
            direction: convert::<Vector3<f64>, Vector3<f32>>(self.input_state.camera.get_cam_dir())
                .into(),
        };

        state
            .encoder
            .update_buffer(&state.data.player_data, &[player_data], 0)
            .unwrap();

        state.encoder.clear(&state.data.out_color, CLEAR_COLOR);
        state.encoder.clear_depth(&state.data.out_depth, 1.0);

        // Render every chunk independently
        for (pos, chunk) in self.game_state.chunks.iter_mut() {
            if let ChunkState::Meshed(ref mut buff) = chunk.borrow_mut().state {
                transform.model = Matrix4::new_translation(
                    &((CHUNK_SIZE as f32)
                        * &Vector3::<f32>::new(pos.0[0] as f32, pos.0[1] as f32, pos.0[2] as f32)),
                )
                .into();
                state
                    .encoder
                    .update_buffer(&state.data.transform, &[transform], 0)
                    .unwrap();
                // Evil swap hack
                std::mem::swap(&mut state.data.vbuf, &mut buff.0);
                state.encoder.draw(&buff.1, &state.pso, &state.data);
                std::mem::swap(&mut state.data.vbuf, &mut buff.0);
            }
        }
        state.encoder.flush(&mut state.device);

        self.input_state.window.swap_buffers().unwrap();
        state.device.cleanup();
    }
}
