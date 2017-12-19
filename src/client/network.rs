use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use ::core::messages::client::{ToMeshing, ToNetwork};
use ::core::messages::network::{ToClient, ToServer};
use ::network::{NetworkReceiver, NetworkSender};

pub fn start<R, S>(client_rx: Receiver<ToNetwork>, meshing_tx: Sender<ToMeshing>, server_rx: R, server_tx: S) where
    R: NetworkReceiver<ToClient>,
    S: NetworkSender<ToServer> {
    loop {
        let mut active = false;

        // Messages to server
        loop {
            match client_rx.try_recv() {
                Ok(message) => match message {
                    ToNetwork::NewChunk(pos) => {
                        println!("Network: sent chunk @ {:?}", pos);
                        //meshing_tx.send(ToMeshing::NewChunk(pos, Box::new(chunk))).unwrap();
                        server_tx.send(ToServer::NewChunk(pos)).unwrap();
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

        // Messages from server
        loop {
            match server_rx.try_recv() {
                Ok(message) => match message {
                    ToClient::NewChunk(pos, chunk) => {
                        println!("Network: received chunk @ {:?}", pos);
                        meshing_tx.send(ToMeshing::NewChunk(pos, chunk)).unwrap();
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

        // Sleep for "a while" if not active, to save CPU
        if !active {
            ::std::thread::sleep(::std::time::Duration::from_millis(1));
        }
    }
}