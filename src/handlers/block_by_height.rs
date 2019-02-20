use super::super::config::Networks;
use std::sync::Arc;

use iron;
use iron::status;
use iron::{IronResult, Request, Response};

use router;
use router::Router;

use super::common;

pub struct Handler {
    networks: Arc<Networks>,
}
impl Handler {
    pub fn new(networks: Arc<Networks>) -> Self {
        Handler { networks: networks }
    }
    pub fn route(self, router: &mut Router) -> &mut Router {
        router.get(":network/height/:height", self, "block_by_height")
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let params = req.extensions.get::<router::Router>().unwrap();

        let net = match common::get_network(req, &self.networks) {
            None => return Ok(Response::with(status::BadRequest)),
            Some((_, net)) => net,
        };

        let ref height_str = params.find("height").unwrap();

        if !height_str.chars().all(|c| c.is_numeric()) {
            error!("invalid block height: {}", height_str);
            return Ok(Response::with(status::BadRequest));
        }

        let height = height_str
            .parse::<u64>()
            .expect(&format!("Failed to parse block height: {}", height_str));

        let storage = &(net.storage).read().unwrap();
        match storage.block_location_by_height(height) {
            Err(_) => {
                warn!("block with height `{}' does not exist", height);
                Ok(Response::with((status::NotFound, "Not Found")))
            }
            Ok(loc) => {
                debug!("blk location: {:?}", loc);
                match storage.read_block_at(&loc) {
                    Err(_) => {
                        error!("error while reading block at location: {:?}", loc);
                        Ok(Response::with(status::InternalServerError))
                    }
                    Ok(rblk) => Ok(Response::with((status::Ok, rblk.as_ref()))),
                }
            }
        }
    }
}
