use cardano_storage::{epoch};

use std::sync::{Arc};

use iron;
use iron::{Request, Response, IronResult};
use iron::status;

use router::{Router};

use super::super::config::{Networks};
use super::common;

pub struct Handler {
    networks: Arc<Networks>
}
impl Handler {
    pub fn new(networks: Arc<Networks>) -> Self {
        Handler {
            networks: networks
        }
    }
    pub fn route(self, router: &mut Router) -> &mut Router {
        router.get(":network/epoch/:epochid", self, "epoch")
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let (net, epochid) = match common::get_network_and_epoch(req, &self.networks) {
            None => { return Ok(Response::with(status::BadRequest)); }
            Some(x) => x
        };

        let storage = net.storage.read().unwrap();
        let opackref = epoch::epoch_read_pack(&storage.config, epochid);
        match opackref {
            Err(_) => {
                return Ok(Response::with(status::NotFound));
            },
            Ok(packref) => {
                let path = storage.config.get_pack_filepath(&packref);
                Ok(Response::with((status::Ok, path)))
            },
        }
    }
}
