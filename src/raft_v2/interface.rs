use std::collections::{HashMap, HashSet};

use async_raft::raft::{AppendEntriesRequest, AppendEntriesResponse, VoteRequest, VoteResponse};
use async_raft::{AppData, NodeId};
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};

use crate::raft_v2::network::RaftRouter;
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
}

#[async_trait]
pub trait ServiceInterface<D: AppData> {
    async fn on_vote_request(&self, request: VoteRequest) -> anyhow::Result<VoteResponse>;
    async fn on_append_entries_request(
        &self,
        request: AppendEntriesRequest<D>,
    ) -> anyhow::Result<AppendEntriesResponse>;
}

pub async fn build_interface(
    member: HashSet<NodeId>,
    node_id: NodeId,
    raft: MemRaft,
) -> ChannelRemoteInterface {
    let (sender, mut receiver) = mpsc::channel::<RaftRequest>(1);

    {
        let mut lock = SENDERS.lock().await;
        if lock.contains_key(&node_id) {
            panic!("node {} has publish", node_id);
        }

        lock.insert(node_id, sender);
    }

    tokio::spawn(interface_service(raft, receiver));

    ChannelRemoteInterface::new(member, node_id)
}

async fn interface_service(raft: MemRaft, mut receiver: mpsc::Receiver<RaftRequest>) {
    let interface_event = ChannelServiceInterface::new(raft);

    while let Some(t) = receiver.recv().await {
        match interface_service_event(&interface_event, t).await {
            Ok(()) => info!("success"),
            Err(e) => error!("failure {}", e),
        }
    }
}

async fn interface_service_event(
    interface_event: &ChannelServiceInterface,
    mut request: RaftRequest,
) -> anyhow::Result<()> {
    match request {
        RaftRequest::VoteRequest(request, tx) => {
            let response = interface_event.on_vote_request(request).await?;
            tx.send(response)
                .map_err(|_e| anyhow!("channel disconnection"));
            Ok(())
        }
        RaftRequest::AppendEntriesRequest(request, tx) => {
            let response = interface_event.on_append_entries_request(request).await?;
            tx.send(response)
                .map_err(|_e| anyhow!("channel disconnection"));
            Ok(())
        }
    }
}

pub struct ChannelRemoteInterface {
    member: HashSet<NodeId>,
    node_id: NodeId,
}

impl ChannelRemoteInterface {
    pub fn new(member: HashSet<NodeId>, node_id: NodeId) -> Self {
        ChannelRemoteInterface { member, node_id }
    }

    pub async fn request(&self, request: RaftRequest) -> anyhow::Result<()> {
        let mut lock = SENDERS.lock().await;
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
}
