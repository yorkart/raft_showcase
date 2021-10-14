#![allow(dead_code)]

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use crate::grpc::client::GRpcClient;
use async_raft::raft::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    VoteRequest, VoteResponse,
};
use async_raft::{Config, NodeId, RaftNetwork};
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::raft_v3::member::MemberGroup;
use crate::raft_v3::ClientRequest;

/// A type which emulates a network transport and implements the `RaftNetwork` trait.
pub struct RaftRouter {
    /// The Raft runtime config which all nodes are using.
    config: Arc<Config>,
    /// The table of all nodes currently known to this router instance.
    routing_table: RwLock<BTreeMap<NodeId, GRpcClient>>,
    /// Nodes which are isolated can neither send nor receive frames.
    isolated_nodes: RwLock<HashSet<NodeId>>,
}

impl RaftRouter {
    pub fn new(config: Arc<Config>, node_id: NodeId, member_group: MemberGroup) -> Self {
        let members = member_group.all_member_node_ids();

        let mut routing_table = BTreeMap::new();
        members.iter().filter(|m| **m != node_id).for_each(|m| {
            let uri = member_group.member(*m).unwrap().uri().clone();
            routing_table.insert(*m, GRpcClient::new(uri));
        });

        RaftRouter {
            config,
            routing_table: RwLock::new(routing_table),
            isolated_nodes: Default::default(),
        }
    }

    // /// Isolate the network of the specified node.
    // #[tracing::instrument(level = "debug", skip(self))]
    // pub async fn isolate_node(&self, id: NodeId) {
    //     self.isolated_nodes.write().await.insert(id);
    // }
    //
    // /// Get a payload of the latest metrics from each node in the cluster.
    // pub async fn latest_metrics(&self) -> Vec<RaftMetrics> {
    //     let rt = self.routing_table.read().await;
    //     let mut metrics = vec![];
    //     for node in rt.values() {
    //         metrics.push(node.0.metrics().borrow().clone());
    //     }
    //     metrics
    // }
    //
    // /// Get the ID of the current leader.
    // pub async fn leader(&self) -> Option<NodeId> {
    //     let isolated = self.isolated_nodes.read().await;
    //     self.latest_metrics().await.into_iter().find_map(|node| {
    //         if node.current_leader == Some(node.id) {
    //             if isolated.contains(&node.id) {
    //                 None
    //             } else {
    //                 Some(node.id)
    //             }
    //         } else {
    //             None
    //         }
    //     })
    // }
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
        let mut rt = self.routing_table.write().await;
        let isolated = self.isolated_nodes.read().await;
        let addr = rt
            .get_mut(&target)
            .expect("target node not found in routing table");
        if isolated.contains(&target) || isolated.contains(&rpc.leader_id) {
            return Err(anyhow!("target node is isolated"));
        }
        Ok(addr.append_entries(rpc).await?)
    }

    /// Send an InstallSnapshot RPC to the target Raft node (§7).
    async fn install_snapshot(
        &self,
        target: NodeId,
        rpc: InstallSnapshotRequest,
    ) -> anyhow::Result<InstallSnapshotResponse> {
        info!("install_snapshot target:{}, rpc: {:?}", target, rpc);
        let mut rt = self.routing_table.write().await;
        let isolated = self.isolated_nodes.read().await;
        let addr = rt
            .get_mut(&target)
            .expect("target node not found in routing table");
        if isolated.contains(&target) || isolated.contains(&rpc.leader_id) {
            return Err(anyhow!("target node is isolated"));
        }
        Ok(addr.install_snapshot(rpc).await?)
    }

    /// Send a RequestVote RPC to the target Raft node (§5).
    async fn vote(&self, target: NodeId, rpc: VoteRequest) -> anyhow::Result<VoteResponse> {
        info!("vote target:{}, rpc: {:?}", target, rpc);
        let mut rt = self.routing_table.write().await;
        let isolated = self.isolated_nodes.read().await;

        let addr = rt.get_mut(&target).ok_or(anyhow!(
            "target node ({:?}) not found in routing table",
            target
        ))?;

        if isolated.contains(&target) || isolated.contains(&rpc.candidate_id) {
            return Err(anyhow!("target node is isolated"));
        }
        Ok(addr.vote(rpc).await?)
    }
}
