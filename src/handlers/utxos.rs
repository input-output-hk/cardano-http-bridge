use cardano_storage::chain_state;
use exe_common::{genesisdata, sync};

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

        let genesis_str = genesisdata::data::get_genesis_data(&net.config.genesis_prev).unwrap();
        let genesis_data = genesisdata::parse::parse(genesis_str.as_bytes());

        let storage = net.storage.read().unwrap();

        let chain_state =
            sync::get_chain_state_at_end_of(&storage, epochid, &genesis_data).unwrap();

        let mut res = vec![];
        chain_state::write_chain_state_delta(
            &storage,
            &genesis_data,
            &chain_state,
            &net.config.genesis_prev,
            &mut res,
        )
        .unwrap();

        Ok(Response::with((status::Ok, res)))
    }
}
