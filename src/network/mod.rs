//! Various network-related utilities.
//! For now this means `ChunkFragment` serialization and deserialization.

use crate::block::BlockId;
use crate::sim::chunk::ChunkFragment;
use crate::CHUNK_SIZE;

/// A client-side network event
pub enum ClientEvent {
    /// Connection with the server established.
    Connection,
    /// Connection with the server closed or lost.
    ConnectionClosed,
    /// Message received from the server.
    Message(Vec<u8>),
}

pub type ConnectionId = usize;

/// A server-side network event
pub enum ServerEvent {
    /// Connection with a new client established.
    Connection(ConnectionId),
    /// Connection with a client closed or lost.
    ConnectionClosed(ConnectionId),
    /// Message received from a client.
    Message(ConnectionId, Vec<u8>),
}

pub trait Server {
    /// Next event.
    fn next_event(&mut self) -> Option<ServerEvent>;
    /// Send a message.
    fn send_message(&mut self, connection: ConnectionId, message: Vec<u8>);
}

pub trait Client {
    /// Next event.
    fn next_event(&mut self) -> Option<ClientEvent>;
    /// Send a message.
    fn send_message(&mut self, message: Vec<u8>);
}

fn serialize_blocks(blocks: &[BlockId]) -> Vec<u8> {
    fn encode(out: &mut Vec<u8>, current_block: BlockId, mut count: u8) {
        if count == 0 {
            return;
        }
        let first_half = (current_block.0 / (1 << 8)) as u8;
        let second_half = (current_block.0 % (1 << 8)) as u8;
        if count > 1 {
            count |= 1 << 7;
            out.push(count);
        }
        out.push(first_half);
        out.push(second_half);
    }

    // Preallocate
    let mut out = Vec::with_capacity(blocks.len() * 2);
    if blocks.len() == 0 {
        return out;
    }
    let mut current_block = blocks[0];
    let mut count: u8 = 1;
    for &id in blocks.split_at(0).1.iter() {
        if id == current_block && count < 255 {
            count += 1;
        } else {
            encode(&mut out, current_block, count);
            current_block = id;
            count = 1;
        }
    }
    encode(&mut out, current_block, count);
    out
}

fn deserialize_blocks(bytes: &[u8]) -> Vec<BlockId> {
    let mut out = Vec::new();
    let mut it = bytes.iter();

    while let Some(b) = it.next() {
        let mut b = *b;
        // Multiple blocks
        let mut count = 1;
        if b & (1 << 7) > 0 {
            count = b ^ (1 << 7);
            match it.next() {
                Some(x) => b = *x,
                None => unreachable!(),
            }
        }
        let first_half = b as u16;
        let second_half = match it.next() {
            Some(x) => *x,
            None => unreachable!(),
        } as u16;
        for _ in 0..count {
            out.push(BlockId::from(first_half * (1 << 8) + second_half));
        }
    }

    out
}

pub fn serialize_fragment(frag: &ChunkFragment) -> Vec<u8> {
    serialize_blocks(&frag[..])
}

pub fn deserialize_fragment(bytes: &[u8]) -> Box<ChunkFragment> {
    let mut frag = Box::new([BlockId::from(0); CHUNK_SIZE]);
    let blocks = deserialize_blocks(bytes);
    for (f, b) in frag.iter_mut().zip(blocks.iter()) {
        *f = *b;
    }
    frag
}
