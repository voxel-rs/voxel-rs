extern crate bincode;
extern crate cobalt;
extern crate noise;
extern crate rand;

use ::block::BlockId;
use ::CHUNK_SIZE;
use ::core::messages::network::{ToClient, ToServer};
use ::util::Ticker;
use ::std::collections::{HashMap, VecDeque};
use ::std::time::{Duration, Instant};

use self::cobalt::{ConnectionID, MessageKind, PacketModifier, Server, ServerEvent, Socket, RateLimiter};
use self::noise::{NoiseModule, Perlin, Seedable};
use self::rand::{SeedableRng, Rng};

pub fn start<S, R, M>(server: Server<S, R, M>) where
    S: Socket,
    R: RateLimiter,
    M: PacketModifier {

    let mut implementation = ServerImpl::from_server(server);
    
    loop {
        implementation.receive_messages();

        implementation.process_messages();

        implementation.try_tick();
    }
}

struct ServerImpl<S, R, M> where
    S: Socket,
    R: RateLimiter,
    M: PacketModifier {

    server: Server<S, R, M>,
    queues: HashMap<ConnectionID, (Instant, VecDeque<ToServer>)>,
    ticker: Ticker,
    chunk_generator: ChunkGenerator,
}

struct ChunkGenerator {
    perlin: Perlin,
}

impl<S, R, M> ServerImpl<S, R, M> where
    S: Socket,
    R: RateLimiter,
    M: PacketModifier {
    
    pub fn from_server(server: Server<S, R, M>) -> Self {
        let tick_rate = server.config().send_rate as u32;
        ServerImpl {
            server,
            queues: HashMap::new(),
            ticker: Ticker::from_tick_rate(tick_rate),
            chunk_generator: ChunkGenerator::new(),
        }
    }

    pub fn receive_messages(&mut self) {
        while let Ok(message) = self.server.accept_receive() {
            println!("[Server] Network: received event {:?}", message);
            match message {
                ServerEvent::Message(id, data) => {
                    let message = bincode::deserialize(data.as_ref()).unwrap();
                    self.queues.entry(id).or_insert((Instant::now(), VecDeque::new())).1.push_back(message);
                },
                // TODO: Use other events
                _ => {},
            }
        }
    }

    pub fn process_messages(&mut self) {
        for (id, &mut(ref mut last_message, ref mut queue)) in self.queues.iter_mut() {
            if Instant::now() - *last_message > Duration::new(0, 100_000_000) && queue.len() > 0 { // Any queued messages ?
                let connection = self.server.connection(&id);
                if let Ok(connection) = connection { // Open connection ?
                    if !connection.congested() { // Not congested ?
                        // Reply to 1 message
                        match queue.pop_front().unwrap() {
                            ToServer::NewChunk(pos) => {
                                println!("[Server] Network: processing chunk @ {:?}", pos);

                                let chunk = self.chunk_generator.generate(&pos);
                                let mut info = [0; CHUNK_SIZE * CHUNK_SIZE / 32];
                                for (cx, chunkyz) in chunk.iter().enumerate() {
                                    'yiter: for (cy, chunkz) in chunkyz.iter().enumerate() {
                                        for block in chunkz.iter() {
                                            if block.0 != 0 { // Only send the message if the ChunkFragment is not empty.
                                                connection.send(MessageKind::Reliable, bincode::serialize(&ToClient::NewChunkFragment(pos.clone(), ::block::FragmentPos(cx, cy), Box::new(chunkz.clone())), bincode::Infinite).unwrap());
                                                continue 'yiter;
                                            }
                                        }
                                        // The ChunkFragment is empty
                                        let index = cx * CHUNK_SIZE + cy;
                                        info[index/32] |= 1 << index%32;
                                    }
                                }
                                connection.send(MessageKind::Reliable, bincode::serialize(&ToClient::NewChunkInfo(pos, info), bincode::Infinite).unwrap());
                            },
                        }
                        *last_message = Instant::now();
                    }
                }
            }
        }
    }

    /// Ticks the server if it is time to
    // TODO: Merge this with ClientImpl's equivalent
    pub fn try_tick(&mut self) {
        if self.ticker.try_tick() {
            self.server.send(false).unwrap();
        }
    }
}

impl ChunkGenerator {
    pub fn new() -> Self {
        let perlin = Perlin::new();
        perlin.set_seed(42);
        ChunkGenerator {
            perlin,
        }
    }

    pub fn generate(&mut self, pos: &::block::ChunkPos) -> Box<::block::ChunkArray> {
        println!("[Server] Network: processing chunk @ {:?}", pos);

        let mut chunk = [[[BlockId::from(0); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        let mut rng = rand::StdRng::from_seed(&[((pos.0*4242424242 + pos.2)%1_000_000_007).abs() as usize]);
        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                let height = (150.0*self.perlin.get([
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
            let height = (150.0*self.perlin.get([
                    0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.0 + x as i64) as f64/3.0),
                    0.5,
                    0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.2 + y as i64) as f64/3.0)])) as i64;
            if pos.1*CHUNK_SIZE as i64 <= height + i && height+i < (pos.1+1)*CHUNK_SIZE as i64 {
                chunk[x][(height - pos.1*CHUNK_SIZE as i64 + i) as usize][y] = BlockId::from(3);
            }
        }
        Box::new(chunk)
    }
}
