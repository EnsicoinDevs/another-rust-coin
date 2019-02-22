extern crate ensicoin_serializer;
use crate::network;
use ensicoin_serializer::Serialize;

#[derive(PartialEq, Eq, Debug)]
pub enum MessageType {
    Whoami,
    WhoamiAck,
    Unknown(String),
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MessageType::Whoami => "Whoami".to_string(),
                MessageType::WhoamiAck => "WhoamiAck".to_string(),
                MessageType::Unknown(s) => {
                    format!("Unknown: {}", s).trim_matches('\x00').to_string()
                }
            }
        )
    }
}

pub trait Message: Serialize {
    fn message_string() -> [u8; 12];
    fn message_type() -> MessageType;
    fn raw_bytes(&self) -> Result<(MessageType, Vec<u8>), network::Error> {
        let magic: u32 = 422021;
        let message_string = Self::message_string();
        let mut payload = self.serialize();
        let payload_length: u64 = payload.len() as u64;

        let mut v = Vec::new();
        v.append(&mut magic.serialize());
        v.extend_from_slice(&message_string);
        v.append(&mut payload_length.serialize());
        v.append(&mut payload);
        Ok((Self::message_type(), v))
    }
}
