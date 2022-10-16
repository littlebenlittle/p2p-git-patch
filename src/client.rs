use crate::api::Response;

pub trait Client  {
    fn send_response(&mut self, response: Response);
}

pub enum Error {}
