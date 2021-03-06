use crate::Error;
use bytes::Bytes;
use cookie_factory::{bytes::be_u16, combinator::slice, sequence::tuple, SerializeFn};
use ensicoin_messages::{
    message::{Address, GetBlocks, InvVect, Message},
    resource::{Block, Transaction},
};
use ensicoin_serializer::{Deserialize, Deserializer};
use std::io::Write;
use tokio::sync::mpsc;

#[derive(Eq, PartialEq)]
pub enum Source {
    Connection(RemoteIdentity),
    Server,
    #[cfg(feature = "grpc")]
    RPC,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Source::Connection(r) => format!("connetion [{}]", r.id),
                #[cfg(feature = "grpc")]
                Source::RPC => "RPC".to_string(),
                Source::Server => "Server".to_string(),
            }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct RemoteIdentity {
    pub id: u64,
    pub peer: Peer,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Hash, Copy)]
pub struct Peer {
    pub ip: [u8; 16],
    pub port: u16,
}

impl From<std::net::SocketAddr> for Peer {
    fn from(socket: std::net::SocketAddr) -> Self {
        let port = socket.port();
        let ip = match socket.ip() {
            std::net::IpAddr::V4(i) => i.to_ipv6_mapped().octets(),
            std::net::IpAddr::V6(i) => i.octets(),
        };
        Peer { port, ip }
    }
}

impl std::str::FromStr for Peer {
    type Err = std::net::AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let socket: std::net::SocketAddr = s.parse()?;
        Ok(Peer::from(socket))
    }
}

impl Deserialize for Peer {
    fn deserialize(de: &mut Deserializer) -> ensicoin_serializer::Result<Self> {
        let ip_bytes = de.extract_bytes(16)?;
        let mut ip = [0; 16];
        for (i, b) in ip_bytes.iter().enumerate() {
            ip[i] = *b;
        }
        let port = u16::deserialize(de)?;

        Ok(Peer { ip, port })
    }
}

pub fn fn_peer<'c, 'a: 'c, W: Write + 'c>(peer: &'a Peer) -> impl SerializeFn<W> + 'c {
    tuple((slice(peer.ip), be_u16(peer.port)))
}

pub struct ConnectionMessage {
    pub content: ConnectionMessageContent,
    pub source: Source,
}

/// Messages sent to the server by the connections for example
pub enum ConnectionMessageContent {
    Disconnect(Error, String),
    Clean(u64),
    CheckInv(Vec<InvVect>),
    Retrieve(Vec<InvVect>),
    SyncBlocks(GetBlocks),
    NewTransaction(Box<Transaction>),
    NewBlock(Box<Block>),
    Connect(std::net::SocketAddr),
    NewConnection(tokio::net::TcpStream),
    Register(mpsc::Sender<ServerMessage>, RemoteIdentity),
    RetrieveAddr,
    ConnectionFailed(std::net::SocketAddr),
    NewAddr(Vec<Address>),
    VerifiedAddr(Address),
    Quit,
}

/// Messages Sent From the server
#[derive(Clone)]
pub enum ServerMessage {
    Terminate(crate::network::TerminationReason),
    SendMsg(Message),
}

impl std::fmt::Display for ConnectionMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} from {}", self.content, self.source)
    }
}

impl std::fmt::Display for ConnectionMessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ConnectionMessageContent::Disconnect(_, _) => "Disconnect",
                ConnectionMessageContent::CheckInv(_) => "CheckInv",
                ConnectionMessageContent::Retrieve(_) => "Retrieve",
                ConnectionMessageContent::SyncBlocks(_) => "SyncBlocks",
                ConnectionMessageContent::NewTransaction(_) => "NewTx",
                ConnectionMessageContent::Connect(_) => "Connect",
                ConnectionMessageContent::NewConnection(_) => "NewConnection",
                ConnectionMessageContent::Register(_, _) => "Register",
                ConnectionMessageContent::NewBlock(_) => "NewBlock",
                ConnectionMessageContent::Clean(_) => "Clean",
                ConnectionMessageContent::RetrieveAddr => "RetrieveAddr",
                ConnectionMessageContent::NewAddr(_) => "NewAddr",
                ConnectionMessageContent::VerifiedAddr(_) => "VerifiedAddr",
                ConnectionMessageContent::ConnectionFailed(_) => "ConnectionFailed",
                ConnectionMessageContent::Quit => "Quit",
            }
        )
    }
}

#[cfg(feature = "grpc")]
#[derive(Clone)]
pub enum BroadcastMessage {
    BestBlock(ensicoin_messages::resource::Block),
    Quit,
}
