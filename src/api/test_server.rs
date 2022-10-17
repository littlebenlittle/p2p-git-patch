use super::{Client, Server as ServerTrait, Request};

use futures::stream::FusedStream;
use futures::stream::Stream;
use futures::channel::oneshot;
use core::task::{Context, Poll};
use core::pin::Pin;

use std::error::Error;
use std::path::{Path, PathBuf};

pub struct Server {}

impl Server {
    pub fn new(shutdown_tx: oneshot::Receiver<()>) -> Result<Box<Self>, Box<dyn Error>> {
        Ok(Box::new(Self {}))
    }
    pub async fn shutdown(&self) -> oneshot::Sender<()> {
        unimplemented!()
    }
}

impl ServerTrait for Server {}

impl Unpin for Server {}

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
