use async_raft::{AppData, AppDataResponse, Raft};

use crate::raft::network::RaftRouter;
use crate::raft::storage::MemStore;

pub mod bootstrap;
pub mod member;
pub mod network;
pub mod storage;

/// A concrete Raft type used during testing.
pub type MemRaft = Raft<ClientRequest, ClientResponse, RaftRouter, MemStore>;

pub type ClientRequest = crate::grpc::pb::ClientRequest;
pub type ClientResponse = crate::grpc::pb::ClientResponse;

impl AppData for ClientRequest {}

impl AppDataResponse for ClientResponse {}
