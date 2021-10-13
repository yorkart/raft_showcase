use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;

use async_raft::NodeId;

#[derive(Clone)]
pub struct Member {
    id: NodeId,
    ip: SocketAddr,
}

impl Member {
    pub fn new(id: u64, ip: SocketAddr) -> Self {
        Member { id, ip }
    }
}

#[derive(Clone)]
pub struct MemberGroup {
    members: HashMap<NodeId, Member>,
}

impl MemberGroup {
    pub fn new() -> Self {
        MemberGroup {
            members: HashMap::new(),
        }
    }

    pub fn add_member(&mut self, member: Member) {
        self.members.insert(member.id, member);
    }

    pub fn get_member_node_ids(&self) -> HashSet<NodeId> {
        let mut node_ids = HashSet::new();
        for n in self.members.keys() {
            node_ids.insert(*n);
        }

        node_ids
    }

    // pub fn get_member(&self, node_id: NodeId) -> Option<&Member> {
    //     self.members.get(&node_id)
    // }
}
