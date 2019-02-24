extern crate ensicoin_serializer;

use ensicoin_serializer::types::Hash;
use ensicoin_serializer::{Deserialize, Result, Serialize};

pub struct GetBlocks {
    block_locator: Vec<Hash>,
    stop_hash: Hash,
}

impl Deserialize for GetBlocks {
    fn deserialize(de: &mut ensicoin_serializer::Deserializer) -> Result<GetBlocks> {
        let block_locator = match Vec::deserialize(de) {
            Ok(inventory) => inventory,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading GetBlocks block_locator: {}",
                    e
                )));
            }
        };
        let stop_hash = match Hash::deserialize(de) {
            Ok(hash) => hash,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading GetBlocks stop_hash: {}",
                    e
                )));
            }
        };
        Ok(GetBlocks {
            block_locator,
            stop_hash,
        })
    }
}

impl Serialize for GetBlocks {
    fn serialize(&self) -> Vec<u8> {
        let mut v = self.block_locator.serialize();
        v.append(&mut self.stop_hash.serialize());
        v
    }
}