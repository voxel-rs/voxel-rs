use ::std::sync::mpsc::{SendError, RecvError, TryRecvError};

/**
 * An interface for sending data over the network
 */
pub trait NetworkSender<T> {
    fn send(&self, t: T) -> Result<(), SendError<T>>;
}

/**
 * An interface for receiving data over the network
 */
pub trait NetworkReceiver<T> {
    fn recv(&self) -> Result<T, RecvError>;
    fn try_recv(&self) -> Result<T, TryRecvError>;
}

/**
 * Dummy implementations
 */
pub struct ChannelSender<T>(::std::sync::mpsc::Sender<T>);
pub struct ChannelReceiver<T>(::std::sync::mpsc::Receiver<T>);

impl<T> NetworkSender<T> for ChannelSender<T> {
    fn send(&self, t: T) -> Result<(), SendError<T>> {
        self.0.send(t)
    }
}

impl<T> NetworkReceiver<T> for ChannelReceiver<T> {
    fn recv(&self) -> Result<T, RecvError> {
        self.0.recv()
    }
    fn try_recv(&self) -> Result<T, TryRecvError> {
        self.0.try_recv()
    }
}

pub fn network_channel<T>() -> (ChannelSender<T>, ChannelReceiver<T>) {
    let (tx, rx) = ::std::sync::mpsc::channel();
    (ChannelSender::<T>(tx), ChannelReceiver::<T>(rx))
}