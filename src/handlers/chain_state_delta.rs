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
        router.get(
            ":network/chain-state-delta/:epochid/:to",
            self,
            "chain-state-delta",
        )
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let (net, from) = match common::get_network_and_epoch(req, &self.networks) {
            None => {
                return Ok(Response::with(status::BadRequest));
            }
            Some(x) => x,
        };

        let to =
            common::validate_epochid(&req.extensions.get::<Router>().unwrap().find("to").unwrap())
                .unwrap();

        let genesis_str = genesisdata::data::get_genesis_data(&net.config.genesis_prev).unwrap();
        let genesis_data = genesisdata::parse::parse(genesis_str.as_bytes());

        let storage = net.storage.read().unwrap();

        let from_block = chain_state::get_last_block_of_epoch(&storage, from).unwrap();

        let to_state = sync::get_chain_state_at_end_of(&storage, to, &genesis_data).unwrap();

        let mut res = vec![];
        chain_state::write_chain_state_delta(
            &storage,
            &genesis_data,
            &to_state,
            &from_block,
            &mut res,
        )
        .unwrap();

        Ok(Response::with((status::Ok, res)))
    }
}
