//! The game thread is the main server thread. It is authoritative over the game.

use crate::sim::chunk::{ChunkContents, ChunkPos, ChunkState};
use crate::config::Config;
use crate::core::messages::server::{ToGame, ToNetwork, ToWorldgen};
use crate::network::ConnectionId;
use crate::util::Ticker;
use crate::sim::{World, player::PlayerId};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Instant;

pub fn start(
    rx: Receiver<ToGame>,
    network_tx: Sender<ToNetwork>,
    worldgen_tx: Sender<ToWorldgen>,
    config: Arc<Config>,
) {
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
    world: World,
    connections: HashMap<ConnectionId, PlayerId>,
    last_tick: Instant,
    last_update: Ticker,
}

impl GameImpl {
    pub fn from_parts(
        rx: Receiver<ToGame>,
        network_tx: Sender<ToNetwork>,
        worldgen_tx: Sender<ToWorldgen>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            config,
            rx,
            network_tx,
            worldgen_tx,
            world: World::new(),
            connections: HashMap::new(),
            last_tick: Instant::now(),
            last_update: Ticker::from_tick_rate(60),
        }
    }

    pub fn process_messages(&mut self) {
        let message = self.rx.recv().unwrap();
        self.process_message(message);
    }

    fn process_message(&mut self, message: ToGame) {
        use crate::core::messages::server::ToGamePlayer as Ev;
        match message {
            ToGame::PlayerEvent(id, ev) => match ev {
                Ev::Connect => {
                    self.connections.insert(
                        id,
                        self.world.players.new_player(
                            [
                                self.config.player_x,
                                self.config.player_y,
                                self.config.player_z,
                            ].into(),
                            true)
                    );
                }
                Ev::Disconnect => {
                    if let Some(id) = self.connections.remove(&id) {
                        self.world.players[id].active = false;
                    }
                }
                Ev::SetInput(input) => self.world.players[*self.connections.get(&id).unwrap()]
                    .set_input(&input),
                Ev::SetRenderDistance(render_distance) => {
                    self.world.players[*self.connections.get(&id).unwrap()]
                        .render_distance = render_distance
                }
            },
            ToGame::NewChunk(pos, s, _) => {
                if let Some(state) = self.world.chunks.get_mut(&pos) {
                    *state = s.into();
                }
            }
        }
    }

    //TODO: change to world ticking
    pub fn tick_game(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_tick;
        self.last_tick = now;
        let dt = dt.subsec_nanos() as f64 / 1_000_000_000.0;

        for p in self.world.players.iter_mut() {
            p.tick(dt, &self.config, &mut self.world.chunks);
        }
    }

    pub fn send_chunks(&mut self) {
        let GameImpl {
            ref mut world,
            ref mut connections,
            ref mut network_tx,
            ref mut last_update,
            ..
        } = *self;

        let World {
            ref mut chunks,
            ref mut players
        } = *world;

        // Send chunks to the players, eventually generating them
        for (id, player) in connections.iter() {
            let player = &mut players[*player];
            let mut nearby = Vec::new();
            let d = player.render_distance as i64;
            let p = player.get_pos();
            // player_chunk
            let pc = p.chunk_pos();
            for x in -d..(d + 1) {
                for y in -d..(d + 1) {
                    for z in -d..(d + 1) {
                        nearby.push((x, y, z));
                    }
                }
            }
            // Sort chunks by squared distance to the player
            nearby.sort_unstable_by_key(|&(x, y, z)| x * x + y * y + z * z);
            for (x, y, z) in nearby.drain(..) {
                let mut pos = ChunkPos([x, y, z]);
                for i in 0..3 {
                    pos.0[i] += pc.0[i];
                }

                // Entry manipulation
                use std::collections::hash_map::Entry;
                let player_entry = player.chunks.entry(pos);
                if let Entry::Occupied(_) = player_entry {
                    if !chunks.is_hot(&pos) {
                        continue;
                    }
                }

                // At this point, the player doesn't have the chunk.
                match chunks.entry(pos) {
                    Entry::Vacant(v) => {
                        // Generate it
                        v.insert(ChunkState::Generating);
                        self.worldgen_tx
                            .send(ToWorldgen::GenerateChunk(pos))
                            .unwrap();
                    }
                    Entry::Occupied(o) => {
                        let contents : Option<ChunkContents> = o.get().clone().into();
                        match contents {
                            None => (),
                            Some(c) => {
                                network_tx
                                    .send(ToNetwork::NewChunk(*id, pos, c, chunks.is_hot(&pos)))
                                    .unwrap();
                                player_entry.or_insert(());
                            }
                        }
                    },
                }
            }
            // Remove chunks that are too far away
            let render_distance = player.render_distance;
            player
                .chunks
                .retain(|pos, _| pos.orthogonal_dist(pc) <= render_distance);
        }

        // Remove chunks that are far from all players

        chunks.retain(|pos, chunk| {
            if chunk.is_modified() {return true;}
            for (_, player) in connections.iter() {
                let player = &players[*player];
                let p = player.get_pos();
                if p.chunk_pos().orthogonal_dist(*pos) <= player.render_distance {
                    return true;
                }
            }
            false
        });


        // Send physics updates
        if last_update.try_tick() {
            for (id, player) in connections {
                let player = &players[*player];
                network_tx
                    .send(ToNetwork::SetPos(*id, player.get_pos().clone()))
                    .unwrap();
            }
        }

        // Cool chunks
        chunks.cool_all();
    }
}
