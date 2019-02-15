use super::super::config::Networks;
use cardano::block;
use cardano::util::{hex, try_from_slice::TryFromSlice};
use cardano_storage::{tag, Error};
use std::sync::Arc;

use iron;
use iron::status;
use iron::{IronResult, Request, Response};

use router;
use router::Router;
use serde_json::json;

use super::common;

pub struct Handler {
    networks: Arc<Networks>,
}
impl Handler {
    pub fn new(networks: Arc<Networks>) -> Self {
        Handler { networks: networks }
    }
    pub fn route(self, router: &mut Router) -> &mut Router {
        router.get(":network/height", self, "current_local_chain_height")
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {

        let net = match common::get_network(req, &self.networks) {
            None => return Ok(Response::with(status::BadRequest)),
            Some((_, net)) => net
        };

        let height = match &net.storage.read().unwrap().get_block_from_tag(tag::HEAD) {
            Ok(b) => b.get_header().get_difficulty().0,
            Err(Error::NoSuchTag) => 0,
            Err(err) => {
                error!("error while reading difficutly from HEAD: {:?}", err);
                return Ok(Response::with(status::InternalServerError))
            }
        };

        let resp = json!({
            "height": height
        });

        return Ok(Response::with((status::Ok, resp.to_string())));
    }
}
