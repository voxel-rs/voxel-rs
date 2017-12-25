extern crate bincode;
extern crate cobalt;
extern crate noise;
extern crate rand;

use ::block::BlockId;
use ::CHUNK_SIZE;
use ::core::messages::network::{ToClient, ToServer};

use self::cobalt::{MessageKind, PacketModifier, Server, ServerEvent, Socket, RateLimiter};
use self::noise::{NoiseModule, Perlin};
use self::rand::{SeedableRng, Rng};

pub fn start<S, R, M>(mut server: Server<S, R, M>) where
    S: Socket,
    R: RateLimiter,
    M: PacketModifier {

    let perlin = Perlin::new();
    loop {
        while let Ok(message) = server.accept_receive() {
            println!("[Server] Network: received event {:?}", message);
            match message {
                ServerEvent::Message(id, message) => match bincode::deserialize(message.as_ref()).unwrap() {
                    ToServer::NewChunk(pos) => {
                        println!("[Server] Network: processing chunk @ {:?}", pos);

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
                        let (x, y) = ((r as f64*theta.cos()) as i64 + CHUNK_SIZE as i64/2, (r as f64*theta.sin()) as i64 + CHUNK_SIZE as i64/2);
                        let (x, y) = (x as usize, y as usize);
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
                        //let mut count: u8 = 0;
                        for (x, chunkyz) in chunk.iter().enumerate() {
                            for (y, chunkz) in chunkyz.iter().enumerate() {
                                server.connection(&id).unwrap().send(MessageKind::Reliable, bincode::serialize(&ToClient::NewChunkFragment(pos.clone(), ::block::FragmentPos(x, y), Box::new(chunkz.clone())), bincode::Infinite).unwrap());
                                /*count += 1;
                                if count >= 7 {
                                    count = 0;
                                    server.send(false).unwrap();
                                }*/
                            }
                        }
                        server.send(false).unwrap();
                    },
                },
                _ => {},
            }
        }

        server.send(false).unwrap();
    }
}