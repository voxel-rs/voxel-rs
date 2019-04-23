//! The worldgen threads generates chunks.
//! It it used to offload computation-intensive operations from the game thread.

use crate::block::{BlockId, ChunkArray, ChunkPos, ChunkContents};
use crate::core::messages::server::{ToGame, ToWorldgen};
use crate::CHUNK_SIZE;

use std::sync::mpsc::{Receiver, Sender};

use noise::{NoiseFn, Perlin, Seedable};
use rand::{Rng, SeedableRng};

pub fn start(rx: Receiver<ToWorldgen>, game_tx: Sender<ToGame>) {
    let mut generator = ChunkGenerator::new();
    for message in rx {
        match message {
            ToWorldgen::GenerateChunk(pos) => {
                game_tx
                    .send(ToGame::NewChunk(pos, ChunkContents::Generated(generator.generate(pos)), false))
                    .unwrap();
            }
        }
    }
}

struct ChunkGenerator {
    perlin: Perlin,
}

impl ChunkGenerator {
    pub fn new() -> Self {
        let perlin = Perlin::new();
        perlin.set_seed(42);
        ChunkGenerator { perlin }
    }

    pub fn generate(&mut self, pos: ChunkPos) -> Box<ChunkArray> {
        //println!("[Server] Game: generating chunk @ {:?}", pos);
        let (cx, cy, cz) = (pos.0[0], pos.0[1], pos.0[2]);
        let mut chunk = [[[BlockId::from(0); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        let seed = ((cx * 4242424242 + cz) % 1_000_000_007).abs();
        let mut seed_array = [0; 32];
        for i in 0..32 {
            seed_array[i] = ((seed >> (i * 8)) % 256) as u8;
        }
        let mut rng = rand::rngs::StdRng::from_seed(seed_array);
        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                let height = (150.0
                    * self.perlin.get([
                        0.005 * (0.0021 + (CHUNK_SIZE as i64 * cx + i as i64) as f64 / 3.0),
                        0.5,
                        0.005 * (0.0021 + (CHUNK_SIZE as i64 * cz + j as i64) as f64 / 3.0),
                    ])) as i64;

                for k in 0..CHUNK_SIZE {
                    let coal_noise = (100.0
                        * (1.0
                            + self.perlin.get([
                                0.1 * (CHUNK_SIZE as i64 * cx + i as i64) as f64,
                                0.1 * (CHUNK_SIZE as i64 * cy + k as i64) as f64,
                                0.1 * (CHUNK_SIZE as i64 * cz + j as i64) as f64,
                            ]))) as i64;

                    if (cy * CHUNK_SIZE as i64 + k as i64) < height {
                        // Dirt
                        chunk[i][k][j] = BlockId::from(1);
                        if (cy * CHUNK_SIZE as i64 + k as i64) < height - 5 {
                            // Stone
                            if coal_noise > 10 && coal_noise < 15 {
                                chunk[i][k][j] = BlockId::from(6);
                            } else {
                                chunk[i][k][j] = BlockId::from(5);
                            }
                        }
                    } else if (cy * CHUNK_SIZE as i64 + k as i64) == height {
                        // Grass
                        chunk[i][k][j] = BlockId::from(2);
                    }
                }

                // Caves
                for s in 0..16 {
                    let cave_noise_1 = (100.0
                        * (1.0
                            + self.perlin.get([
                                0.005 * (0.0021 + (CHUNK_SIZE as i64 * cx + i as i64) as f64),
                                50.0 * s as f64,
                                0.005 * (0.0021 + (CHUNK_SIZE as i64 * cz + j as i64) as f64),
                            ]))) as i64;
                    let cave_noise_2 = (100.0
                        * (1.0
                            + self.perlin.get([
                                0.005 * (0.0021 + (CHUNK_SIZE as i64 * cx + i as i64) as f64),
                                50.0 * s as f64 + 80.0,
                                0.005 * (0.0021 + (CHUNK_SIZE as i64 * cz + j as i64) as f64),
                            ]))) as i64;
                    let cave_noise_3 = (100.0
                        * (1.0
                            + self.perlin.get([
                                0.005 * (0.0021 + (CHUNK_SIZE as i64 * cx + i as i64) as f64),
                                50.0 * s as f64 + 80.0,
                                0.005 * (0.0021 + (CHUNK_SIZE as i64 * cz + j as i64) as f64),
                            ]))) as i64;

                    let cave_deep = -16
                        + (96.0
                            * (1.0
                                + self.perlin.get([
                                    0.005
                                        * (0.0021
                                            + (CHUNK_SIZE as i64 * cx + i as i64) as f64 / 2.0),
                                    100.0 * s as f64,
                                    0.005
                                        * (0.0021
                                            + (CHUNK_SIZE as i64 * cz + j as i64) as f64 / 2.0),
                                ]))) as i64;

                    for k in 0..CHUNK_SIZE {
                        if (cave_noise_1 > 45 && cave_noise_1 < 50)
                            || (cave_noise_2 > 45 && cave_noise_2 < 50)
                            || (cave_noise_3 > 45 && cave_noise_3 < 50)
                        {
                            let cd = cy * CHUNK_SIZE as i64 + k as i64;
                            if cd > height - cave_deep - 5 && cd <= height - cave_deep {
                                chunk[i][k][j] = BlockId::from(0); // TO DO : REPLACE WITH FILL SPHERE
                            }
                        }
                    }
                }
            }
        }
        let theta: f64 = rng.gen_range(0.0, 2.0 * 3.14);
        let r: usize = rng.gen_range(0, CHUNK_SIZE / 2 - 5);
        let (x, y) = (
            (r as f64 * theta.cos()) as i64 + CHUNK_SIZE as i64 / 2,
            (r as f64 * theta.sin()) as i64 + CHUNK_SIZE as i64 / 2,
        );
        let (x, y) = (x as usize, y as usize);
        // Spawn tree trunk
        for i in 0..7 {
            let height = (150.0
                * self.perlin.get([
                    0.005 * (0.0021 + (CHUNK_SIZE as i64 * cx + x as i64) as f64 / 3.0),
                    0.5,
                    0.005 * (0.0021 + (CHUNK_SIZE as i64 * cz + y as i64) as f64 / 3.0),
                ])) as i64;
            if cy * CHUNK_SIZE as i64 <= height + i && height + i < (cy + 1) * CHUNK_SIZE as i64 {
                chunk[x][(height - cy * CHUNK_SIZE as i64 + i) as usize][y] = BlockId::from(3);
            }
            for ii in (-3i64)..4 {
                for j in (-3i64)..4 {
                    if ii.abs() != 3 && j.abs() != 3 {
                        let mut k = 3;

                        if ii.abs() + j.abs() <= 2 {
                            k = 5
                        }
                        if ii.abs() + j.abs() <= 1 {
                            k = 6
                        }

                        for s in 0..k {
                            let xx = x as i64 + ii;
                            let yy = y as i64 + j;
                            let zz = height - cy * CHUNK_SIZE as i64 + s + 3;
                            if xx >= 0
                                && yy >= 0
                                && zz >= 0
                                && zz < CHUNK_SIZE as i64
                                && xx < CHUNK_SIZE as i64
                                && yy < CHUNK_SIZE as i64
                            {
                                let xx = xx as usize;
                                let yy = yy as usize;
                                let zz = zz as usize;
                                if chunk[xx][zz][yy] == BlockId::from(0) {
                                    chunk[xx][zz][yy] = BlockId::from(4);
                                }
                            }
                        }
                    }
                }
            }
        }

        Box::new(chunk)
    }
}
