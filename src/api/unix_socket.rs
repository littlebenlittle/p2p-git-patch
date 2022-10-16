use super::{Client as ClientTrait, Request, Response};

use libp2p::Multiaddr;

use futures::stream::FusedStream;
use futures::stream::Stream;
use core::task::{Context, Poll};
use core::pin::Pin;

use std::error::Error;
use std::path::{Path, PathBuf};

pub struct Server {}

impl Server {
    pub fn new(socket_path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {})
    }
}

impl TryFrom<Multiaddr> for Server {
    type Error = Box<dyn Error>;
    fn try_from(addr: Multiaddr) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

impl Stream for Server {
    type Item = (Client, Request);
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unimplemented!()
    }
}

impl FusedStream for Server {
    fn is_terminated(&self) -> bool {
        unimplemented!()
    }
}

pub struct Client {}

impl ClientTrait for Client {
    fn send_response(&mut self, response: Response) {
        unimplemented!()
    }
}
