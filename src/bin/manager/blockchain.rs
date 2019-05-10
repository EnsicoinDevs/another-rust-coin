use ensicoin_messages::resource::Block;
use ensicoin_serializer::{Deserialize, Serialize, Sha256Result};

pub struct Blockchain {
    database: sled::Db,
    reverse_chain: sled::Db,
    spent_tx: sled::Db,
}

impl Blockchain {
    pub fn new(data_dir: &std::path::Path) -> Blockchain {
        let mut blockchain_dir = std::path::PathBuf::new();
        blockchain_dir.push(data_dir);
        blockchain_dir.push("blockchain");
        let database = sled::Db::start_default(blockchain_dir).unwrap();

        let mut rev_dir = std::path::PathBuf::new();
        rev_dir.push(data_dir);
        rev_dir.push("reverse_chain");
        let reverse_chain = sled::Db::start_default(rev_dir).unwrap();

        let mut spent_tx_dir = std::path::PathBuf::new();
        spent_tx_dir.push(data_dir);
        spent_tx_dir.push("spent_tx");
        let spent_tx = sled::Db::start_default(spent_tx_dir).unwrap();

        Blockchain {
            database,
            reverse_chain,
            spent_tx,
        }
    }

    pub fn add_block(&mut self, block: &Block) -> Result<(), sled::Error<()>> {
        let raw_block = block.serialize().to_vec();
        let hash = block.double_hash();
        //let utxo = block.utxo().serialize().to_vec();
        let spent_tx = Vec::new();
        self.database.set(hash, raw_block.clone())?;
        self.reverse_chain.set(block.header.prev_block, raw_block)?;
        self.spent_tx.set(hash, spent_tx)?;
        Ok(())
    }
}