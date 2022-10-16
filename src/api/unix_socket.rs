use crate::api::{Client as ClientTrait, Request as ApiRequest, Response as ApiResponse};

use libp2p::Multiaddr;

use futures::stream::FusedStream;
use futures::stream::Stream;

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
    type Item = (Client, ApiRequest);
}

impl FusedStream for Server {}

pub struct Client {}

impl ClientTrait for Client {}
