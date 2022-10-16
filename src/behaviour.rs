use async_std::io;
use async_trait::async_trait;
use futures::{AsyncRead, AsyncWrite};

use libp2p::{
    core::upgrade::ProtocolName,
    mdns::{Mdns, MdnsEvent},
    request_response::{ProtocolSupport, RequestResponse, RequestResponseCodec, RequestResponseEvent},
    NetworkBehaviour,
};

pub fn new() -> GitPatch {
    RequestResponse::new(
        GitPatchExchangeCodec(),
        std::iter::once((GitPatchExchangeProtocol(), ProtocolSupport::Full)),
        Default::default(),
    )
}

pub type GitPatch = RequestResponse<GitPatchExchangeCodec>;
type GitPatchEvent = RequestResponseEvent<GitPatchRequest, GitPatchResponse>;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event")]
pub struct Behaviour {
    pub mdns: Mdns,
    pub git_patch: GitPatch,
}

#[derive(Debug)]
pub enum Event {
    Mdns(MdnsEvent),
    Git(GitPatchEvent),
}

impl From<MdnsEvent> for Event {
    fn from(e: MdnsEvent) -> Self {
        Self::Mdns(e)
    }
}

impl From<GitPatchEvent> for Event {
    fn from(e: GitPatchEvent) -> Self {
        Self::Git(e)
    }
}

#[derive(Debug, Clone)]
pub struct GitPatchExchangeProtocol();

#[derive(Clone)]
pub struct GitPatchExchangeCodec();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitPatchRequest {

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitPatchResponse(Vec<u8>);

impl ProtocolName for GitPatchExchangeProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/file-exchange/1".as_bytes()
    }
}

#[async_trait]
impl RequestResponseCodec for GitPatchExchangeCodec {
    type Protocol = GitPatchExchangeProtocol;
    type Request = GitPatchRequest;
    type Response = GitPatchResponse;
    async fn read_request<T>(
        &mut self,
        _: &GitPatchExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        unimplemented!()
    }

    async fn read_response<T>(
        &mut self,
        _: &GitPatchExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        unimplemented!()
    }

    async fn write_request<T>(
        &mut self,
        _: &GitPatchExchangeProtocol,
        io: &mut T,
        request: GitPatchRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        unimplemented!()
    }

    async fn write_response<T>(
        &mut self,
        _: &GitPatchExchangeProtocol,
        io: &mut T,
        response: GitPatchResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        unimplemented!()
    }
}
