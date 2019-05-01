//! The network thread manages client-server interaction.

use crate::core::messages::network::{ToClient, ToServer};
use crate::core::messages::server::{ToGame, ToGamePlayer, ToNetwork};
use crate::network::{serialize_fragment, ConnectionId, Server};
use crate::CHUNK_SIZE;

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

pub fn start(rx: Receiver<ToNetwork>, game_tx: Sender<ToGame>, server: impl Server) {
    let mut implementation = ServerImpl::from_parts(rx, game_tx, server);

    loop {
        implementation.receive_messages();

        implementation.process_messages();
    }
}

struct ServerImpl<S>
where
    S: Server,
{
    rx: Receiver<ToNetwork>,
    game_tx: Sender<ToGame>,
    server: S,
    // TODO: either use this Instant or remove it
    queues: HashMap<ConnectionId, (Instant, VecDeque<ToNetwork>)>,
}

impl<S> ServerImpl<S>
where
    S: Server,
{
    pub fn from_parts(rx: Receiver<ToNetwork>, game_tx: Sender<ToGame>, server: S) -> Self {
        ServerImpl {
            rx,
            game_tx,
            server,
            queues: HashMap::new(),
        }
    }

    pub fn receive_messages(&mut self) {
        use crate::network::ServerEvent;
        // Network messages
        while let Some(message) = self.server.next_event() {
            let message = match message {
                ServerEvent::Connection(id) => Some((id, ToGamePlayer::Connect)),
                ServerEvent::ConnectionClosed(id) => Some((id, ToGamePlayer::Disconnect)),
                ServerEvent::Message(id, data) => Some((
                    id,
                    match bincode::deserialize(data.as_ref()).unwrap() {
                        ToServer::SetInput(input) => ToGamePlayer::SetInput(input),
                        ToServer::SetRenderDistance(render_distance) => {
                            ToGamePlayer::SetRenderDistance(render_distance)
                        }
                    },
                )),
            };
            if let Some(message) = message {
                let message = ToGame::PlayerEvent(message.0, message.1);
                self.game_tx.send(message).unwrap();
            }
        }

        // Internal messages
        while let Ok(message) = self.rx.try_recv() {
            let (queue, id) = match &message {
                &ToNetwork::NewChunk(id, _, _) => {
                    // Enqueue large message for later
                    (true, id)
                }
                &ToNetwork::SetPos(id, pos) => {
                    // Instantly send the message because it is very important
                    self.server
                        .send_message(id, bincode::serialize(&ToClient::SetPos(pos)).unwrap());
                    (false, id)
                }
            };
            if queue {
                self.queues
                    .entry(id)
                    .or_insert((Instant::now(), VecDeque::new()))
                    .1
                    .push_back(message);
            }
        }
    }

    pub fn process_messages(&mut self) {
        for (id, &mut (ref mut _last_message, ref mut queue)) in self.queues.iter_mut() {
            // Any queued messages ?
            if queue.len() > 0 {
                // Reply to 1 message
                match queue.pop_front().unwrap() {
                    ToNetwork::NewChunk(_, pos, chunk) => {
                        //println!("[Server] Network: processing chunk @ {:?}", pos);

                        let mut info = [0; CHUNK_SIZE * CHUNK_SIZE / 32];
                        let version = chunk.get_version();
                        for (cx, chunkyz) in chunk.iter().enumerate() {
                            'yiter: for (cy, chunkz) in chunkyz.iter().enumerate() {
                                for block in chunkz.iter() {
                                    // Only send the message if the ChunkFragment is not empty.
                                    if block.0 != 0 {
                                        self.server.send_message(
                                            *id,
                                            bincode::serialize(&ToClient::NewChunkFragment(
                                                pos.clone(),
                                                crate::sim::chunk::FragmentPos([cx, cy]),
                                                serialize_fragment(&chunkz),
                                                chunk.get_version()
                                            ))
                                            .unwrap(),
                                        );
                                        continue 'yiter;
                                    } else {
                                        // The ChunkFragment is empty
                                        let index = cx * CHUNK_SIZE + cy;
                                        info[index / 32] |= 1 << index % 32;
                                    }
                                }
                            }
                            self.server.send_message(
                                *id,
                                bincode::serialize(&ToClient::NewChunkInfo(pos, info, version)).unwrap(),
                            );
                        }
                    }
                    ToNetwork::SetPos(_, _) => unreachable!(),
                }
            }
        }
    }
}
