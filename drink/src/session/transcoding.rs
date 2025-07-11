use std::{collections::BTreeMap, sync::Arc};

use contract_transcode::ContractMessageTranscoder;
use ink_primitives::H160;

pub struct TranscoderRegistry {
    transcoders: BTreeMap<H160, Arc<ContractMessageTranscoder>>,
}

impl TranscoderRegistry {
    pub fn new() -> Self {
        Self {
            transcoders: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, contract: H160, transcoder: &Arc<ContractMessageTranscoder>) {
        self.transcoders.insert(contract, Arc::clone(transcoder));
    }

    pub fn get(&self, contract: &H160) -> Option<Arc<ContractMessageTranscoder>> {
        self.transcoders.get(contract).map(Arc::clone)
    }
}
