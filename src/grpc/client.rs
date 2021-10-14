use async_raft::raft::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    VoteRequest, VoteResponse,
};
use http::Uri;
use tonic::transport::Channel;

use crate::grpc::pb;
use crate::grpc::pb::raft_network_client::RaftNetworkClient;
use crate::grpc::utils::{
    append_entries_request_to_pb, append_entries_response_to_raft, install_snapshot_request_to_pb,
    install_snapshot_response_to_raft, vote_request_to_pb, vote_response_to_raft,
};
use crate::raft_v3::ClientRequest;

pub struct GRpcClient {
    uri: Uri,
    client: Option<RaftNetworkClient<Channel>>,
}

impl GRpcClient {
    pub fn new(uri: Uri) -> Self {
        // let uri = Uri::try_from(remote_addr).unwrap();
        GRpcClient { uri, client: None }
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let endpoint = Channel::builder(self.uri.clone());
        let channel = endpoint.connect().await?;
        let client = RaftNetworkClient::new(channel);

        self.client = Some(client);
        Ok(())
    }

    pub async fn connection_check(&mut self) -> anyhow::Result<()> {
        if self.client.is_none() {
            self.connect().await.map_err(|e| {
                error!("connect error: {}", e);
                e
            })?;
        }

        Ok(())
    }

    pub async fn append_entries(
        &mut self,
        request: AppendEntriesRequest<ClientRequest>,
    ) -> anyhow::Result<AppendEntriesResponse> {
        let req = append_entries_request_to_pb(request);
        let response = self.append_entries_raw(req).await?;
        Ok(append_entries_response_to_raft(response))
    }

    async fn append_entries_raw(
        &mut self,
        request: pb::AppendEntriesRequest,
    ) -> anyhow::Result<pb::AppendEntriesResponse> {
        self.connection_check().await?;

        let request = tonic::Request::new(request);
        let response = self
            .client
            .as_mut()
            .unwrap()
            .append_entries(request)
            .await
            .map_err(|e| anyhow!(e))?;
        Ok(response.into_inner())
    }

    pub async fn install_snapshot(
        &mut self,
        request: InstallSnapshotRequest,
    ) -> anyhow::Result<InstallSnapshotResponse> {
        let req = install_snapshot_request_to_pb(request);
        let response = self.install_snapshot_raw(req).await?;
        Ok(install_snapshot_response_to_raft(response))
    }

    async fn install_snapshot_raw(
        &mut self,
        request: pb::InstallSnapshotRequest,
    ) -> anyhow::Result<pb::InstallSnapshotResponse> {
        self.connection_check().await?;

        let request = tonic::Request::new(request);
        let response = self
            .client
            .as_mut()
            .unwrap()
            .install_snapshot(request)
            .await
            .map_err(|e| anyhow!(e))?;
        Ok(response.into_inner())
    }

    pub async fn vote(&mut self, request: VoteRequest) -> anyhow::Result<VoteResponse> {
        let req = vote_request_to_pb(request);
        let response = self.vote_raw(req).await?;
        Ok(vote_response_to_raft(response))
    }

    pub async fn vote_raw(&mut self, request: pb::VoteRequest) -> anyhow::Result<pb::VoteResponse> {
        self.connection_check().await?;

        let request = tonic::Request::new(request);
        let response = self
            .client
            .as_mut()
            .unwrap()
            .vote(request)
            .await
            .map_err(|e| anyhow!(e))?;
        Ok(response.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use http::Uri;

    use crate::grpc::client::GRpcClient;

    #[tokio::test]
    async fn t() {
        let uri = Uri::from_static("http://127.0.0.1:35502");
        let mut client = GRpcClient::new(uri);
        client.connect().await.unwrap();

        println!("end");
    }
}
