use super::super::config::Networks;
use cardano::util::hex;
use cardano_storage::{tag, types::header_to_blockhash, Error};
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
        router.get(":network/status", self, "bridge_status")
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let net = match common::get_network(req, &self.networks) {
            None => return Ok(Response::with(status::BadRequest)),
            Some((_, n)) => n,
        };

        let local_tip_json = match &net.storage.read().unwrap().get_block_from_tag(tag::HEAD)
        {
            Ok(b) => Some(json!({
                "height": u64::from(b.header().difficulty()),
                "slot": b.header().blockdate().epoch_and_slot(),
                "hash": hex::encode(&header_to_blockhash(&b.header().compute_hash())),
            })),
            Err(Error::NoSuchTag) => None,
            Err(err) => {
                error!("error while reading difficutly from HEAD: {:?}", err);
                return Ok(Response::with(status::InternalServerError));
            }
        };

        let resp = json!({
            "tip": {
                "local": local_tip_json
            }
        });

        return Ok(Response::with((status::Ok, resp.to_string())));
    }
}
