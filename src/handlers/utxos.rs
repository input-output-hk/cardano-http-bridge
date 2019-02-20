use cardano_storage::chain_state;
use exe_common::genesisdata;

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
        router.get(":network/chain-state/:epochid", self, "chain-state")
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

        let mut res = vec![];

        let genesis_str = genesisdata::data::get_genesis_data(&net.config.genesis_prev).unwrap();
        let genesis_data = genesisdata::parse::parse(genesis_str.as_bytes());

        let last_hdr =
            &chain_state::get_last_block_of_epoch(&net.storage.read().unwrap(), epochid).unwrap();

        let chain_state =
            chain_state::read_chain_state(&net.storage.read().unwrap(), &genesis_data, last_hdr)
                .unwrap();

        chain_state::write_chain_state(&net.storage.read().unwrap(), &genesis_data, &chain_state)
            .unwrap();

        Ok(Response::with((status::Ok, res)))
    }
}
