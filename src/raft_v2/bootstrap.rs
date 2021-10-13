use crate::raft_v2::member::Member;
use crate::raft_v2::member::MemberGroup;
use crate::raft_v2::network::RaftRouter;
use crate::raft_v2::storage::MemStore;
use crate::raft_v2::RaftRequest;
use async_raft::{NodeId, Raft};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn raft_main() {
    let (sender1, mut receiver1) = mpsc::channel::<RaftRequest>(1);
    let (sender2, mut receiver2) = mpsc::channel::<RaftRequest>(1);
    let (sender3, mut receiver3) = mpsc::channel::<RaftRequest>(1);

    let sender_group = vec![sender1, sender2, sender3];

    let mut member_group = MemberGroup::new();
    member_group.add_member(Member::new(1u64, "0.0.0.0:35501".parse().unwrap()));
    member_group.add_member(Member::new(2u64, "0.0.0.0:35502".parse().unwrap()));
    member_group.add_member(Member::new(3u64, "0.0.0.0:35503".parse().unwrap()));

    let members = member_group.get_member_node_ids();

    raft_bootstrap(members.clone(), 1u64).await;
    raft_bootstrap(members.clone(), 2u64).await;
    raft_bootstrap(members.clone(), 3u64).await;
}

async fn raft_bootstrap(members: HashSet<NodeId>, node_id: NodeId) {
    // Build our Raft runtime config, then instantiate our
    // RaftNetwork & RaftStorage impls.
    let config = Arc::new(
        Config::build("primary-raft-group".into())
            .election_timeout_max(3000)
            .heartbeat_interval(1500)
            .validate()
            .expect("failed to build Raft config"),
    );
    let network = Arc::new(RaftRouter::new(config.clone()));
    let storage = Arc::new(MemStore::new(node_id));

    // Create a new Raft node, which spawns an async task which
    // runs the Raft core logic. Keep this Raft instance around
    // for calling API methods based on events in your app.
    let raft = Raft::new(node_id, config, network, storage);

    raft.initialize(members).await.unwrap();

    let (sender, mut receiver) = mpsc::channel::<RaftRequest>(1);
    tokio::spawn(async move {
        let raft = raft;
        while let Some(request) = receiver.recv().await {
            match request {
                RaftRequest::VoteRequest(v, tx) => {
                    let response = raft.vote(v).await.unwrap();
                    tx.send(response);
                }
                RaftRequest::AppendEntriesRequest(a, tx) => {
                    let response = raft.append_entries(a).await.unwrap();
                    tx.send(response);
                }
            }
        }
    });
}
