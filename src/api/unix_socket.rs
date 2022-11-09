use super::{Client, ClientId, Server as ServerTrait, Request, Response};

use futures::stream::FusedStream;
use futures::stream::Stream;
use futures::channel::mpsc;
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

impl ServerTrait for Server {}

impl Unpin for Server {}

impl Stream for Server {
    type Item = (ClientId, Request);
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unimplemented!()
    }
}

impl FusedStream for Server {
    fn is_terminated(&self) -> bool {
        unimplemented!()
    }
}
