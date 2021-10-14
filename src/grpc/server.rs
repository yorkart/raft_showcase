use std::net::SocketAddr;

use tonic::transport::Server;

use crate::grpc::pb;
use crate::grpc::utils::{
    append_entries_request_to_raft, append_entries_response_to_pb,
    install_snapshot_request_to_raft, install_snapshot_response_to_pb, vote_request_to_raft,
    vote_response_to_pb,
};
use crate::raft::MemRaft;

pub struct RaftNetworkServer {
    raft: MemRaft,
}

impl RaftNetworkServer {
    fn new(raft: MemRaft) -> Self {
        RaftNetworkServer { raft }
    }
}

#[tonic::async_trait]
impl pb::raft_network_server::RaftNetwork for RaftNetworkServer {
    async fn append_entries(
        &self,
        request: tonic::Request<pb::AppendEntriesRequest>,
    ) -> Result<tonic::Response<pb::AppendEntriesResponse>, tonic::Status> {
        let req = append_entries_request_to_raft(request.get_ref().clone());
        let resp = self
            .raft
            .append_entries(req)
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        Ok(tonic::Response::new(append_entries_response_to_pb(resp)))
    }

    async fn install_snapshot(
        &self,
        request: tonic::Request<pb::InstallSnapshotRequest>,
    ) -> Result<tonic::Response<pb::InstallSnapshotResponse>, tonic::Status> {
        let req = install_snapshot_request_to_raft(request.get_ref().clone());
        let resp = self
            .raft
            .install_snapshot(req)
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        Ok(tonic::Response::new(install_snapshot_response_to_pb(resp)))
    }

    async fn vote(
        &self,
        request: tonic::Request<pb::VoteRequest>,
    ) -> Result<tonic::Response<pb::VoteResponse>, tonic::Status> {
        let req = vote_request_to_raft(request.get_ref().clone());
        let resp = self
            .raft
            .vote(req)
            .await
            .map_err(|e| tonic::Status::internal(format!("{:?}", e)))?;
        Ok(tonic::Response::new(vote_response_to_pb(resp)))
    }
}

pub async fn serve(raft: MemRaft, bind_addr: SocketAddr) -> anyhow::Result<()> {
    // let addr = "[::1]:50051".parse().unwrap();
    let server = RaftNetworkServer::new(raft);

    let svc = pb::raft_network_server::RaftNetworkServer::new(server);

    Server::builder().add_service(svc).serve(bind_addr).await?;

    Ok(())
}
