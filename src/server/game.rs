extern crate cgmath;
extern crate cobalt;
extern crate noise;
extern crate rand;

use ::CHUNK_SIZE;
use ::block::{BlockId, ChunkArray, ChunkPos};
use ::config::Config;
use ::core::messages::server::{ToGame, ToNetwork};
use ::player::Player;
use ::util::Ticker;

use ::std::collections::HashMap;
use ::std::sync::Arc;
use ::std::sync::mpsc::{Sender, Receiver};
use ::std::time::Instant;

use self::cgmath::Deg;
use self::cobalt::ConnectionID;
use self::noise::{NoiseModule, Perlin, Seedable};
use self::rand::{SeedableRng, Rng};

pub fn start(rx: Receiver<ToGame>, network_tx: Sender<ToNetwork>, config: Arc<Config>) {
    let mut implementation = GameImpl::from_parts(rx, network_tx, config);
    loop {
        implementation.process_messages();

        implementation.tick_game();

        implementation.send_chunks();
    }
}

struct GameImpl {
    config: Arc<Config>,
    rx: Receiver<ToGame>,
    network_tx: Sender<ToNetwork>,
    chunks: HashMap<ChunkPos, Box<ChunkArray>>,
    generator: ChunkGenerator,
    players: HashMap<ConnectionID, Player>,
    last_tick: Instant,
    last_update: Ticker,
}

