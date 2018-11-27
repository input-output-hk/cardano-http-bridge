use super::super::config::{Networks};
use cardano_storage::{tag, block_read};
use cardano::util::{hex, try_from_slice::TryFromSlice};
use cardano::block;
use std::sync::{Arc};

use iron;
use iron::{Request, Response, IronResult};
use iron::status;

use router;
use router::{Router};

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
        router.get(":network/block/:blockid", self, "block")
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let ref network_name = req.extensions.get::<router::Router>().unwrap().find("network").unwrap();

        if ! common::validate_network_name (network_name) {
            return Ok(Response::with(status::BadRequest));
        }

        let net = match self.networks.get(network_name.to_owned()) {
            None => return Ok(Response::with(status::BadRequest)),
            Some(net) => net
        };

        let ref blockid = req.extensions.get::<router::Router>().unwrap().find("blockid").unwrap();
        if ! blockid.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            error!("invalid blockid: {}", blockid);
            return Ok(Response::with(status::BadRequest));
        }
        let storage = net.storage.read().unwrap();
        let hh_bytes = match tag::read(&storage, &blockid) {
            None => hex::decode(&blockid).unwrap(),
            Some(t) => t
        };
        let hh = block::HeaderHash::try_from_slice(&hh_bytes).expect("blockid invalid");
        info!("querying block header: {}", hh);

        match block_read(&storage, &hh) {
            Err(cardano_storage::Error::HashNotFound(_)) => {
                warn!("block `{}' does not exist", hh);
                Ok(Response::with((status::NotFound, "Not Found")))
            },
            Err(err) => {
                error!("error while reading block at location: {:?}", err);
                Ok(Response::with(status::InternalServerError))
            },
            Ok(rblk) => {
                Ok(Response::with((status::Ok, rblk.as_ref())))
            }
        }
    }
}
