use cardano_storage::chain_state;
use exe_common::{sync, parse_genesis_data, genesis_data};

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
        router.get(":network/chain-state-delta/:epochid/:to", self, "utxos-delta")
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {

        let (net, from) = match common::get_network_and_epoch(req, &self.networks) {
            None => { return Ok(Response::with(status::BadRequest)); }
            Some(x) => x
        };

        // FIXME: this is slow, should keep genesis_data in memory.
        let genesis_data = {
            let genesis_data = genesis_data::get_genesis_data(&net.config.genesis_prev)
                .expect("genesis data not found");
            parse_genesis_data::parse_genesis_data(genesis_data.as_bytes())
        };

        let to = common::validate_epochid(
            &req.extensions.get::<Router>().unwrap().find("to").unwrap()).unwrap();

        let storage = net.storage.read().unwrap();

        let from_block = chain_state::get_last_block_of_epoch(&storage, from).unwrap();

        let to_state = sync::get_chain_state_at_end_of(
            &storage, to, &genesis_data).unwrap();

        let mut res = vec![];
        chain_state::write_chain_state_delta(
            &storage,
            &genesis_data,
            &to_state,
            &from_block,
            &mut res).unwrap();

        Ok(Response::with((status::Ok, res)))
    }
}
