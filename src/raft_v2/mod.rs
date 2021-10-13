use async_raft::raft::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    VoteRequest, VoteResponse,
};
use async_raft::{AppData, AppDataResponse, Raft};
use tokio::sync::oneshot;

use crate::raft_v2::network::RaftRouter;
use crate::raft_v2::storage::MemStore;

pub mod bootstrap;
pub mod interface;
pub mod member;
pub mod network;
pub mod storage;

/// A concrete Raft type used during testing.
pub type MemRaft = Raft<ClientRequest, ClientResponse, RaftRouter, MemStore>;

#[derive(Debug)]
pub enum RaftRequest {
    VoteRequest(VoteRequest, oneshot::Sender<VoteResponse>),
    AppendEntriesRequest(
        AppendEntriesRequest<ClientRequest>,
        oneshot::Sender<AppendEntriesResponse>,
    ),
    InstallSnapshotRequest(
        InstallSnapshotRequest,
        oneshot::Sender<InstallSnapshotResponse>,
    ),
}

/// The application data request type which the `MemStore` works with.
///
/// Conceptually, for demo purposes, this represents an update to a client's status info,
/// returning the previously recorded status.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientRequest {
    /// The ID of the client which has sent the request.
    pub client: String,
    /// The serial number of this request.
    pub serial: u64,
    /// A string describing the status of the client. For a real application, this should probably
    /// be an enum representing all of the various types of requests / operations which a client
    /// can perform.
    pub status: String,
}

impl AppData for ClientRequest {}

/// The application data response type which the `MemStore` works with.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientResponse(Option<String>);

impl AppDataResponse for ClientResponse {}
