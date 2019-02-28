//! The network thread manages client-server interaction.

use crate::core::messages::client::{ToInput, ToNetwork};
use crate::core::messages::network::{ToClient, ToServer};
use crate::network::{deserialize_fragment, Client, ClientEvent};
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

pub fn start<C>(client_rx: Receiver<ToNetwork>, input_tx: Sender<ToInput>, client: C)
where
    C: Client,
{
    let mut implementation = ClientImpl::from_parts(client_rx, input_tx, client);

    loop {
        implementation.send_messages();

        implementation.receive_messages();
    }
}

struct ClientImpl<C>
where
    C: Client,
{
    client_rx: Receiver<ToNetwork>,
    input_tx: Sender<ToInput>,
    client: C,
    pending_messages: VecDeque<ToNetwork>,
}

impl<C> ClientImpl<C>
where
    C: Client,
{
    pub fn from_parts(
        client_rx: Receiver<ToNetwork>,
        input_tx: Sender<ToInput>,
        client: C,
    ) -> Self {
        ClientImpl {
            client_rx,
            input_tx,
            client,
            pending_messages: VecDeque::new(),
        }
    }

    pub fn send_messages(&mut self) {
        loop {
            match self.client_rx.try_recv() {
                Ok(message) => {
                    self.pending_messages.push_back(message);
                }
                Err(kind) => match kind {
                    // TODO: something better than panicking
                    TryRecvError::Disconnected => panic!("Network thread disconnected"),
                    TryRecvError::Empty => break,
                },
            }
        }
        while let Some(message) = self.pending_messages.pop_front() {
            let message = match message {
                ToNetwork::SetInput(input) => ToServer::SetInput(input),
                ToNetwork::SetRenderDistance(render_distance) => {
                    ToServer::SetRenderDistance(render_distance)
                }
            };
            self.client
                .send_message(bincode::serialize(&message).unwrap());
        }
    }

    pub fn receive_messages(&mut self) {
        while let Some(event) = self.client.next_event() {
            match event {
                ClientEvent::Connection => (),
                ClientEvent::ConnectionClosed => panic!("Connection closed"),
                ClientEvent::Message(msg) => {
                    //println!("Network: received event {:?}", message);
                    match bincode::deserialize(msg.as_ref()).unwrap() {
                        ToClient::NewChunkFragment(pos, fpos, frag) => {
                            //println!("Network: received chunk fragment @ {:?}, {:?}", pos, fpos);
                            self.input_tx
                                .send(ToInput::NewChunkFragment(
                                    pos,
                                    fpos,
                                    deserialize_fragment(&frag[..]),
                                ))
                                .unwrap();
                        }
                        ToClient::NewChunkInfo(pos, info) => {
                            //println!("Received ChunkInfo @ {:?}", pos);
                            self.input_tx
                                .send(ToInput::NewChunkInfo(pos, info))
                                .unwrap();
                        }
                        ToClient::SetPos(pos) => {
                            self.input_tx.send(ToInput::SetPos(pos)).unwrap();
                        }
                    }
                }
            }
        }
        // if connection_lost {
        //     self.client
        //         .disconnect()
        //         .expect("Failed to disconnect client.");
        //     self.client
        //         .connect("127.0.0.1:1106")
        //         .expect("Failed to bind to socket.");
        //     println!("Reconnecting to server...");
        // }
    }
}
