extern crate bincode;
extern crate cobalt;

use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use ::core::messages::client::{ToMeshing, ToNetwork};
use ::core::messages::network::{ToClient, ToServer};

use self::cobalt::{Client, ClientEvent, MessageKind, PacketModifier, Socket, RateLimiter};

pub fn start<S, R, M>(client_rx: Receiver<ToNetwork>, meshing_tx: Sender<ToMeshing>, mut client: Client<S, R, M>) where
    S: Socket,
    R: RateLimiter,
    M: PacketModifier {

    loop {
        let mut active = false;

        // Messages to server
        loop {
            match client_rx.try_recv() {
                Ok(message) => match message {
                    ToNetwork::NewChunk(pos) => {
                        println!("Network: sent chunk @ {:?}", pos);
                        let out = ToServer::NewChunk(pos);
                        client.connection().unwrap().send(MessageKind::Reliable, bincode::serialize(&out, bincode::Infinite).unwrap());
                    },
                },
                Err(kind) => match kind {
                    // TODO: something better than panicking
                    TryRecvError::Disconnected => panic!(),
                    TryRecvError::Empty => break,
                },
            }
            active = true;
        }

        client.send(false).unwrap();

        // Messages from server
        loop {
            match client.receive() {
                Ok(message) => {
                    //println!("Network: received event {:?}", message);
                    match message {
                        ClientEvent::Message(bytes) => match bincode::deserialize(bytes.as_ref()).unwrap() {
                            ToClient::NewChunkFragment(pos, fpos, chunk) => {
                                //println!("Network: received chunk fragment @ {:?}, {:?}", pos, fpos);
                                meshing_tx.send(ToMeshing::NewChunkFragment(pos, fpos, chunk)).unwrap();
                            }
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
            active = true;
        }

        // Sleep for "a while" if not active, to save CPU
        if !active {
            //::std::thread::sleep(::std::time::Duration::from_millis(1));
            ();
        }
    }
}