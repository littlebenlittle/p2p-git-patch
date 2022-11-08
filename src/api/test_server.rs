use super::{
    protocol, Client as ClientTrait, ClientResult, Error, Request, Response, Server as ServerTrait,
};

use core::pin::Pin;
use core::task::{Context, Poll};
use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::stream::{FusedStream, Stream, StreamExt};

use libp2p::PeerId;

pub struct Client {
    api_tx: mpsc::Sender<Request>,
    api_rx: mpsc::Receiver<Response>,
}

impl ClientTrait for Client {
    fn get_id(&mut self) -> ClientResult<PeerId> {
        async_std::task::block_on(async move {
            self.api_tx.try_send(Request::Id { nickname: None })?;
            match self.api_rx.select_next_some().await {
                protocol::Response::Id(resp) => Ok(resp?),
                r => Err(Error::UnexpectedResponseType(r)),
            }
        })
    }
    fn get_peer(&mut self, nickname: &str) -> ClientResult<PeerId> {
        async_std::task::block_on(async move {
            self.api_tx.try_send(Request::Id { nickname: Some(nickname.to_owned()) })?;
            match self.api_rx.select_next_some().await {
                protocol::Response::Id(resp) => Ok(resp?),
                r => Err(Error::UnexpectedResponseType(r)),
            }
        })
    }
    fn shutdown(&mut self) -> ClientResult<()> {
        async_std::task::block_on(async move {
            self.api_tx.try_send(Request::Shutdown)?;
            match self.api_rx.select_next_some().await {
                protocol::Response::Shutdown(resp) => Ok(resp?),
                r => Err(Error::UnexpectedResponseType(r)),
            }
        })
    }
    fn add_peer(&mut self, peer: PeerId, nickname: &str) -> ClientResult<()> {
        unimplemented!()
    }
}

pub struct Server {
    clients: Vec<(
        mpsc::Sender<protocol::Response>,
        mpsc::Receiver<protocol::Request>,
    )>,
}

impl Server {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            clients: Vec::new(),
        })
    }
    /// create a new a client for this server
    pub fn client(&mut self) -> Client {
        let (server_tx, client_rx) = mpsc::channel::<protocol::Response>(1);
        let (client_tx, server_rx) = mpsc::channel::<protocol::Request>(1);
        self.clients.push((server_tx, server_rx));
        Client {
            api_rx: client_rx,
            api_tx: client_tx,
        }
    }
}

impl ServerTrait for Server {}

impl Unpin for Server {}

impl Stream for Server {
    type Item = (oneshot::Sender<Response>, Request);
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unimplemented!()
    }
}

impl FusedStream for Server {
    fn is_terminated(&self) -> bool {
        unimplemented!()
    }
}
