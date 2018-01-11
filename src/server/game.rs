extern crate cobalt;
extern crate noise;
extern crate rand;

use ::CHUNK_SIZE;
use ::block::{BlockId, ChunkArray, ChunkPos};
use ::core::messages::server::{ToGame, ToNetwork};
use ::player::Player;

use ::std::collections::HashMap;
use ::std::sync::mpsc::{Sender, Receiver};

use self::cobalt::ConnectionID;
use self::noise::{NoiseModule, Perlin, Seedable};
use self::rand::{SeedableRng, Rng};

pub fn start(rx: Receiver<ToGame>, network_tx: Sender<ToNetwork>) {
    let mut implementation = GameImpl::from_parts(rx, network_tx);
    loop {
        implementation.process_messages();

        implementation.send_chunks();
    }
}

struct GameImpl {
    rx: Receiver<ToGame>,
    network_tx: Sender<ToNetwork>,
    chunks: HashMap<ChunkPos, Box<ChunkArray>>,
    generator: ChunkGenerator,
    players: HashMap<ConnectionID, Player>,
}

impl GameImpl {
    pub fn from_parts(rx: Receiver<ToGame>, network_tx: Sender<ToNetwork>) -> Self {
        Self {
            rx,
            network_tx,
            chunks: HashMap::new(),
            generator: ChunkGenerator::new(),
            players: HashMap::new(),
        }
    }

    pub fn process_messages(&mut self) {
        let message = self.rx.recv().unwrap();
        self.process_message(message);
    }

    fn process_message(&mut self, message: ToGame) {
        use ::core::messages::server::ToGamePlayer as Ev;
        match message {
            ToGame::PlayerEvent(id, ev) => match ev {
                Ev::Connect => {
                    self.players.insert(id, Player {
                        pos: (0.0, -115.0, 0.0),
                        render_distance: 0,
                        chunks: HashMap::new(),
                    });
                },
                Ev::Disconnect => {
                    self.players.remove(&id);
                },
                Ev::SetPos(pos) => self.players.get_mut(&id).unwrap().pos = pos,
                Ev::SetRenderDistance(render_distance) => self.players.get_mut(&id).unwrap().render_distance = render_distance,
            },
        }
    }

    pub fn send_chunks(&mut self) {
        let GameImpl {
            ref mut chunks,
            ref mut players,
            ref mut generator,
            ref mut network_tx,
            ..
        } = *self;

        // Send chunks to the players, eventually generating them
        for (id, player) in players.iter_mut() {
            let d = player.render_distance as i64;
            let p = player.pos;
            let (px, py, pz) = (p.0 as i64 / CHUNK_SIZE as i64, p.1 as i64 / CHUNK_SIZE as i64, p.2 as i64 / CHUNK_SIZE as i64);
            for x in -d..(d+1) {
                for y in -d..(d+1) {
                    for z in -d..(d+1) {
                        let pos = ChunkPos(px + x, py + y, pz + z);
                        player.chunks.entry(pos.clone()).or_insert_with(|| {
                            let chunk = chunks.entry(pos.clone()).or_insert(generator.generate(&pos));
                            network_tx.send(ToNetwork::NewChunk(*id, pos, chunk.clone())).unwrap();
                            ()
                        });
                    }
                }
            }
            // Remove chunks that are too far away
            let render_distance = player.render_distance;
            player.chunks.retain(|pos, _| {
                i64::max(i64::max((pos.0 - px).abs(), (pos.1 - py).abs()), (pos.2 - pz)) <= render_distance as i64
            });
        }

        // Remove chunks that are far from all players
        chunks.retain(|pos, _| {
            for (_, player) in players.iter() {
                let p = player.pos;
                let (px, py, pz) = (p.0 as i64 / CHUNK_SIZE as i64, p.1 as i64 / CHUNK_SIZE as i64, p.2 as i64 / CHUNK_SIZE as i64);
                if i64::max(i64::max((pos.0 - px).abs(), (pos.1 - py).abs()), (pos.2 - pz).abs()) <= player.render_distance as i64 {
                    return true;
                }
            }
            false
        });
    }
}

struct ChunkGenerator {
    perlin: Perlin,
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
        println!("[Server] Game: generating chunk @ {:?}", pos);

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
                        // Dirt
                        chunk[i][k][j] = BlockId::from(1);
                        if (pos.1*CHUNK_SIZE as i64 + k as i64) < height - 5{
                            // Stone
                            chunk[i][k][j] = BlockId::from(5);
                        }
                    }
                    else if (pos.1*CHUNK_SIZE as i64 + k as i64) == height {
                        // Grass
                        chunk[i][k][j] = BlockId::from(2);
                    }
                }

                // Caves
                for s in 0..9{
                    let cave_noise_1 = (100.0*(1.0 + self.perlin.get([
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.0 + i as i64) as f64),
                        50.0*s as f64,
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.2 + j as i64) as f64)]))) as i64;
                    let cave_noise_2 = (100.0*(1.0 + self.perlin.get([
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.0 + i as i64) as f64),
                        50.0*s  as f64 + 80.0,
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.2 + j as i64) as f64)]))) as i64;

                    let cave_deep = -32 + (96.0*(1.0+self.perlin.get([
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.0 + i as i64) as f64/2.0 ),
                        100.0*s as f64,
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.2 + j as i64) as f64/2.0 )]))) as i64;

                    for k in 0..CHUNK_SIZE {
                        if (cave_noise_1 > 45 && cave_noise_1 < 50)  || (cave_noise_2 > 45 && cave_noise_2 < 50) {
                            let cd = pos.1*CHUNK_SIZE as i64 + k as i64;
                            if cd > height - cave_deep - 5 && cd <= height - cave_deep {
                                chunk[i][k][j] = BlockId::from(0); // TO DO : REPLACE WITH FILL SPHERE
                            }
                        }
                    }
                }
            }
        }
        let theta: f64 = rng.gen_range(0.0, 2.0*3.14);
        let r: usize = rng.gen_range(0, CHUNK_SIZE/2 - 5);
        let (x, y) = ((r as f64*theta.cos()) as i64 + CHUNK_SIZE as i64/2, (r as f64*theta.sin()) as i64 + CHUNK_SIZE as i64/2);
        let (x, y) = (x as usize, y as usize);
        // Spawn tree trunk
        for i in 0..7 {
            let height = (150.0*self.perlin.get([
                    0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.0 + x as i64) as f64/3.0),
                    0.5,
                    0.005*(0.0021 + (CHUNK_SIZE as i64 * pos.2 + y as i64) as f64/3.0)])) as i64;
            if pos.1*CHUNK_SIZE as i64 <= height + i && height+i < (pos.1+1)*CHUNK_SIZE as i64 {
                chunk[x][(height - pos.1*CHUNK_SIZE as i64 + i) as usize][y] = BlockId::from(3);
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
                            let zz = height - pos.1*CHUNK_SIZE as i64 + s + 3;
                            if xx >= 0 && yy >= 0 && zz >= 0 && zz < CHUNK_SIZE as i64 && xx < CHUNK_SIZE as i64 && yy < CHUNK_SIZE as i64 {
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
