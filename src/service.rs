use super::config::{Config, Network, Networks};
use super::handlers;
use exe_common::config::net;
use exe_common::{genesisdata, sync};
use iron;
use router::Router;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use std::sync::mpsc::{channel, Sender};

use cardano::config::GenesisData;
use cardano_storage::Storage;
use exe_common::network::Api;
use exe_common::network::Peer;

use super::shared_chain_state;

pub fn start(cfg: Config) {
    let networks = Arc::new(match cfg.get_networks() {
        Err(err) => panic!("Unable to get networks: {:?}", err),
        Ok(nets) => nets,
    });
    let _refreshers = if cfg.sync {
        Some(start_networks_refreshers(networks.clone()))
    } else {
        None
    };

    let _server = start_http_server(&cfg, networks);

    // XXX: consider installing a signal handler to initiate a graceful shutdown here
    // XXX: after initiating shutdown, do `refresher.join()` and something similar for `server`.
}

fn start_http_server(cfg: &Config, networks: Arc<Networks>) -> iron::Listening {
    let mut router = Router::new();
    handlers::block::Handler::new(networks.clone()).route(&mut router);
    handlers::genesis::Handler::new(networks.clone()).route(&mut router);
    handlers::pack::Handler::new(networks.clone()).route(&mut router);
    handlers::epoch::Handler::new(networks.clone()).route(&mut router);
    handlers::tip::Handler::new(networks.clone()).route(&mut router);
    handlers::tx::Handler::new(networks.clone()).route(&mut router);
    handlers::chain_state::Handler::new(networks.clone()).route(&mut router);
    handlers::chain_state_delta::Handler::new(networks.clone()).route(&mut router);
    handlers::utxos::Handler::new(networks.clone()).route(&mut router);
    info!("listening to port {}", cfg.port);
    iron::Iron::new(router)
        .http(format!("0.0.0.0:{}", cfg.port))
        .expect("start http server")
}

// TODO: make this a struct which receives a shutdown message on a channel and then wraps itself up
fn start_networks_refreshers(networks: Arc<Networks>) -> Vec<thread::JoinHandle<()>> {
    let mut threads = vec![];
    for (label, net) in networks.iter() {
        let label = label.to_owned();
        let mut net = net.clone();
        threads.push(thread::spawn(move || {
            loop {
                refresh_network(&label, &mut net);
                // In case of an error, wait a while before retrying.
                thread::sleep(Duration::from_secs(10));
            }
        }));
    }
    threads
}

// XXX: how do we want to report partial failures?
fn refresh_network(label: &str, net: &mut Network) {
    info!("Refreshing network {:?}", label);

    let netcfg_file = net.storage.read().unwrap().config.get_config_file();
    let net_cfg = net::Config::from_file(&netcfg_file).expect("no network config present");

    let genesis_data = {
        let genesis_data = genesisdata::data::get_genesis_data(&net_cfg.genesis_prev)
            .expect("genesis data not found");
        genesisdata::parse::parse(genesis_data.as_bytes())
    };

    let (tx, rx) = channel::<shared_chain_state::Event>();

    shared_chain_state::start_update_thread(net.shared_chain_state.clone(), rx);

    net_sync(
        &mut sync::get_peer(&label, &net_cfg, true),
        &net_cfg,
        &genesis_data,
        net.storage.clone(),
        tx,
    )
    .unwrap_or_else(|err| warn!("Sync failed: {:?}", err));
}

fn net_sync(
    net: &mut Peer,
    net_cfg: &net::Config,
    genesis_data: &GenesisData,
    storage: Arc<RwLock<Storage>>,
    tx: Sender<shared_chain_state::Event>,
) -> Result<(), exe_common::network::error::Error> {
    let mut tip_header = net.get_tip()?;

    loop {
        sync::net_sync(net, &net_cfg, &genesis_data, storage.clone(), true)?;

        match tx.send(shared_chain_state::Event::NewTip) {
            Ok(_) => (),
            Err(_) => warn!("There is no receiver for the NewTip message"),
        }

        tip_header = net.wait_for_new_tip(&tip_header.compute_hash())?;
    }
}
