mod protocol;
mod unix_socket;
mod test_server;

use crate::config::MultiaddrUnixSocket;
use futures::stream::FusedStream;
use libp2p::multiaddr::Protocol;
use std::error::Error;
use std::sync::mpsc;

pub use protocol::{IdError, PatchError, Request, Response, UpdateError};
pub use unix_socket::Server as UnixSocketServer;
pub use test_server::Server as TestServer;

pub type Client = mpsc::Sender<Response>;

pub trait Server: FusedStream<Item = (Client, Request)> + Unpin + Send {}

impl TryFrom<MultiaddrUnixSocket> for Box<dyn Server> {
    type Error = Box<dyn Error>;
    fn try_from(addr: MultiaddrUnixSocket) -> Result<Self, Self::Error> {
        use MultiaddrUnixSocket::*;
        let server = match addr {
            Multiaddr(mut addr) => match addr.pop() {
                Some(proto) => return Err(Box::new(ProtocolError::UnhandledProtocol(proto))),
                None => return Err(Box::new(ProtocolError::EmptyProtocolString))
            }
            UnixSocket(path) => Box::new(UnixSocketServer::new(path)?),
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
