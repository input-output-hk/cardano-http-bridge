use cardano_storage::epoch;

use std::sync::Arc;

use iron;
use iron::status;
use iron::{IronResult, Request, Response};

use router::Router;

use super::super::config::Networks;
use super::common;

pub struct Handler {
    networks: Arc<Networks>,
}
impl Handler {
    pub fn new(networks: Arc<Networks>) -> Self {
        Handler { networks: networks }
    }
    pub fn route(self, router: &mut Router) -> &mut Router {
        router.get(":network/epoch/:epochid", self, "epoch")
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let (net, epochid) = match common::get_network_and_epoch(req, &self.networks) {
            None => {
                return Ok(Response::with(status::BadRequest));
            }
            Some(x) => x,
        };

        let opackref = epoch::epoch_read_pack(&net.storage.read().unwrap().config, epochid);
        match opackref {
            Err(_) => {
                return Ok(Response::with(status::NotFound));
            }
            Ok(packref) => {
                let path = net
                    .storage
                    .read()
                    .unwrap()
                    .config
                    .get_pack_filepath(&packref);
                Ok(Response::with((status::Ok, path)))
            }
        }
    }
}
