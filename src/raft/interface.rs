use async_raft::raft::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    VoteRequest, VoteResponse,
};
use async_raft::{AppData, NodeId};

use crate::grpc::server::serve;
use crate::raft::member::MemberGroup;
use crate::raft::MemRaft;

#[async_trait]
pub trait RemoteInterface<D: AppData> {
    async fn vote_request(&self, request: VoteRequest) -> anyhow::Result<VoteResponse>;
    async fn append_entries_request(
        &self,
        request: AppendEntriesRequest<D>,
    ) -> anyhow::Result<AppendEntriesResponse>;
    async fn install_snapshot_request(
        &self,
        rpc: InstallSnapshotRequest,
    ) -> anyhow::Result<InstallSnapshotResponse>;
}

pub async fn run_service_interface(node_id: NodeId, raft: MemRaft, member_group: MemberGroup) {
    let bind_addr = member_group.member(node_id).unwrap().as_socket_addr();
    tokio::spawn(serve(raft, bind_addr));
}
