pub mod node {
    include!(concat!(env!("OUT_DIR"), "/ensicoin_rpc.node.rs"));
}

pub mod rpc_types {
    include!(concat!(env!("OUT_DIR"), "/ensicoin_rpc.rs"));
}

pub use rpc_types::{Block, BlockTemplate, Tx};

use node::{
    server, GetBestBlockHashReply, GetBestBlockHashRequest, GetBlockByHashReply,
    GetBlockByHashRequest, GetBlockTemplateReply, GetBlockTemplateRequest, GetInfoReply,
    GetInfoRequest, GetTxByHashReply, GetTxByHashRequest, PublishRawBlockReply,
    PublishRawBlockRequest, PublishRawTxReply, PublishRawTxRequest,
};

use crate::constants::{IMPLEMENTATION, VERSION};
use crate::data::intern_messages::ConnectionMessage;
use crate::manager::{Blockchain, Mempool};
use ensicoin_serializer::{Deserialize, Deserializer};
use futures::{future, stream, Future, Sink, Stream};
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use tower_grpc::{Request, Response, Streaming};
use tower_hyper::server::{Http, Server};

#[derive(Clone)]
pub struct RPCNode {
    state: Arc<State>,
}

struct State {
    mempool: Arc<RwLock<Mempool>>,
    blockchain: Arc<RwLock<Blockchain>>,
    server_sender: futures::sync::mpsc::Sender<ConnectionMessage>,
}

impl State {
    fn new(
        mempool: Arc<RwLock<Mempool>>,
        blockchain: Arc<RwLock<Blockchain>>,
        server_sender: futures::sync::mpsc::Sender<ConnectionMessage>,
    ) -> State {
        State {
            mempool,
            blockchain,
            server_sender,
        }
    }
}

impl RPCNode {
    pub fn new(
        mempool: Arc<RwLock<Mempool>>,
        blockchain: Arc<RwLock<Blockchain>>,
        sender: futures::sync::mpsc::Sender<ConnectionMessage>,
        bind_address: &str,
        port: u16,
    ) -> Box<Future<Item = (), Error = ()> + Send> {
        let handler = RPCNode {
            state: Arc::new(State::new(mempool, blockchain, sender)),
        };
        let new_service = server::NodeServer::new(handler);

        let mut server = Server::new(new_service);
        let http = Http::new().http2_only(true).clone();

        let addr = format!("{}:{}", bind_address, port).parse().unwrap();
        let bind = TcpListener::bind(&addr).unwrap();

        info!("Started gRPC server");

        let serve = bind
            .incoming()
            .for_each(move |sock| {
                if let Err(e) = sock.set_nodelay(true) {
                    return Err(e);
                }
                let serve = server.serve_with(sock, http.clone());
                tokio::spawn(serve.map_err(|e| error!("[gRPC] h2 error: {}", e)));
                Ok(())
            })
            .map_err(|e| error!("[gRPC] accept error: {}", e));
        Box::new(serve)
    }
}

impl node::server::Node for RPCNode {
    type GetInfoFuture = future::FutureResult<Response<GetInfoReply>, tower_grpc::Status>;

    fn get_info(&mut self, _request: Request<GetInfoRequest>) -> Self::GetInfoFuture {
        info!("[grpc] GetInfo");
        let response = Response::new(GetInfoReply {
            implementation: IMPLEMENTATION.to_string(),
            protocol_version: VERSION,
        });
        future::ok(response)
    }

    type PublishRawTxFuture =
        Box<Future<Item = Response<PublishRawTxReply>, Error = tower_grpc::Status> + Send>;

    fn publish_raw_tx(
        &mut self,
        request: Request<Streaming<PublishRawTxRequest>>,
    ) -> Self::PublishRawTxFuture {
        info!("[grpc] PublishRawTx");
        let sender = self.state.server_sender.clone();
        let response = request
            .into_inner()
            .map_err(|e| {
                warn!("[grpc] error in PublishRawTx: {}", e);
                e
            })
            //.zip(sender)
            .for_each(move |raw_tx_msg| {
                let mut de = Deserializer::new(bytes::BytesMut::from(raw_tx_msg.raw_tx));
                let tx = match ensicoin_messages::resource::Transaction::deserialize(&mut de) {
                    Ok(tx) => tx,
                    Err(e) => {
                        warn!("[grpc] Error reading tx: {}", e);
                        return Err(tower_grpc::Status::new(
                            tower_grpc::Code::InvalidArgument,
                            format!("Error parsing: {}", e),
                        ));
                    }
                };
                tokio::spawn(
                    sender
                        .clone()
                        .send(ConnectionMessage::NewTransaction(tx))
                        .map_err(|e| warn!("[grpc] can't contact server: {}", e))
                        .map(|_| ()),
                );
                Ok(())
            })
            .map(|_| Response::new(PublishRawTxReply {}));
        Box::new(response)
    }

    type PublishRawBlockFuture =
        Box<Future<Item = Response<PublishRawBlockReply>, Error = tower_grpc::Status> + Send>;

    fn publish_raw_block(
        &mut self,
        request: Request<Streaming<PublishRawBlockRequest>>,
    ) -> Self::PublishRawBlockFuture {
        info!("[grpc] PublishRawBlock");
        let sender = self.state.server_sender.clone();
        let response = request
            .into_inner()
            .map_err(|e| {
                warn!("[grpc] error in PublishRawBlock: {}", e);
                e
            })
            .for_each(move |raw_blk_msg| {
                let mut de = Deserializer::new(bytes::BytesMut::from(raw_blk_msg.raw_block));
                let block = match ensicoin_messages::resource::Block::deserialize(&mut de) {
                    Ok(b) => b,
                    Err(e) => {
                        warn!("[grpc] Error reading block: {}", e);
                        return Err(tower_grpc::Status::new(
                            tower_grpc::Code::InvalidArgument,
                            format!("Error parsing: {}", e),
                        ));
                    }
                };
                tokio::spawn(
                    sender
                        .clone()
                        .send(ConnectionMessage::NewBlock(block))
                        .map_err(|e| warn!("[grpc] can't contact server: {}", e))
                        .map(|_| ()),
                );
                Ok(())
            })
            .map(|_| Response::new(PublishRawBlockReply {}));
        Box::new(response)
    }

    type GetBlockTemplateStream =
        Box<Stream<Item = GetBlockTemplateReply, Error = tower_grpc::Status> + Send>;
    type GetBlockTemplateFuture =
        future::FutureResult<Response<Self::GetBlockTemplateStream>, tower_grpc::Status>;

    fn get_block_template(
        &mut self,
        request: Request<GetBlockTemplateRequest>,
    ) -> Self::GetBlockTemplateFuture {
        unimplemented!()
    }

    type GetBestBlockHashFuture =
        future::FutureResult<Response<GetBestBlockHashReply>, tower_grpc::Status>;
    fn get_best_block_hash(
        &mut self,
        request: Request<GetBestBlockHashRequest>,
    ) -> Self::GetBestBlockHashFuture {
        unimplemented!()
    }

    type GetBlockByHashFuture =
        future::FutureResult<Response<GetBlockByHashReply>, tower_grpc::Status>;
    fn get_block_by_hash(
        &mut self,
        request: Request<GetBlockByHashRequest>,
    ) -> Self::GetBlockByHashFuture {
        unimplemented!()
    }

    type GetTxByHashFuture = future::FutureResult<Response<GetTxByHashReply>, tower_grpc::Status>;
    fn get_tx_by_hash(&mut self, request: Request<GetTxByHashRequest>) -> Self::GetTxByHashFuture {
        unimplemented!()
    }
}