impl GameImpl {
    pub fn from_parts(rx: Receiver<ToGame>, network_tx: Sender<ToNetwork>, config: Arc<Config>) -> Self {
        Self {
            config,
            rx,
            network_tx,
            chunks: HashMap::new(),
            generator: ChunkGenerator::new(),
            players: HashMap::new(),
            last_tick: Instant::now(),
            last_update: Ticker::from_tick_rate(60),
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
                        pos: [self.config.player_x, self.config.player_y, self.config.player_z].into(),
                        yaw: Deg(0.0),
                        pitch: Deg(0.0),
                        render_distance: 0,
                        chunks: HashMap::new(),
                        keys: 0,
                    });
                },
                Ev::Disconnect => {
                    self.players.remove(&id);
                },
                Ev::SetInput(input) => self.players.get_mut(&id).unwrap().set_input(&input),
                Ev::SetRenderDistance(render_distance) => self.players.get_mut(&id).unwrap().render_distance = render_distance,
            },
        }
    }

    pub fn tick_game(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_tick;
        self.last_tick = now;
        let dt = dt.subsec_nanos() as f64 / 1_000_000_000.0;

        for (_, p) in &mut self.players {
            p.tick(dt, &self.config);
        }
    }

    pub fn send_chunks(&mut self) {
        let GameImpl {
            ref mut chunks,
            ref mut players,
            ref mut generator,
            ref mut network_tx,
            ref mut last_update,
            ..
        } = *self;

        // Send chunks to the players, eventually generating them
        for (id, player) in players.iter_mut() {
            let mut nearby = Vec::new();
            let d = player.render_distance as i64;
            let p = player.get_pos();
            // player_chunk
            let pc = p.chunk_pos();
            for x in -d..(d+1) {
                for y in -d..(d+1) {
                    for z in -d..(d+1) {
                        nearby.push((x, y, z));
                    }
                }
            }
            // Sort chunks by squared distance to the player
            nearby.sort_unstable_by_key(|&(x, y, z)| x*x + y*y + z*z);
            for (x, y, z) in nearby.drain(..) {
                let mut pos = ChunkPos([x, y, z]);
                for i in 0..3 {
                    pos.0[i] += pc.0[i];
                }
                player.chunks.entry(pos.clone()).or_insert_with(|| {
                    let chunk = chunks.entry(pos.clone()).or_insert(generator.generate(&pos));
                    network_tx.send(ToNetwork::NewChunk(*id, pos, chunk.clone())).unwrap();
                    ()
                });
            }
            // Remove chunks that are too far away
            let render_distance = player.render_distance;
            player.chunks.retain(|pos, _| {
                pos.orthogonal_dist(pc) <= render_distance
            });
        }

        // Remove chunks that are far from all players
        chunks.retain(|pos, _| {
            for (_, player) in players.iter() {
                let p = player.get_pos();
                if p.chunk_pos().orthogonal_dist(*pos) <= player.render_distance {
                    return true;
                }
            }
            false
        });

        // Send physics updates
        if last_update.try_tick() {
            for (id, player) in players {
                network_tx.send(ToNetwork::SetPos(*id, player.get_pos().clone())).unwrap();
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
        ChunkGenerator {
            perlin,
        }
    }

    pub fn generate(&mut self, pos: &::block::ChunkPos) -> Box<::block::ChunkArray> {
        //println!("[Server] Game: generating chunk @ {:?}", pos);
        let (cx, cy, cz) = (pos.0[0], pos.0[1], pos.0[2]);
        let mut chunk = [[[BlockId::from(0); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        let mut rng = rand::StdRng::from_seed(&[((cx*4242424242 + cz)%1_000_000_007).abs() as usize]);
        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                let height = (150.0*self.perlin.get([
                    0.005*(0.0021 + (CHUNK_SIZE as i64 * cx + i as i64) as f64/3.0),
                    0.5,
                    0.005*(0.0021 + (CHUNK_SIZE as i64 * cz + j as i64) as f64/3.0)])) as i64;

                for k in 0..CHUNK_SIZE {

                    let coal_noise = (100.0*(1.0+self.perlin.get([
                        0.1*(CHUNK_SIZE as i64 * cx + i as i64) as f64,
                        0.1*(CHUNK_SIZE as i64 * cy + k as i64) as f64,
                        0.1*(CHUNK_SIZE as i64 * cz + j as i64) as f64]))) as i64;

                    if (cy*CHUNK_SIZE as i64 + k as i64) < height {
                        // Dirt
                        chunk[i][k][j] = BlockId::from(1);
                        if (cy*CHUNK_SIZE as i64 + k as i64) < height - 5{
                            // Stone
                            if coal_noise > 10 && coal_noise < 15{
                                chunk[i][k][j] = BlockId::from(6);
                            }else{
                                chunk[i][k][j] = BlockId::from(5);
                            }
                        }
                    }
                    else if (cy*CHUNK_SIZE as i64 + k as i64) == height {
                        // Grass
                        chunk[i][k][j] = BlockId::from(2);
                    }
                }

                // Caves
                for s in 0..9 {
                    let cave_noise_1 = (100.0*(1.0 + self.perlin.get([
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * cx + i as i64) as f64),
                        50.0*s as f64,
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * cz + j as i64) as f64)]))) as i64;
                    let cave_noise_2 = (100.0*(1.0 + self.perlin.get([
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * cx + i as i64) as f64),
                        50.0*s  as f64 + 80.0,
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * cz + j as i64) as f64)]))) as i64;

                    let cave_deep = -32 + (96.0*(1.0+self.perlin.get([
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * cx + i as i64) as f64/2.0 ),
                        100.0*s as f64,
                        0.005*(0.0021 + (CHUNK_SIZE as i64 * cz + j as i64) as f64/2.0 )]))) as i64;

                    for k in 0..CHUNK_SIZE {
                        if (cave_noise_1 > 45 && cave_noise_1 < 50)  || (cave_noise_2 > 45 && cave_noise_2 < 50) {
                            let cd = cy*CHUNK_SIZE as i64 + k as i64;
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
                    0.005*(0.0021 + (CHUNK_SIZE as i64 * cx + x as i64) as f64/3.0),
                    0.5,
                    0.005*(0.0021 + (CHUNK_SIZE as i64 * cz + y as i64) as f64/3.0)])) as i64;
            if cy*CHUNK_SIZE as i64 <= height + i && height+i < (cy+1)*CHUNK_SIZE as i64 {
                chunk[x][(height - cy*CHUNK_SIZE as i64 + i) as usize][y] = BlockId::from(3);
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
                            let zz = height - cy*CHUNK_SIZE as i64 + s + 3;
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
