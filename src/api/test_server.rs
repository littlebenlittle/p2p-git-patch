use super::{
    protocol, Client as ClientTrait, ClientError, ClientId, ClientResult, Request, Response,
    Server as ServerTrait, ServerError, ServerResult,
};

use async_trait::async_trait;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::channel::mpsc::{self, Receiver, Sender};
use futures::channel::oneshot;
use futures::{stream::FusedStream, SinkExt, Stream, StreamExt};
use std::collections::BTreeMap;
use std::time::Duration;

use libp2p::PeerId;

pub struct Client {
    api_tx: Sender<Request>,
    api_rx: Receiver<Response>,
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
    fn add_peer(&mut self, peer_id: PeerId, nickname: &str) -> ClientResult<()> {
        async_std::task::block_on(async move {
            self.api_tx.try_send(Request::AddPeer { peer_id, nickname: nickname.to_owned() })?;
            let resp = self.next_response().await?;
            match resp {
                protocol::Response::AddPeer(r) => Ok(r?),
                r => Err(ClientError::UnexpectedResponseType(r)),
            }
        })
    }
}

pub struct Server {
    clients: BTreeMap<ClientId, (Sender<protocol::Response>, Receiver<protocol::Request>)>,
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

#[async_trait]
impl ServerTrait for Server {
    async fn send_response(&mut self, client: &ClientId, res: Response) {
        log::debug!("api server sending response to client: {res:?}");
        if let Some((tx, rx)) = self.clients.get_mut(client) {
            match tx.send(res).await {
                Ok(()) => {},
                Err(e) => log::error!("{e:?}")
            }
        }
    }
}

impl Unpin for Server {}

impl Stream for Server {
    type Item = (ClientId, Request);
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // TODO: this polls clients in the order they were created, which is unnecessary
        for (client_id, (_res_tx, req_rx)) in &mut self.get_mut().clients {
            let req_rx = std::pin::Pin::new(req_rx);
            match req_rx.poll_next(cx) {
                Poll::Pending => {}
                Poll::Ready(req) => match req {
                    Some(r) => return Poll::Ready(Some((*client_id, r))),
                    None => return Poll::Ready(None),
                },
            }
        }
        Poll::Pending
    }
}

impl FusedStream for Server {
    fn is_terminated(&self) -> bool {
        self.clients.iter().all(|(_id, (_tx, rx))| rx.is_terminated())
    }
}
