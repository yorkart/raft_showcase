#![allow(dead_code)]

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use anyhow::Result;
use async_raft::raft::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    VoteRequest, VoteResponse,
};
use async_raft::{Config, NodeId, Raft, RaftMetrics, RaftNetwork};
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::raft_v2::storage::MemStore;
use crate::raft_v2::{ClientRequest, MemRaft};

/// A type which emulates a network transport and implements the `RaftNetwork` trait.
pub struct RaftRouter {
    /// The Raft runtime config which all nodes are using.
    config: Arc<Config>,
    /// The table of all nodes currently known to this router instance.
    routing_table: RwLock<BTreeMap<NodeId, (MemRaft, Arc<MemStore>)>>,
    /// Nodes which are isolated can neither send nor receive frames.
    isolated_nodes: RwLock<HashSet<NodeId>>,
}

impl RaftRouter {
    pub fn new(config: Arc<Config>) -> Self {
        RaftRouter {
            config,
            routing_table: Default::default(),
            isolated_nodes: Default::default(),
        }
    }

    /// Create and register a new Raft node bearing the given ID.
    pub async fn new_raft_node(self: &Arc<Self>, id: NodeId) {
        let memstore = Arc::new(MemStore::new(id));
        let node = Raft::new(id, self.config.clone(), self.clone(), memstore.clone());
        let mut rt = self.routing_table.write().await;
        rt.insert(id, (node, memstore));
    }

    /// Remove the target node from the routing table & isolation.
    pub async fn remove_node(&self, id: NodeId) -> Option<(MemRaft, Arc<MemStore>)> {
        let mut rt = self.routing_table.write().await;
        let opt_handles = rt.remove(&id);
        let mut isolated = self.isolated_nodes.write().await;
        isolated.remove(&id);

        opt_handles
    }

    /// Initialize all nodes based on the config in the routing table.
    pub async fn initialize_from_single_node(&self, node: NodeId) -> Result<()> {
        tracing::info!({ node }, "initializing cluster from single node");
        let rt = self.routing_table.read().await;
        let members: HashSet<NodeId> = rt.keys().cloned().collect();
        rt.get(&node)
            .ok_or_else(|| anyhow!("node {} not found in routing table", node))?
            .0
            .initialize(members.clone())
            .await?;
        Ok(())
    }

    /// Isolate the network of the specified node.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn isolate_node(&self, id: NodeId) {
        self.isolated_nodes.write().await.insert(id);
    }

    /// Get a payload of the latest metrics from each node in the cluster.
    pub async fn latest_metrics(&self) -> Vec<RaftMetrics> {
        let rt = self.routing_table.read().await;
        let mut metrics = vec![];
        for node in rt.values() {
            metrics.push(node.0.metrics().borrow().clone());
        }
        metrics
    }

    /// Get the ID of the current leader.
    pub async fn leader(&self) -> Option<NodeId> {
        let isolated = self.isolated_nodes.read().await;
        self.latest_metrics().await.into_iter().find_map(|node| {
            if node.current_leader == Some(node.id) {
                if isolated.contains(&node.id) {
                    None
                } else {
                    Some(node.id)
                }
            } else {
                None
            }
        })
    }
}

#[async_trait]
impl RaftNetwork<ClientRequest> for RaftRouter {
    /// Send an AppendEntries RPC to the target Raft node (§5).
    async fn append_entries(
        &self,
        target: NodeId,
        rpc: AppendEntriesRequest<ClientRequest>,
    ) -> anyhow::Result<AppendEntriesResponse> {
        info!("append_entries target:{}, rpc: {:?}", target, rpc);
        let rt = self.routing_table.read().await;
        let isolated = self.isolated_nodes.read().await;
        let addr = rt
            .get(&target)
            .expect("target node not found in routing table");
        if isolated.contains(&target) || isolated.contains(&rpc.leader_id) {
            return Err(anyhow!("target node is isolated"));
        }
        Ok(addr.0.append_entries(rpc).await?)
    }

    /// Send an InstallSnapshot RPC to the target Raft node (§7).
    async fn install_snapshot(
        &self,
        target: NodeId,
        rpc: InstallSnapshotRequest,
    ) -> anyhow::Result<InstallSnapshotResponse> {
        info!("install_snapshot target:{}, rpc: {:?}", target, rpc);
        let rt = self.routing_table.read().await;
        let isolated = self.isolated_nodes.read().await;
        let addr = rt
            .get(&target)
            .expect("target node not found in routing table");
        if isolated.contains(&target) || isolated.contains(&rpc.leader_id) {
            return Err(anyhow!("target node is isolated"));
        }
        Ok(addr.0.install_snapshot(rpc).await?)
    }

    /// Send a RequestVote RPC to the target Raft node (§5).
    async fn vote(&self, target: NodeId, rpc: VoteRequest) -> anyhow::Result<VoteResponse> {
        info!("vote target:{}, rpc: {:?}", target, rpc);
        let rt = self.routing_table.read().await;
        let isolated = self.isolated_nodes.read().await;

        let addr = rt.get(&target).ok_or(anyhow!(
            "target node ({:?}) not found in routing table",
            target
        ))?;

        if isolated.contains(&target) || isolated.contains(&rpc.candidate_id) {
            return Err(anyhow!("target node is isolated"));
        }
        Ok(addr.0.vote(rpc).await?)
    }
}
