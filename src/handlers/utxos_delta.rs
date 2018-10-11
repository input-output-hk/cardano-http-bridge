use cardano_storage::utxo;

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
        router.get(":network/utxos-delta/:epochid/:to", self, "utxos-delta")
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {

        let (net, from) = match common::get_network_and_epoch(req, &self.networks) {
            None => { return Ok(Response::with(status::BadRequest)); }
            Some(x) => x
        };

        let to = common::validate_epochid(
            &req.extensions.get::<Router>().unwrap().find("to").unwrap()).unwrap();

        let mut res = vec![];

        let to_state = utxo::get_utxos_for_epoch(&net.storage, to).unwrap();

        utxo::write_utxos_delta(&net.storage,
                                &to_state.last_block,
                                &to_state.last_date,
                                &to_state.utxos,
                                Some(from), &mut res).unwrap();

        Ok(Response::with((status::Ok, res)))
    }
}
