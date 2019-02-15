use cardano::{block::Verify, tx::TxAux};

use std::{io::Read, sync::Arc};

use iron;
use iron::status;
use iron::{IronResult, Request, Response};

use router::Router;

use serde_json;

use super::super::config::Networks;
use super::common;
use exe_common::{config::net, network::Api, sync};

pub struct Handler {
    networks: Arc<Networks>,
}
impl Handler {
    pub fn new(networks: Arc<Networks>) -> Self {
        Handler { networks: networks }
    }
    pub fn route(self, router: &mut Router) -> &mut Router {
        router.post(":network/txs/signed", self, "txs_signed_send")
    }
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        fn read_txaux_from_req_str(tx_str: &str) -> Option<TxAux> {
            let json = serde_json::from_str::<serde_json::Value>(tx_str).ok()?;
            let base_64 = json.as_object()?.get("signedTx")?.as_str()?;
            let bytes = base64::decode(&base_64).ok()?;
            let mut de = cbor_event::de::Deserializer::from(std::io::Cursor::new(&bytes));
            de.deserialize_complete().ok()
        }
        let mut req_body_str = String::new();
        req.body.read_to_string(&mut req_body_str).unwrap();
        let txaux = match read_txaux_from_req_str(req_body_str.as_str()) {
            None => {
                return Ok(Response::with((
                    status::BadRequest,
                    "Invalid input format for transaction",
                )));
            }
            Some(x) => x,
        };

        let (net_name, net) = match common::get_network(req, &self.networks) {
            None => {
                return Ok(Response::with((status::BadRequest, "Invalid network name")));
            }
            Some(x) => x,
        };
        let netcfg_file = net.storage.read().unwrap().config.get_config_file();
        let net_cfg = net::Config::from_file(&netcfg_file).expect("no network config present");

        if let Err(verify_error) = txaux.verify(net_cfg.protocol_magic) {
            return Ok(Response::with((
                status::BadRequest,
                format!("Transaction failed verification: {}", verify_error),
            )));
        }

        let mut peer = sync::get_peer(&net_name, &net_cfg, true);
        match peer.send_transaction(txaux) {
            Err(e) => {
                return Ok(Response::with((
                    status::InternalServerError,
                    format!("Failed to send to peers: {}", e),
                )));
            }
            Ok(value) => assert!(value),
        };

        Ok(Response::with((
            status::Ok,
            "Transaction sent successfully!",
        )))
    }
}
