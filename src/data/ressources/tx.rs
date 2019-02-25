extern crate ensicoin_serializer;
use ensicoin_serializer::types::Hash;
use ensicoin_serializer::{Deserialize, Serialize};

use super::script::Script;

pub struct Outpoint {
    hash: Hash,
    index: u32,
}

impl Serialize for Outpoint {
    fn serialize(&self) -> Vec<u8> {
        let mut v = self.hash.serialize();
        v.append(&mut self.index.serialize());
        v
    }
}

impl Deserialize for Outpoint {
    fn deserialize(
        de: &mut ensicoin_serializer::Deserializer,
    ) -> ensicoin_serializer::Result<Outpoint> {
        let hash = match Hash::deserialize(de) {
            Ok(h) => h,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading Outpoint hash: {}",
                    e
                )));
            }
        };
        let index = match u32::deserialize(de) {
            Ok(i) => i,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading Outpoint index: {}",
                    e
                )));
            }
        };
        Ok(Outpoint { hash, index })
    }
}

pub struct TransactionInput {
    previous_output: Outpoint,
    script: Script,
}

impl Serialize for TransactionInput {
    fn serialize(&self) -> Vec<u8> {
        let mut v = self.previous_output.serialize();
        v.append(&mut self.script.serialize());
        v
    }
}

impl Deserialize for TransactionInput {
    fn deserialize(
        de: &mut ensicoin_serializer::Deserializer,
    ) -> ensicoin_serializer::Result<TransactionInput> {
        let previous_output = match Outpoint::deserialize(de) {
            Ok(p) => p,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading TransactionInput previous_output: {}",
                    e
                )));
            }
        };
        let script = match Script::deserialize(de) {
            Ok(s) => s,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading TransactionInput script: {}",
                    e
                )));
            }
        };
        Ok(TransactionInput {
            previous_output,
            script,
        })
    }
}

pub struct TransactionOutput {
    value: u64,
    script: Script,
}

impl Deserialize for TransactionOutput {
    fn deserialize(
        de: &mut ensicoin_serializer::Deserializer,
    ) -> ensicoin_serializer::Result<TransactionOutput> {
        let value = match u64::deserialize(de) {
            Ok(v) => v,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading TransactionOutput value: {}",
                    e
                )));
            }
        };
        let script = match Script::deserialize(de) {
            Ok(s) => s,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading TransactionOutput script: {}",
                    e
                )));
            }
        };
        Ok(TransactionOutput { value, script })
    }
}

impl Serialize for TransactionOutput {
    fn serialize(&self) -> Vec<u8> {
        let mut v = self.value.serialize();
        v.append(&mut self.script.serialize());
        v
    }
}

pub struct Transaction {
    version: u32,
    flags: Vec<String>,
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
}

impl Serialize for Transaction {
    fn serialize(&self) -> Vec<u8> {
        let mut v = self.version.serialize();
        v.append(&mut self.flags.serialize());
        v.append(&mut self.inputs.serialize());
        v.append(&mut self.outputs.serialize());
        v
    }
}

impl Deserialize for Transaction {
    fn deserialize(
        de: &mut ensicoin_serializer::Deserializer,
    ) -> ensicoin_serializer::Result<Transaction> {
        let version = match u32::deserialize(de) {
            Ok(v) => v,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading Transaction version: {}",
                    e
                )));
            }
        };
        let flags: Vec<String> = match Vec::deserialize(de) {
            Ok(f) => f,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading Transaction flags: {}",
                    e
                )));
            }
        };
        let inputs: Vec<TransactionInput> = match Vec::deserialize(de) {
            Ok(i) => i,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading Transaction inputs: {}",
                    e
                )));
            }
        };
        let outputs: Vec<TransactionOutput> = match Vec::deserialize(de) {
            Ok(o) => o,
            Err(e) => {
                return Err(ensicoin_serializer::Error::Message(format!(
                    "Error in reading Transaction outputs: {}",
                    e
                )));
            }
        };
        Ok(Transaction {
            version,
            flags,
            inputs,
            outputs,
        })
    }
}
