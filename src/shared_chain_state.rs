use cardano::block::chain_state::ChainState;
use cardano::block::{Block, HeaderHash};
use cardano_storage::chain_state::restore_chain_state;
use cardano_storage::{self, Storage};
use exe_common::config::net::Config;
use exe_common::genesisdata;
use std::sync::mpsc::Receiver;
use std::{sync::Arc, sync::RwLock, thread};

pub fn start_update_thread(shared_chain_state: Arc<SharedChainState>, events: Receiver<Event>) {
    let chain_state = shared_chain_state.clone();
    thread::spawn(move || loop {
        let event = match events.recv() {
            Ok(msg) => msg,
            Err(_) => {
                error!("The network refresher hung up");
                break;
            }
        };
        match event {
            Event::NewTip => match chain_state.update() {
                Ok(_) => (),
                Err(err) => warn!("{}", err),
            },
        };
    });
}

pub struct SharedChainState {
    chain_state: RwLock<Arc<ChainState>>,
    storage: Arc<RwLock<Storage>>,
}

pub enum Event {
    NewTip,
}

impl SharedChainState {
    pub fn from_storage(
        storage: Arc<RwLock<Storage>>,
        net: &Config,
    ) -> Result<Self, cardano_storage::Error> {
        info!("Recovering chain_state from storage");
        let chain_state = {
            let storage = storage.read().unwrap();
            let blockid = Self::get_head(&*storage)?;

            let genesis_str = genesisdata::data::get_genesis_data(&net.genesis_prev).unwrap();
            let genesis_data = genesisdata::parse::parse(genesis_str.as_bytes());

            restore_chain_state(&storage, &genesis_data, &blockid).unwrap()
        };
        Ok(Self {
            chain_state: RwLock::new(Arc::new(chain_state)),
            storage: storage,
        })
    }

    pub fn read(&self) -> Arc<ChainState> {
        let rguard = self.chain_state.read().unwrap();
        Arc::clone(&*rguard)
    }

    fn update(&self) -> Result<(), &'static str> {
        let storage = self.storage.read().unwrap();

        let blockid = match Self::get_head(&*storage) {
            Ok(hash) => hash,
            Err(_) => return Err("Couldn't read the head"),
        };

        let new_state = {
            let chain_state = self.chain_state.read().unwrap();
            if chain_state.last_block != blockid {
                match Self::update_from_memory(&chain_state, &blockid, &storage) {
                    Ok(new_chain_state) => Some(new_chain_state),
                    Err(_) => return Err("Verification error"),
                }
            } else {
                None
            }
        };

        //Only take the write lock for the update part
        if let Some(chain_state) = new_state {
            info!("Applying updates to the ChainState");
            let mut write_guard = self.chain_state.write().unwrap();
            *write_guard = Arc::new(chain_state);
        }
        Ok(())
    }

    fn update_from_memory(
        current_state: &ChainState,
        current: &cardano::block::HeaderHash,
        storage: &Storage,
    ) -> Result<ChainState, cardano::block::verify::Error> {
        let mut blocks_to_apply = vec![];
        let mut cursor = current.clone();

        //Find all the new blocks
        while cursor != current_state.last_block {
            //I'm not sure there is something to do on this error
            let block = Self::read_block(&cursor, storage).unwrap();
            let previous = block.header().previous_header();
            // As this should be called frequently,
            // I don't think storing the blocks should be a problem memorywise
            blocks_to_apply.push(block);
            cursor = previous;
        }

        let mut chain_state = current_state.clone();

        for block in blocks_to_apply.iter().rev() {
            let hash = block.header().compute_hash();
            chain_state.verify_block(&hash, &block)?;
        }
        Ok(chain_state)
    }

    fn read_block(
        blockid: &cardano::block::HeaderHash,
        storage: &Storage,
    ) -> Result<Block, cardano_storage::Error> {
        let blockhash = cardano_storage::types::header_to_blockhash(blockid);
        Ok(storage.read_block(&blockhash)?.decode()?)
    }

    fn get_head(storage: &Storage) -> Result<HeaderHash, cardano_storage::Error> {
        storage
            .get_block_from_tag(&cardano_storage::tag::HEAD)
            .and_then(|block| Ok(block.header().compute_hash()))
    }
}
