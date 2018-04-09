//! The game thread is the main server thread. It is authoritative over the game.

extern crate cgmath;
extern crate cobalt;

use ::ICHUNK_SIZE;
use ::block::{ChunkArray, ChunkPos};
use ::config::Config;
use ::core::messages::server::{ToGame, ToNetwork, ToWorldgen};
use ::player::Player;
use ::util::Ticker;

use ::std::collections::HashMap;
use ::std::sync::Arc;
use ::std::sync::mpsc::{Sender, Receiver};
use ::std::time::Instant;

use self::cgmath::Deg;
use self::cobalt::ConnectionID;


pub fn start(rx: Receiver<ToGame>, network_tx: Sender<ToNetwork>, worldgen_tx: Sender<ToWorldgen>, config: Arc<Config>) {
    let mut implementation = GameImpl::from_parts(rx, network_tx, worldgen_tx, config);
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
    worldgen_tx: Sender<ToWorldgen>,
    chunks: HashMap<ChunkPos, ChunkState>,
    players: HashMap<ConnectionID, Player>,
    last_tick: Instant,
    last_update: Ticker,
}

enum ChunkState {
    Generating,
    Generated(Box<ChunkArray>),
}

impl GameImpl {
    pub fn from_parts(rx: Receiver<ToGame>, network_tx: Sender<ToNetwork>, worldgen_tx: Sender<ToWorldgen>, config: Arc<Config>) -> Self {
        Self {
            config,
            rx,
            network_tx,
            worldgen_tx,
            chunks: HashMap::new(),
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
                        prev_pos: [self.config.player_x, self.config.player_y, self.config.player_z].into(),
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
            ToGame::NewChunk(pos, c) => {
                if let Some(mut state) = self.chunks.get_mut(&pos) {
                    *state = ChunkState::Generated(c);
                }
            }
        }
    }

    pub fn tick_game(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_tick;
        self.last_tick = now;
        let dt = dt.subsec_nanos() as f64 / 1_000_000_000.0;

        for (_, p) in &mut self.players {
            // Try to move player, revert if there is a collision
            p.tick(dt, &self.config);
            let mut allow_movement = false;
            let pos: [f64; 3] = p.pos.into();
            // TODO: Should pos[].floor() be used?
            let mut pos = [pos[0] as i64, pos[1] as i64, pos[2] as i64];
            for x in pos.iter_mut() {
                *x = (*x % ICHUNK_SIZE + ICHUNK_SIZE) % ICHUNK_SIZE;
            }
            if let Some(state) = self.chunks.get_mut(&p.get_pos().chunk_pos()) {
                if let ChunkState::Generated(ref chunk) = *state {
                    // TODO: Use BlockRegistry!
                    if chunk[pos[0] as usize][pos[1] as usize][pos[2] as usize].0 == 0 {
                        allow_movement = true;
                    }
                }
            }
            if !allow_movement {
                p.revert();
            }
        }
    }

    pub fn send_chunks(&mut self) {
        let GameImpl {
            ref mut chunks,
            ref mut players,
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

                // Entry manipulation
                use std::collections::hash_map::Entry;
                let player_entry = player.chunks.entry(pos);
                if let Entry::Occupied(_) = player_entry {
                    continue;
                }

                // At this point, the player doesn't have the chunk.
                match chunks.entry(pos) {
                    Entry::Vacant(v) => {
                        // Generate it
                        v.insert(ChunkState::Generating);
                        self.worldgen_tx.send(ToWorldgen::GenerateChunk(pos)).unwrap();
                    },
                    Entry::Occupied(o) => match *o.get() {
                        // Wait until generated
                        ChunkState::Generating => (),
                        // Send a copy of the chunk
                        ChunkState::Generated(ref c) => {
                            network_tx.send(ToNetwork::NewChunk(*id, pos, c.clone())).unwrap();
                            player_entry.or_insert(());
                        },
                    },
                }
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
