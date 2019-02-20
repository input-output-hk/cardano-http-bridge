use std::{str::FromStr, sync::Arc};

use cardano::block;
use exe_common::genesisdata;
use iron;
use iron::status;
use iron::{IronResult, Request, Response};
use router;
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
        router.get(":network/genesis/:hash", self, "genesis_by_hash")
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let params = req.extensions.get::<router::Router>().unwrap();

        let ref network_name = params.find("network").unwrap();

        if !common::validate_network_name(network_name) {
            return Ok(Response::with((status::BadRequest, "Invalid network")));
        }

        let _net = match self.networks.get(network_name.to_owned()) {
            None => {
                return Ok(Response::with((
                    status::BadRequest,
                    "Failed to parse network",
                )));
            }
            Some(net) => net,
        };

        let ref hash = params.find("hash").unwrap();

        if !hash.chars().all(|c| c.is_ascii_alphanumeric()) {
            error!("invalid genesis hash: {}", hash);
            return Ok(Response::with((status::BadRequest, "Invalid genesis hash")));
        }

        let header_hash = match block::HeaderHash::from_str(hash) {
            Err(_) => {
                error!("failed to parse genesis hash: {}", hash);
                return Ok(Response::with((
                    status::BadRequest,
                    "Failed to parse genesis",
                )));
            }
            Ok(hh) => hh,
        };

        info!("Searching genesis: {}", header_hash);
        let genesis_data = genesisdata::data::get_genesis_data(&header_hash);

        match genesis_data {
            Err(hh) => {
                warn!("genesis `{}' does not exist", hh);
                Ok(Response::with((status::NotFound, "Not Found")))
            }
            Ok(json_str) => Ok(Response::with((status::Ok, json_str))),
        }
    }
}
