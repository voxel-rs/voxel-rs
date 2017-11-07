extern crate noise;
extern crate rand;
use self::noise::{NoiseModule, Perlin};

use std::sync::mpsc::{Sender, Receiver};
use self::rand::{SeedableRng, Rng};
use ::core::messages::client::{ToMeshing, ToNetwork};
use ::block::BlockId;
use ::CHUNK_SIZE;

pub fn start(rx: Receiver<ToNetwork>, meshing_tx: Sender<ToMeshing>) {
    let perlin = Perlin::new();
    for message in rx {
        match message {
            ToNetwork::NewChunk(pos) => {
                let mut chunk = [[[BlockId::from(0); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
                let mut rng = rand::StdRng::from_seed(&[((pos.0*4242424242 + pos.2)%1_000_000_007).abs() as usize]);
                for i in 0..CHUNK_SIZE {
                    for j in 0..CHUNK_SIZE {
                        let height = (150.0*perlin.get([
                            0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.0 + i as i64) as f64/3.0),
                            0.5,
                            0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.2 + j as i64) as f64/3.0)])) as i64;
                        for k in 0..CHUNK_SIZE {
                            if (pos.1*CHUNK_SIZE as i64 + k as i64) < height {
                                chunk[i][k][j] = BlockId::from(1);
                            }
                            else if (pos.1*CHUNK_SIZE as i64 + k as i64) == height {
                                chunk[i][k][j] = BlockId::from(2);
                            }
                        }
                    }
                }
                let theta: f64 = rng.gen_range(0.0, 2.0*3.14);
                let r: usize = rng.gen_range(0, CHUNK_SIZE/2 - 5);
                let (x, y) = ((r as f64*theta.cos()) as usize + CHUNK_SIZE/2, (r as f64*theta.sin()) as usize + CHUNK_SIZE/2);
                // Spawn tree trunk
                for i in 0..6 {
                    let height = (150.0*perlin.get([
                            0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.0 + x as i64) as f64/3.0),
                            0.5,
                            0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.2 + y as i64) as f64/3.0)])) as i64;
                    if pos.1*CHUNK_SIZE as i64 <= height + i && height+i < (pos.1+1)*CHUNK_SIZE as i64 {
                        chunk[x][(height - pos.1*CHUNK_SIZE as i64 + i) as usize][y] = BlockId::from(3);
                    }
                }

                println!("Network: processed chunk @ {:?}", pos);
                meshing_tx.send(ToMeshing::NewChunk(pos, Box::new(chunk))).unwrap();
            }
        }
    }
}