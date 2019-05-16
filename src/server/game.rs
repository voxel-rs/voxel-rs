//! The game thread is the main server thread. It is authoritative over the game.

use crate::sim::chunk::{ChunkPos, ChunkState, SubIndex};
use crate::config::Config;
use crate::core::messages::server::{ToGame, ToNetwork, ToWorldgen};
use crate::network::ConnectionId;
use crate::util::Ticker;
use crate::sim::{World, player::PlayerId};
use hashbrown::hash_map::HashMap;
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

        implementation.garbage_collect();
    }
}

struct GameImpl {
    config: Arc<Config>,
    rx: Receiver<ToGame>,
    network_tx: Sender<ToNetwork>,
    worldgen_tx: Sender<ToWorldgen>,
    world: World,
    connections: HashMap<ConnectionId, PlayerId>,
    player_chunks : HashMap<PlayerId, HashMap<ChunkPos, u64>>,
    last_tick: Instant,
    last_update: Ticker
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
            connections: HashMap::default(),
            player_chunks: HashMap::default(),
            last_tick: Instant::now(),
            last_update: Ticker::from_tick_rate(60)
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
                    let new_player = self.world.players.new_player(
                        [
                            self.config.player_x,
                            self.config.player_y,
                            self.config.player_z,
                        ].into(),
                        true);
                    self.connections.insert(id, new_player);
                    self.player_chunks.insert(new_player, HashMap::default());
                }
                Ev::Disconnect => {
                    if let Some(id) = self.connections.remove(&id) {
                        self.world.players[id].active = false;
                        self.player_chunks.remove(&id);
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
                use hashbrown::hash_map::Entry;
                match self.world.chunks.entry(pos) {
                    Entry::Occupied(mut o) => {
                        o.get_mut().update_worldgen(s);
                    },
                    Entry::Vacant(v) => {v.insert(ChunkState::Generated(s.into()));}
                }
            }
        }
    }

    pub fn tick_game(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_tick;
        self.last_tick = now;
        let dt = dt.subsec_nanos() as f64 / 1_000_000_000.0;

        self.world.tick(dt, &self.config);
    }

    pub fn send_chunks(&mut self) {
        let GameImpl {
            ref mut world,
            ref mut connections,
            ref mut network_tx,
            ref mut last_update,
            ref mut player_chunks,
            ..
        } = *self;

        let World {
            ref mut chunks,
            ref mut players,
            ..
        } = *world;

        // Send chunks to the players, eventually generating them
        for (id, player) in connections.iter() {
            let mut nearby = Vec::new();
            let render_distance = players[*player].render_distance;
            let d = render_distance as i64;
            let p = players[*player].get_pos();
            // player_chunk
            let pc : ChunkPos = p.high();
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
                let mut pos : ChunkPos = [x, y, z].into();
                for i in 0..3 {
                    pos[i] += pc[i];
                }

                // Entry manipulation
                use hashbrown::hash_map::Entry;

                let player_entry = player_chunks.get_mut(player).unwrap().entry(pos);
                let server_entry = chunks.entry(pos);

                if let Entry::Occupied(ref version) = &player_entry {
                    if let Entry::Occupied(ref entry) = &server_entry {
                        if let Some(server_version) = entry.get().get_version() {
                            if *version.get() == server_version {
                                continue;
                            }
                        }
                    }
                }

                // At this point, the player doesn't have the chunk.
                match server_entry {
                    Entry::Vacant(v) => {
                        // Generate it
                        v.insert(ChunkState::Generating);
                        self.worldgen_tx
                            .send(ToWorldgen::GenerateChunk(pos))
                            .unwrap();
                    },
                    Entry::Occupied(o) => {
                        match o.get() {
                            ChunkState::Generating => {}
                            ChunkState::Generated(c) => {
                                player_entry.or_insert(c.get_version());
                                network_tx
                                    .send(ToNetwork::NewChunk(*id, pos, c.clone_contents()))
                                    .unwrap();
                            }
                        }
                    }
                }
            }
            // Remove chunks that are too far away
            player_chunks.get_mut(player).unwrap().retain(|pos, _| pos.orthogonal_dist(pc) <= render_distance);
        }

        // Send physics updates
        if last_update.try_tick() {
            for (id, player) in connections {
                let player = &players[*player];
                network_tx
                    .send(ToNetwork::SetPos(*id, player.get_pos().clone()))
                    .unwrap();
            }
        }
    }

    pub fn garbage_collect(&mut self) {
        let GameImpl {
            ref mut world,
            ref mut connections,
            ref mut config,
            ..
        } = *self;

        //TODO: configure how often this happens
        world.physics_gc(&config);

        //TODO: put this into the chunk_gc method, configure how often this happens
        let World {
            ref mut chunks,
            ref mut players,
            ..
        } = *world;

        // Remove chunks that are far from all players

        chunks.retain(|pos, chunk| {
            if chunk.is_modified() {return true;}
            for (_, player) in connections.iter() {
                let player = &players[*player];
                let p : ChunkPos = player.get_pos().high();
                if p.orthogonal_dist(*pos) <= player.render_distance {
                    return true;
                }
            }
            false
        });

    }
}
