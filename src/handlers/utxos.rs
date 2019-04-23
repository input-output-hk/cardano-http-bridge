use cardano_storage::{chain_state, tag, Error};
use exe_common::network::BlockRef;
use exe_common::{genesisdata, sync};

use std::sync::Arc;

use iron;
use iron::status;
use iron::{IronResult, Request, Response};

use router::Router;

use super::super::config::Networks;
use super::common;

use std::str::FromStr;

pub struct Handler {
    networks: Arc<Networks>,
}
impl Handler {
    pub fn new(networks: Arc<Networks>) -> Self {
        Handler { networks: networks }
    }
    pub fn route(self, router: &mut Router) -> &mut Router {
        router.get(":network/utxos/:address", self, "utxos")
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Utxo {
    txid: cardano::tx::TxId,
    index: u32,
    address: cardano::address::ExtendedAddr,
    coin: cardano::coin::Coin,
}

impl iron::Handler for Handler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let (_, net) = match common::get_network(req, &self.networks) {
            None => {
                return Ok(Response::with(status::BadRequest));
            }
            Some(x) => x,
        };

        let params = req.extensions.get::<router::Router>().unwrap();
        let address = params.find("address").unwrap();

        let genesis_str = genesisdata::data::get_genesis_data(&net.config.genesis_prev).unwrap();
        let genesis_data = genesisdata::parse::parse(genesis_str.as_bytes());

        let storage = net.storage.read().unwrap();

        let tip = match net.storage.read().unwrap().get_block_from_tag(&tag::HEAD) {
            Err(Error::NoSuchTag) => {
                return Ok(Response::with((status::NotFound, "No Tip To Serve")));
            }
            Err(err) => {
                error!("error while reading block: {:?}", err);
                return Ok(Response::with(status::InternalServerError));
            }
            Ok(block) => {
                let header = block.header();
                BlockRef {
                    hash: header.compute_hash(),
                    parent: header.previous_header(),
                    date: header.blockdate(),
                }
            }
        };

        let chain_state =
            chain_state::restore_chain_state(&storage, &genesis_data, &tip.hash).unwrap();

        let filter_address = match cardano::address::ExtendedAddr::from_str(&address) {
            Ok(addr) => addr,
            Err(_) => return Ok(Response::with((status::BadRequest, "Invalid address"))),
        };

        let utxos = utxos_by_address(&chain_state.utxos, &filter_address);

        let serialized_data = serde_json::to_string(&utxos).unwrap();

        let mut response = Response::with((status::Ok, serialized_data));
        response.headers.set(iron::headers::ContentType::json());

        Ok(response)
    }
}

fn utxos_by_address(
    utxos: &cardano::block::Utxos,
    address: &cardano::address::ExtendedAddr,
) -> Vec<Utxo> {
    utxos
        .iter()
        .filter_map(|(k, v)| {
            if v.address == *address {
                Some(Utxo {
                    txid: k.id,
                    index: k.index,
                    address: v.address.clone(),
                    coin: v.value,
                })
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use cardano::address::ExtendedAddr;
    use cardano::tx::TxOut;
    use cardano::tx::TxoPointer;
    use std::collections::BTreeMap;

    static BASE58_ADDRESS : &str = "DdzFFzCqrhsjcfsReoiHddtt3ih6YusHbNXMTAjCvi5vakqk6sHkXDbMkaYgAbZyiy6hNK4761cF33AaCog93vbwgXGEXKgmA52dhrhJ";
    static BYTES : [u8; 32] = [0u8; 32];

    #[test]
    fn filter_existent_address() {
        let mut utxos = BTreeMap::<TxoPointer, TxOut>::new();

        let filter_address = ExtendedAddr::from_str(&BASE58_ADDRESS).unwrap();
        let txid = cardano::hash::Blake2b256::new(&BYTES);

        utxos.insert(
            TxoPointer { id: txid, index: 0 },
            TxOut {
                address: filter_address.clone(),
                value: cardano::coin::Coin::new(1000).unwrap(),
            },
        );

        let res = utxos_by_address(&utxos, &filter_address);
        assert!(res.contains(&Utxo {
            txid: txid,
            index: 0,
            address: ExtendedAddr::from_str(&BASE58_ADDRESS).unwrap(),
            coin: cardano::coin::Coin::new(1000).unwrap(),
        }));
    }

    #[test]
    fn filter_inexistent_address() {
        let mut utxos = BTreeMap::<TxoPointer, TxOut>::new();

        let filter_address = ExtendedAddr::from_str(&BASE58_ADDRESS).unwrap();

        let txid = cardano::hash::Blake2b256::new(&BYTES);

        let different_address = "DdzFFzCqrhtD4c7dNAyVG29R64GapneLWUbVTECYywUsc6baB7FatGkTGcLWNj3hZnhXJ1ZD43ZBooiUVnVEGQSmEjrxdAP7YUk8dQze";
        utxos.insert(
            TxoPointer { id: txid, index: 1 },
            TxOut {
                address: ExtendedAddr::from_str(&different_address).unwrap(),
                value: cardano::coin::Coin::new(1000).unwrap(),
            },
        );

        let res = utxos_by_address(&utxos, &filter_address);

        assert!(!res.contains(&Utxo {
            txid: txid,
            index: 0,
            address: ExtendedAddr::from_str(&BASE58_ADDRESS).unwrap(),
            coin: cardano::coin::Coin::new(1000).unwrap(),
        }));
    }
}
