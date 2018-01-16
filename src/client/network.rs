extern crate bincode;
extern crate cobalt;

use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::collections::VecDeque;
use ::core::messages::client::{ToInput, ToMeshing, ToNetwork};
use ::core::messages::network::{ToClient, ToServer};
use ::network::deserialize_fragment;
use ::util::Ticker;

use self::cobalt::{Client, ClientEvent, MessageKind, PacketModifier, Socket, RateLimiter};

pub fn start<S, R, M>(client_rx: Receiver<ToNetwork>, meshing_tx: Sender<ToMeshing>, input_tx: Sender<ToInput>, client: Client<S, R, M>) where
    S: Socket,
    R: RateLimiter,
    M: PacketModifier {

    let mut implementation = ClientImpl::from_parts(client_rx, meshing_tx, input_tx, client);

    loop {
        implementation.send_messages();

        implementation.receive_messages();

        implementation.try_tick();
    }
}

struct ClientImpl<S, R, M> where
    S: Socket,
    R: RateLimiter,
    M: PacketModifier {

    client_rx: Receiver<ToNetwork>,
    meshing_tx: Sender<ToMeshing>,
    input_tx: Sender<ToInput>,
    client: Client<S, R, M>,
    connected: bool,
    pending_messages: VecDeque<ToNetwork>,
    ticker: Ticker,
    received_messages: u64,
}

impl<S, R, M> ClientImpl<S, R, M> where
    S: Socket,
    R: RateLimiter,
    M: PacketModifier {

    pub fn from_parts(client_rx: Receiver<ToNetwork>, meshing_tx: Sender<ToMeshing>, input_tx: Sender<ToInput>, client: Client<S, R, M>) -> Self {
        let tick_rate = client.config().send_rate as u32;
        ClientImpl {
            client_rx,
            meshing_tx,
            input_tx,
            client,
            connected: false,
            pending_messages: VecDeque::new(),
            ticker: Ticker::from_tick_rate(tick_rate),
            received_messages: 0,
        }
    }

    pub fn send_messages(&mut self) {
        loop {
            match self.client_rx.try_recv() {
                Ok(message) => {
                    self.pending_messages.push_back(message);
                },
                Err(kind) => match kind {
                    // TODO: something better than panicking
                    TryRecvError::Disconnected => panic!(),
                    TryRecvError::Empty => break,
                },
            }
        }
        if self.connected {
            while let Some(message) = self.pending_messages.pop_front() {
                let (out, kind) = match message {
                    ToNetwork::SetPressedKeys(keys) => (ToServer::SetPressedKeys(keys), MessageKind::Instant),
                    ToNetwork::SetRenderDistance(render_distance) => (ToServer::SetRenderDistance(render_distance), MessageKind::Reliable),
                };
                self.client.connection().unwrap().send(kind, bincode::serialize(&out, bincode::Infinite).unwrap());
            }
        }
    }

    pub fn receive_messages(&mut self) {
        loop {
            let mut connection_lost = false;
            match self.client.receive() {
                Ok(message) => {
                    //println!("Network: received event {:?}", message);
                    match message {
                        ClientEvent::Message(bytes) => match bincode::deserialize(bytes.as_ref()).unwrap() {
                            ToClient::NewChunkFragment(pos, fpos, frag) => {
                                //println!("Network: received chunk fragment @ {:?}, {:?}", pos, fpos);
                                self.meshing_tx.send(ToMeshing::NewChunkFragment(pos, fpos, deserialize_fragment(&frag[..]))).unwrap();
                                self.received_messages += 1;
                            },
                            ToClient::NewChunkInfo(pos, info) => {
                                //println!("Received ChunkInfo @ {:?}", pos);
                                self.meshing_tx.send(ToMeshing::NewChunkInfo(pos, info)).unwrap();
                            },
                            ToClient::SetPos(pos) => {
                                self.input_tx.send(ToInput::SetPos(pos)).unwrap();
                            },
                        },
                        ClientEvent::Connection => {
                            self.connected = true;
                        },
                        ClientEvent::ConnectionFailed => {
                            connection_lost = true;
                        },
                        _ => {},
                    }
                },
                Err(kind) => match kind {
                    // TODO: something better than panicking
                    TryRecvError::Disconnected => panic!(),
                    TryRecvError::Empty => break,
                },
            }
            if connection_lost {
                self.client.disconnect().expect("Failed to disconnect client.");
                self.client.connect("127.0.0.1:1106").expect("Failed to bind to socket.");
                println!("Reconnecting to server...");
            }
            if self.received_messages >= 1024 {
                //println!("Network: received {} messages", self.received_messages);
                self.received_messages = 0;
            }
        }
    }

    /// Ticks the client if it is time to
    // TODO: Merge this with ServerImpl's equivalent
    pub fn try_tick(&mut self) {
        if self.ticker.try_tick() {
            self.client.send(false).unwrap();
        }
    }
}
