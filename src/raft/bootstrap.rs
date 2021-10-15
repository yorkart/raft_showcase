use std::sync::Arc;

use async_raft::{Config, NodeId, Raft};

use crate::grpc::server::serve;
use crate::raft::member::Member;
use crate::raft::member::MemberGroup;
use crate::raft::network::RaftRouter;
use crate::raft::storage::MemStore;
use crate::raft::MemRaft;

pub async fn raft_main(node_id: NodeId) {
    let mut member_group = MemberGroup::new();
    member_group.add_member(Member::new(1u64, "http://127.0.0.1:35501".parse().unwrap()));
    member_group.add_member(Member::new(2u64, "http://127.0.0.1:35502".parse().unwrap()));
    member_group.add_member(Member::new(3u64, "http://127.0.0.1:35503".parse().unwrap()));

    raft_bootstrap(member_group, node_id).await;
}

async fn raft_bootstrap(member_group: MemberGroup, node_id: NodeId) {
    let members = member_group.all_member_node_ids();

    // Build our Raft runtime config, then instantiate our
    // RaftNetwork & RaftStorage impls.
    let config = Arc::new(
        Config::build("primary-raft-group".into())
            .election_timeout_max(6000)
            .election_timeout_min(5000)
            .heartbeat_interval(3000)
            .validate()
            .expect("failed to build Raft config"),
    );
    let network = Arc::new(RaftRouter::new(
        config.clone(),
        node_id,
        member_group.clone(),
    ));
    let storage = Arc::new(MemStore::new(node_id));

    // Create a new Raft node, which spawns an async task which
    // runs the Raft core logic. Keep this Raft instance around
    // for calling API methods based on events in your app.
    let raft = Raft::new(node_id, config, network, storage);

    raft.initialize(members).await.unwrap();

    run_service_interface(node_id, raft.clone(), member_group).await;
}

async fn run_service_interface(node_id: NodeId, raft: MemRaft, member_group: MemberGroup) {
    let bind_addr = member_group.member(node_id).unwrap().as_socket_addr();
    tokio::spawn(serve(raft, bind_addr));
}
