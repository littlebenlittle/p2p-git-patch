use super::{
    protocol, Client as ClientTrait, ClientId, ClientResult, ClientError, Request, Response,
    Server as ServerTrait, ServerError, ServerResult,
};

use core::pin::Pin;
use core::task::{Context, Poll};
use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::{stream::FusedStream, SinkExt, Stream, StreamExt};
use std::collections::BTreeMap;
use std::time::Duration;

use libp2p::PeerId;

pub struct Client {
    api_tx: mpsc::Sender<Request>,
    api_rx: mpsc::Receiver<Response>,
    timeout: Duration,
}

impl Client {
    /// try to receive a response or timeout
    /// after self.timeout
    async fn next_response(&mut self) -> std::io::Result<Response> {
        async_std::io::timeout(self.timeout, async {
            loop {
                match self.api_rx.next().await {
                    Some(r) => {
                        return Ok(r);
                    }
                    None => {}
                };
            }
        })
        .await
    }
}

impl ClientTrait for Client {
    fn get_id(&mut self) -> ClientResult<PeerId> {
        async_std::task::block_on(async move {
            self.api_tx.try_send(Request::Id { nickname: None })?;
            match self.api_rx.select_next_some().await {
                protocol::Response::Id(resp) => Ok(resp?),
                r => Err(ClientError::UnexpectedResponseType(r)),
            }
        })
    }
    fn get_peer(&mut self, nickname: &str) -> ClientResult<PeerId> {
        async_std::task::block_on(async move {
            self.api_tx.try_send(Request::Id {
                nickname: Some(nickname.to_owned()),
            })?;
            match self.api_rx.select_next_some().await {
                protocol::Response::Id(resp) => Ok(resp?),
                r => Err(ClientError::UnexpectedResponseType(r)),
            }
        })
    }
    fn shutdown(&mut self) -> ClientResult<()> {
        async_std::task::block_on(async move {
            self.api_tx.try_send(Request::Shutdown)?;
            let resp = self.next_response().await?;
            match resp {
                protocol::Response::Shutdown(r) => Ok(r?),
                r => Err(ClientError::UnexpectedResponseType(r)),
            }
        })
    }
    fn add_peer(&mut self, peer: PeerId, nickname: &str) -> ClientResult<()> {
        unimplemented!()
    }
}

pub struct Server {
    clients: BTreeMap<
        ClientId,
        (
            mpsc::Sender<protocol::Response>,
            mpsc::Receiver<protocol::Request>,
        ),
    >,
    last_client_id: ClientId,
}

impl Server {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            clients: BTreeMap::new(),
            last_client_id: 0,
        })
    }

    fn next_client_id(&mut self) -> ServerResult<ClientId> {
        if self.last_client_id < u16::MAX {
            self.last_client_id += 1;
            Ok(self.last_client_id - 1)
        } else {
            Err(ServerError::ClientIdOverflow)
        }
    }

    /// create a new a client for this server
    pub fn client(&mut self) -> ServerResult<Client> {
        let (server_tx, client_rx) = mpsc::channel::<protocol::Response>(1);
        let (client_tx, server_rx) = mpsc::channel::<protocol::Request>(1);
        let id = self.next_client_id()?;
        self.clients.insert(id, (server_tx, server_rx));
        Ok(Client {
            api_rx: client_rx,
            api_tx: client_tx,
            timeout: Duration::from_secs(3),
        })
    }
}

impl ServerTrait for Server {}

impl Unpin for Server {}

impl Stream for Server {
    type Item = (ClientId, Request);
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // TODO: this polls clients in the order they were created, which is unnecessary
        for (client_id, (_tx, rx)) in &mut self.get_mut().clients {
            let rx = std::pin::Pin::new(rx);
            match rx.poll_next(cx) {
                Poll::Pending => {}
                Poll::Ready(req) => match req {
                    Some(r) => return Poll::Ready(Some((*client_id, r))),
                    None => {}
                },
            }
        }
        Poll::Pending
    }
}

impl FusedStream for Server {
    fn is_terminated(&self) -> bool {
        unimplemented!()
    }
}
