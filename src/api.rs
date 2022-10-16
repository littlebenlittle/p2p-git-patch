mod unix_socket;
mod protocol;

pub use unix_socket::Server as UnixSocketServer;

pub use protocol::{Request, Response, IdError, UpdateError, PatchError};

pub trait Client  {
    fn send_response(&mut self, response: Response);
}

pub enum Error {}
