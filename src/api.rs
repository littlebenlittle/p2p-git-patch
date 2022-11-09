pub mod protocol;
mod test_server;
mod unix_socket;

use crate::config::MultiaddrUnixSocket;
use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::stream::FusedStream;
use libp2p::multiaddr::Protocol;
use libp2p::PeerId;

use std::error::Error;

pub use protocol::{IdError, PatchError, Request, Response, UpdateError};
pub use test_server::Client as TestClient;
pub use test_server::Server as TestServer;
pub use unix_socket::Server as UnixSocketServer;

type ClientResult<T> = Result<T, ClientError>;

pub trait Client {
    fn get_id(&mut self) -> ClientResult<PeerId>;
    fn get_peer(&mut self, nickname: &str) -> ClientResult<PeerId>;
    fn shutdown(&mut self) -> ClientResult<()>;
    fn add_peer(&mut self, peer: PeerId, nickname: &str) -> ClientResult<()>;
}

#[derive(Debug)]
pub enum ClientError {
    RequestError(mpsc::TrySendError<protocol::Request>),
    IdError(protocol::IdError),
    ShutdownError(protocol::ShutdownError),
    UnexpectedResponseType(protocol::Response),
    IoError(std::io::Error),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::RequestError(e) => {
                f.write_str("request error: ");
                e.fmt(f);
            }
            Self::IdError(e) => {
                std::fmt::Debug::fmt(e, f);
            }
            Self::ShutdownError(e) => {
                std::fmt::Debug::fmt(e, f);
            }
            Self::UnexpectedResponseType(resp) => {
                f.write_str("unexpected response: ");
                std::fmt::Debug::fmt(resp, f);
            }
            Self::IoError(e) => {
                std::fmt::Debug::fmt(e, f);
            }
        }
        Ok(())
    }
}

impl Error for ClientError {}

impl From<mpsc::TrySendError<protocol::Request>> for ClientError {
    fn from(e: mpsc::TrySendError<protocol::Request>) -> Self {
        Self::RequestError(e)
    }
}

impl From<protocol::IdError> for ClientError {
    fn from(e: protocol::IdError) -> Self {
        Self::IdError(e)
    }
}

impl From<protocol::ShutdownError> for ClientError {
    fn from(e: protocol::ShutdownError) -> Self {
        Self::ShutdownError(e)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

pub type ResponseSender = oneshot::Sender<Response>;
pub type ClientId = u16;

pub trait Server: FusedStream<Item = (ClientId, Request)> + Unpin + Send {}

impl TryFrom<MultiaddrUnixSocket> for Box<dyn Server> {
    type Error = Box<dyn Error>;
    fn try_from(addr: MultiaddrUnixSocket) -> Result<Self, Self::Error> {
        use MultiaddrUnixSocket::*;
        let server = match addr {
            Multiaddr(mut addr) => match addr.pop() {
                Some(proto) => return Err(Box::new(ProtocolError::UnhandledProtocol(proto))),
                None => return Err(Box::new(ProtocolError::EmptyProtocolString)),
            },
            UnixSocket(path) => Box::new(UnixSocketServer::new(path)?),
        };
        Ok(server)
    }
}

#[derive(Debug)]
pub enum ServerError {
    ClientIdOverflow
}

impl Error for ServerError {}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ClientIdOverflow => {
                f.write_str("exceeded max # of clients")
            }
        }
    }
}

pub type ServerResult<T> = Result<T, ServerError>;

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

impl<'a> std::error::Error for ProtocolError<'a> {}
