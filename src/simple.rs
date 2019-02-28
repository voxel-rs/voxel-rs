//! Simple implementations used as placeholders for necessary but complicated features.

pub mod network {
    use crate::network::{Client, ClientEvent, ConnectionId, Server, ServerEvent};
    use std::sync::mpsc::{Receiver, Sender, TryRecvError};

    pub struct SimpleServer {
        from_client: Receiver<Vec<u8>>,
        to_client: Sender<Vec<u8>>,
        sent_connected: bool,
    }

    impl SimpleServer {
        pub fn new(from_client: Receiver<Vec<u8>>, to_client: Sender<Vec<u8>>) -> Self {
            Self {
                from_client,
                to_client,
                sent_connected: false,
            }
        }
    }

    impl Server for SimpleServer {
        fn next_event(&mut self) -> Option<ServerEvent> {
            if !self.sent_connected {
                self.sent_connected = true;
                return Some(ServerEvent::Connection(0));
            }
            match self.from_client.try_recv() {
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => Some(ServerEvent::ConnectionClosed(0)),
                Ok(message) => Some(ServerEvent::Message(0, message)),
            }
        }

        fn send_message(&mut self, client: ConnectionId, message: Vec<u8>) {
            if client == 0 {
                self.to_client.send(message).unwrap();
            } else {
                println!("WARNING: request to send message to client {}, but SimpleServer only handles client 0!", client);
            }
        }
    }

    pub struct SimpleClient {
        from_server: Receiver<Vec<u8>>,
        to_server: Sender<Vec<u8>>,
        sent_connected: bool,
    }

    impl SimpleClient {
        pub fn new(from_server: Receiver<Vec<u8>>, to_server: Sender<Vec<u8>>) -> Self {
            Self {
                from_server,
                to_server,
                sent_connected: false,
            }
        }
    }

    impl Client for SimpleClient {
        fn next_event(&mut self) -> Option<ClientEvent> {
            if !self.sent_connected {
                self.sent_connected = true;
                return Some(ClientEvent::Connection);
            }
            match self.from_server.try_recv() {
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => Some(ClientEvent::ConnectionClosed),
                Ok(message) => Some(ClientEvent::Message(message)),
            }
        }

        fn send_message(&mut self, message: Vec<u8>) {
            self.to_server.send(message).unwrap();
        }
    }
}
