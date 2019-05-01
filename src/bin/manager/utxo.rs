use crate::data::ressources::Script;
use crate::data::ressources::UtxoData;
use crate::data::Outpoint;
use crate::data::Transaction;
use bytes::BytesMut;

use ensicoin_serializer::{Deserialize, Serialize, Sha256Result};

pub enum Error {
    DatabaseError(sled::Error<()>),
    ParseError(ensicoin_serializer::Error),
    NoValueFound,
}

impl From<sled::Error<()>> for Error {
    fn from(error: sled::Error<()>) -> Error {
        Error::DatabaseError(error)
    }
}

impl From<ensicoin_serializer::Error> for Error {
    fn from(error: ensicoin_serializer::Error) -> Error {
        Error::ParseError(error)
    }
}

pub struct UtxoManager {
    database: sled::Db,
}

impl UtxoManager {
    pub fn new(data_dir: &std::path::Path) -> UtxoManager {
        let mut utxo_dir = std::path::PathBuf::new();
        utxo_dir.push(data_dir);
        utxo_dir.push("utxo");
        let database = sled::Db::start_default(utxo_dir).unwrap();
        UtxoManager { database }
    }

    pub fn register(
        &self,
        tx: &Transaction,
        hash: &[u8],
        coin_base: bool,
        block_height: u32,
    ) -> Result<(), Error> {
        for (i, output) in tx.get_outputs().iter().enumerate() {
            let data = UtxoData {
                script: output.get_script().clone(),
                value: output.get_value().clone(),
                block_height,
                coin_base,
            }
            .serialize();
            let outpoint = Outpoint {
                hash: Sha256Result::clone_from_slice(hash),
                index: (i as u32),
            };
            self.database.set(outpoint.serialize(), data.to_vec())?;
        }
        Ok(())
    }
    pub fn exists(&self, utxo: &Outpoint) -> Result<bool, Error> {
        Ok(match self.database.get(utxo.serialize())? {
            Some(_) => true,
            None => false,
        })
    }

    pub fn get(&self, utxo: &Outpoint) -> Result<UtxoData, Error> {
        match self.database.get(utxo.serialize())? {
            Some(x) => {
                let mut de = ensicoin_serializer::Deserializer::new(BytesMut::from(&*x));
                Ok(UtxoData::deserialize(&mut de)?)
            }
            None => Err(Error::NoValueFound),
        }
    }

    pub fn delete(&self, utxo: &Outpoint) -> Result<(), Error> {
        self.database.del(utxo.serialize())?;
        Ok(())
    }

    pub fn link(&self, linkedtx: &mut crate::data::ressources::LinkedTransaction) {
        for parent in linkedtx.unknown().clone() {
            if let Ok(utxo) = self.get(&parent) {
                linkedtx.add_dependency(
                    parent.clone(),
                    crate::data::ressources::Dependency {
                        data: utxo,
                        dep_type: crate::data::ressources::DependencyType::Block,
                    },
                );
            }
        }
    }
}
