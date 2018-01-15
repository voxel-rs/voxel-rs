extern crate bincode;
extern crate cobalt;

use ::CHUNK_SIZE;
use ::core::messages::network::{ToClient, ToServer};
use ::core::messages::server::{ToGame, ToGamePlayer, ToNetwork};
use ::network::serialize_fragment;
use ::util::Ticker;

use ::std::collections::{HashMap, VecDeque};
use ::std::sync::mpsc::{Sender, Receiver};
use ::std::time::{Duration, Instant};

use self::cobalt::{ConnectionID, MessageKind, PacketModifier, Server, ServerEvent, Socket, RateLimiter};

pub fn start<S, R, M>(rx: Receiver<ToNetwork>, game_tx: Sender<ToGame>, server: Server<S, R, M>) where
    S: Socket,
    R: RateLimiter,
    M: PacketModifier {

    let mut implementation = ServerImpl::from_parts(rx, game_tx, server);

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

    rx: Receiver<ToNetwork>,
    game_tx: Sender<ToGame>,
    server: Server<S, R, M>,
    queues: HashMap<ConnectionID, (Instant, VecDeque<ToNetwork>)>,
    ticker: Ticker,
}

impl<S, R, M> ServerImpl<S, R, M> where
    S: Socket,
    R: RateLimiter,
    M: PacketModifier {

    pub fn from_parts(rx: Receiver<ToNetwork>, game_tx: Sender<ToGame>, server: Server<S, R, M>) -> Self {
        let tick_rate = server.config().send_rate as u32;
        ServerImpl {
            rx,
            game_tx,
            server,
            queues: HashMap::new(),
            ticker: Ticker::from_tick_rate(tick_rate),
        }
    }

    pub fn receive_messages(&mut self) {
        // Network messages
        while let Ok(message) = self.server.accept_receive() {
            let message = match message {
                ServerEvent::Connection(id) => Some((id, ToGamePlayer::Connect)),
                ServerEvent::ConnectionClosed(id, _) |
                ServerEvent::ConnectionLost(id, _) => Some((id, ToGamePlayer::Disconnect)),
                ServerEvent::Message(id, data) => Some((id, match bincode::deserialize(data.as_ref()).unwrap() {
                    ToServer::SetPosition(pos) => ToGamePlayer::SetPos(pos),
                    ToServer::SetRenderDistance(render_distance) => ToGamePlayer::SetRenderDistance(render_distance),
                })),
                _ => None,
            };
            if let Some(message) = message {
                let message = ToGame::PlayerEvent(message.0, message.1);
                self.game_tx.send(message).unwrap();
            }
        }

        // Internal messages
        while let Ok(message) = self.rx.try_recv() {
            let id = match &message {
                &ToNetwork::NewChunk(id, _, _) => id,
            };
            self.queues.entry(id).or_insert((Instant::now(), VecDeque::new())).1.push_back(message);
        }
    }

    pub fn process_messages(&mut self) {
        for (id, &mut(ref mut last_message, ref mut queue)) in self.queues.iter_mut() {
            if Instant::now() - *last_message > Duration::new(0, 5_000_000) && queue.len() > 0 { // Any queued messages ?
                let connection = self.server.connection(&id);
                if let Ok(connection) = connection { // Open connection ?
                    if !connection.congested() { // Not congested ?
                        // Reply to 1 message
                        match queue.pop_front().unwrap() {
                            ToNetwork::NewChunk(_, pos, chunk) => {
                                //println!("[Server] Network: processing chunk @ {:?}", pos);

                                let mut info = [0; CHUNK_SIZE * CHUNK_SIZE / 32];
                                for (cx, chunkyz) in chunk.iter().enumerate() {
                                    'yiter: for (cy, chunkz) in chunkyz.iter().enumerate() {
                                        for block in chunkz.iter() {
                                            if block.0 != 0 { // Only send the message if the ChunkFragment is not empty.
                                                connection.send(MessageKind::Reliable, bincode::serialize(&ToClient::NewChunkFragment(pos.clone(), ::block::FragmentPos(cx, cy), serialize_fragment(&chunkz)), bincode::Infinite).unwrap());
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
            /*if should_tick {
                for _ in 0..(5 * CHUNK_SIZE * CHUNK_SIZE) {
                    self.server.send(false).unwrap();
                }
            }*/
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
