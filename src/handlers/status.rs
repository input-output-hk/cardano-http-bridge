use super::super::config::Networks;
use cardano::block;
use cardano::util::{hex, try_from_slice::TryFromSlice};
use cardano_storage::{tag, Error, types::header_to_blockhash};
use exe_common::{sync, network::{Peer, Api}};
use std::sync::Arc;
use std::collections::HashMap;

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
            Some((_, n)) => n
        };

        let (height, date, hash) = match &net.storage.read().unwrap().get_block_from_tag(tag::HEAD) {
            Ok(b) => (
                b.get_header().get_difficulty().0,
                match b.get_header().get_blockdate().get_epoch_and_slot() {
                    (e, b) => (Some(e), b)
                },
                hex::encode(&header_to_blockhash(&b.get_header().compute_hash())),
            ),
            Err(Error::NoSuchTag) => (0 as u64, (None, None), String::new()),
            Err(err) => {
                error!("error while reading difficutly from HEAD: {:?}", err);
                return Ok(Response::with(status::InternalServerError))
            }
        };

        let resp = json!({
            "tip": {
                "local": {
                    "height": height,
                    "slot": date,
                    "hash": hash,
                }
            }
        });

        return Ok(Response::with((status::Ok, resp.to_string())));
    }
}
