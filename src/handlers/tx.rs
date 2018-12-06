use cardano::{tx::TxAux};

use std::{io::Read, sync::Arc};

use iron;
use iron::{Request, Response, IronResult};
use iron::status;

use router::{Router};

use serde_json;

use super::super::config::{Networks};
use super::common;
use exe_common::{config::net, network::Api, sync};

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
        router.post(":network/txs/signed", self, "txs_signed_send")
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        fn read_txaux_from_req_str(tx_str: &str) -> Option<TxAux> {
            let json = serde_json::from_str::<serde_json::Value>(tx_str).ok()?;
            let base_64 = json.
                as_object()?.
                get("signedTx")?.
                as_str()?.clone();
            let bytes = base64::decode(&base_64).ok()?;
            cbor_event::de::RawCbor::from(&bytes).deserialize_complete().ok()
        }
        let mut req_body_str = String::new();
        req.body.read_to_string(&mut req_body_str).unwrap();
        let txaux = match read_txaux_from_req_str(req_body_str.as_str()) {
            None => { return Ok(Response::with(status::BadRequest)); }
            Some(x) => x
        };

        println!("Received a valid txauth to send:\n{:?}", txaux);

        let (net_name, net) = match common::get_network(req, &self.networks) {
            None => { return Ok(Response::with(status::BadRequest)); }
            Some(x) => x
        };

        let netcfg_file = net.storage.config.get_config_file();
        let net_cfg = net::Config::from_file(&netcfg_file).expect("no network config present");
        let mut peer = sync::get_peer(&net_name, &net_cfg, true);

        println!("found a peer to send from!");

        // match peer.send_transaction(txaux) {
        //     Err(_) => return Ok(Response::with(status::InternalServerError)),
        //     Ok(value) => assert!(value)
        // };

        Ok(Response::with((status::Ok, "Transaction sent successfully!")))
    }
}
