use serde_yaml;

use cardano_storage::config::StorageConfig;
use cardano_storage::{self, Storage};
use exe_common::config::net;
use std::collections::HashSet;
use std::{collections::BTreeMap, num::ParseIntError, sync::Arc, sync::RwLock};
use std::{
    env::{self, home_dir, VarError},
    io,
    path::{Path, PathBuf},
    result,
};

use super::shared_chain_state;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    VarError(VarError),
    YamlError(serde_yaml::Error),
    ParseIntError(ParseIntError),
    StorageError(cardano_storage::Error),
    BlockchainConfigError(&'static str),
}
impl From<VarError> for Error {
    fn from(e: VarError) -> Error {
        Error::VarError(e)
    }
}
impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Error {
        Error::ParseIntError(e)
    }
}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}
impl From<cardano_storage::Error> for Error {
    fn from(e: cardano_storage::Error) -> Error {
        Error::StorageError(e)
    }
}
impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Error {
        Error::YamlError(e)
    }
}

type Result<T> = result::Result<T, Error>;

/// Configuration file for the Wallet CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub root_dir: PathBuf,
    pub port: u16,
    pub network_names: HashSet<String>,
    pub sync: bool,
}

impl Default for Config {
    fn default() -> Self {
        let storage_dir = hermes_path().unwrap().join("networks");
        Config::new(storage_dir, 80)
    }
}

impl Config {
    pub fn new(root_dir: PathBuf, port: u16) -> Self {
        Config {
            root_dir: root_dir,
            port: port,
            network_names: HashSet::new(),
            sync: true,
        }
    }

    pub fn get_networks_dir(&self) -> PathBuf {
        self.root_dir.clone()
    }

    pub fn get_networks(&self) -> Result<Networks> {
        let mut networks = Networks::new();

        use shared_chain_state::SharedChainState;

        for name in &self.network_names {
            let netcfg_dir = self.get_networks_dir().join(name);

            let netcfg = self.get_network_config(name)?;
            let storage = Arc::new(RwLock::new(self.get_storage(name)?));

            //This could slow the startup a bit, but is is the simpler way
            let shared_chain_state =
                SharedChainState::from_storage(storage.clone(), &netcfg).unwrap();

            let network = Network {
                path: netcfg_dir,
                config: netcfg,
                storage: storage,
                shared_chain_state: Arc::new(shared_chain_state),
            };

            networks.insert(name.to_owned(), network);
        }

        Ok(networks)
    }

    pub fn get_network_config<P: AsRef<Path>>(&self, name: P) -> Result<net::Config> {
        let path = self.get_networks_dir().join(name).join("config.yml");
        match net::Config::from_file(&path) {
            None => {
                error!("error while parsing config file: {:?}", path);
                Err(Error::BlockchainConfigError(
                    "error while parsing network config file",
                ))
            }
            Some(cfg) => Ok(cfg),
        }
    }

    pub fn add_network(&mut self, name: &str, netcfg: &net::Config) -> Result<()> {
        let netcfg_dir = self.get_networks_dir().join(name);

        if netcfg_dir.exists() {
            self.network_names.insert(name.to_string());
            return Ok(());
        }

        let storage_config = self.get_storage_config(name);
        let _ = Storage::init(&storage_config)?;

        let network_file = storage_config.get_config_file();
        netcfg.to_file(network_file);

        info!("Added network {}", name);
        self.network_names.insert(name.to_string());
        Ok(())
    }

    pub fn get_storage_config<P: AsRef<Path>>(&self, name: P) -> StorageConfig {
        StorageConfig::new(&self.get_networks_dir().join(name))
    }

    pub fn get_storage<P: AsRef<Path>>(&self, name: P) -> Result<cardano_storage::Storage> {
        let cfg = cardano_storage::Storage::init(&self.get_storage_config(name))?;
        Ok(cfg)
    }
}

#[derive(Clone)]
pub struct Network {
    pub path: PathBuf,
    pub config: net::Config,
    pub storage: Arc<RwLock<cardano_storage::Storage>>,
    pub shared_chain_state: Arc<shared_chain_state::SharedChainState>,
}

/*
impl Network {
    pub fn read_storage_config<'a>(&'a self) -> &'a StorageConfig {
        let r = self.storage.read().unwrap();
        let cfg = &r.config;
        &cfg
    }

    pub fn with_ro_storage(&self) -> &Storage {
        &self.storage.read().unwrap()
    }

    pub fn with_rw_storage(&self) -> &mut Storage {
        &mut self.storage.write().unwrap()
    }
}
*/

pub type Networks = BTreeMap<String, Network>;

/// the environment variable to define where the Hermes files are stores
///
/// this will include all the cardano network you will connect to (mainnet, testnet, ...),
/// the different wallets you will create and all metadata.
pub static HERMES_PATH_ENV: &'static str = "HERMES_PATH";

/// the home directory hidden directory where to find Hermes files.
///
/// # TODO
///
/// This is not standard on windows, set the appropriate setting here
///
pub static HERMES_HOME_PATH: &'static str = ".hermes";

/// get the root directory of all the hermes path
///
/// it is either environment variable `HERMES_PATH` or the `${HOME}/.hermes`
pub fn hermes_path() -> Result<PathBuf> {
    match env::var(HERMES_PATH_ENV) {
        Ok(path) => Ok(PathBuf::from(path)),
        Err(VarError::NotPresent) => match home_dir() {
            None => Err(Error::BlockchainConfigError("no home directory to base hermes root dir. Set `HERMES_PATH' variable environment to fix the problem.")),
            Some(path) => Ok(path.join(HERMES_HOME_PATH))
        },
        Err(err) => Err(Error::VarError(err))
    }
}
