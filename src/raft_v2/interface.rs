use std::collections::HashMap;

use async_raft::raft::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    VoteRequest, VoteResponse,
};
use async_raft::{AppData, NodeId};
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};

use crate::raft_v2::{ClientRequest, MemRaft, RaftRequest};

lazy_static! {
    pub static ref SENDERS: Mutex<HashMap<NodeId, mpsc::Sender<RaftRequest>>> =
        Mutex::new(HashMap::new());
}

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

#[async_trait]
pub trait ServiceInterface<D: AppData> {
    async fn on_vote_request(&self, request: VoteRequest) -> anyhow::Result<VoteResponse>;
    async fn on_append_entries_request(
        &self,
        request: AppendEntriesRequest<D>,
    ) -> anyhow::Result<AppendEntriesResponse>;
    async fn on_install_snapshot_request(
        &self,
        request: InstallSnapshotRequest,
    ) -> anyhow::Result<InstallSnapshotResponse>;
}

pub async fn run_service_interface(node_id: NodeId, raft: MemRaft) {
    let (sender, receiver) = mpsc::channel::<RaftRequest>(1);

    {
        let mut lock = SENDERS.lock().await;
        if lock.contains_key(&node_id) {
            panic!("node {} has publish", node_id);
        }

        lock.insert(node_id, sender);
    }

    tokio::spawn(interface_service(node_id, raft, receiver));
}

async fn interface_service(
    node_id: NodeId,
    raft: MemRaft,
    mut receiver: mpsc::Receiver<RaftRequest>,
) {
    let interface_event = ChannelServiceInterface::new(raft);

    while let Some(request) = receiver.recv().await {
        debug!("node {} handle event: {:?}", node_id, request);
        match interface_service_event(&interface_event, request).await {
            Ok(()) => {}
            Err(e) => error!("failure {}", e),
        }
    }
}

async fn interface_service_event(
    interface_event: &ChannelServiceInterface,
    request: RaftRequest,
) -> anyhow::Result<()> {
    match request {
        RaftRequest::VoteRequest(request, tx) => {
            let response = interface_event.on_vote_request(request).await?;
            tx.send(response)
                .map_err(|_e| anyhow!("channel disconnection"))?;
            Ok(())
        }
        RaftRequest::AppendEntriesRequest(request, tx) => {
            let response = interface_event.on_append_entries_request(request).await?;
            tx.send(response)
                .map_err(|_e| anyhow!("channel disconnection"))?;
            Ok(())
        }
        RaftRequest::InstallSnapshotRequest(request, tx) => {
            let response = interface_event.on_install_snapshot_request(request).await?;
            tx.send(response)
                .map_err(|_e| anyhow!("channel disconnection"))?;
            Ok(())
        }
    }
}

pub struct ChannelRemoteInterface {
    node_id: NodeId,
}

impl ChannelRemoteInterface {
    pub fn new(node_id: NodeId) -> Self {
        ChannelRemoteInterface { node_id }
    }

    pub async fn request(&self, request: RaftRequest) -> anyhow::Result<()> {
        let lock = SENDERS.lock().await;
        let sender = lock.get(&self.node_id).ok_or(anyhow!("not found"))?;
        sender
            .send(request)
            .await
            .map_err(|_e| anyhow!("channel disconnection"))?;

        Ok(())
    }
}

#[async_trait]
impl RemoteInterface<ClientRequest> for ChannelRemoteInterface {
    async fn vote_request(&self, request: VoteRequest) -> anyhow::Result<VoteResponse> {
        let (sender, receiver) = oneshot::channel();
        let request = RaftRequest::VoteRequest(request, sender);

        self.request(request).await?;

        let response = receiver.await?;
        Ok(response)
    }

    async fn append_entries_request(
        &self,
        request: AppendEntriesRequest<ClientRequest>,
    ) -> anyhow::Result<AppendEntriesResponse> {
        let (sender, receiver) = oneshot::channel();
        let request = RaftRequest::AppendEntriesRequest(request, sender);

        self.request(request).await?;

        let response = receiver.await?;
        Ok(response)
    }

    async fn install_snapshot_request(
        &self,
        request: InstallSnapshotRequest,
    ) -> anyhow::Result<InstallSnapshotResponse> {
        let (sender, receiver) = oneshot::channel();
        let request = RaftRequest::InstallSnapshotRequest(request, sender);

        self.request(request).await?;

        let response = receiver.await?;
        Ok(response)
    }
}

pub struct ChannelServiceInterface {
    raft: MemRaft,
}

impl ChannelServiceInterface {
    pub fn new(raft: MemRaft) -> Self {
        ChannelServiceInterface { raft }
    }
}

#[async_trait]
impl ServiceInterface<ClientRequest> for ChannelServiceInterface {
    async fn on_vote_request(&self, request: VoteRequest) -> anyhow::Result<VoteResponse> {
        let response = self.raft.vote(request).await?;
        Ok(response)
    }

    async fn on_append_entries_request(
        &self,
        request: AppendEntriesRequest<ClientRequest>,
    ) -> anyhow::Result<AppendEntriesResponse> {
        let response = self.raft.append_entries(request).await?;
        Ok(response)
    }

    async fn on_install_snapshot_request(
        &self,
        request: InstallSnapshotRequest,
    ) -> anyhow::Result<InstallSnapshotResponse> {
        let response = self.raft.install_snapshot(request).await?;
        Ok(response)
    }
}
