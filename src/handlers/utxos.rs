use cardano_storage::utxo;

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
        router.get(":network/utxos/:epochid", self, "utxos")
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

        let utxo_state = utxo::get_utxos_for_epoch(&net.storage, epochid).unwrap();

        utxo::write_utxos_delta(
            &net.storage,
            &utxo_state.last_block,
            &utxo_state.last_date,
            &utxo_state.utxos,
            None,
            &mut res,
        )
        .unwrap();

        Ok(Response::with((status::Ok, res)))
    }
}
