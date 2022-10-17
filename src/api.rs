mod protocol;
mod unix_socket;

use futures::stream::FusedStream;
use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
use std::error::Error;
use std::sync::mpsc;

pub use protocol::{IdError, PatchError, Request, Response, UpdateError};
pub use unix_socket::Server as UnixSocketServer;

pub type Client = mpsc::Sender<Response>;

pub trait Server: FusedStream<Item = (Client, Request)> + Unpin {}

impl TryFrom<Multiaddr> for Box<dyn Server> {
    type Error = Box<dyn Error>;
    fn try_from(addr: Multiaddr) -> Result<Self, Self::Error> {
        if addr.is_empty() {
            return Err(Box::new(ProtocolError::EmptyProtocolString));
        }
        let server = match addr.pop().unwrap() {
            Protocol::Unix(path_str) => Box::new(UnixSocketServer::new(path_str.to_string())?),
            p => return Err(Box::new(ProtocolError::UnhandledProtocol(p))),
        };
        Ok(server)
    }
}

#[derive(Debug)]
enum ProtocolError<'a> {
    EmptyProtocolString,
    UnhandledProtocol(Protocol<'a>),
}

impl<'a> std::fmt::Display for ProtocolError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::EmptyProtocolString => f.write_str("empty protocol string")?,
            Self::UnhandledProtocol(p) => f.write_str("unhandled protocol {p}")?,
        }
        Ok(())
    }
}

impl<'a> Error for ProtocolError<'a> {}
