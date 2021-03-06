use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;

use async_raft::NodeId;

use http::Uri;

#[derive(Clone)]
pub struct Member {
    id: NodeId,
    uri: Uri,
}

impl Member {
    pub fn new(id: u64, uri: Uri) -> Self {
        Member { id, uri }
    }

    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn as_socket_addr(&self) -> SocketAddr {
        let host = self.uri.host().unwrap();
        let port = self.uri.port_u16().unwrap();
        SocketAddr::new(host.parse().unwrap(), port)
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

    pub fn all_member_node_ids(&self) -> HashSet<NodeId> {
        let mut node_ids = HashSet::new();
        for n in self.members.keys() {
            node_ids.insert(*n);
        }

        node_ids
    }

    pub fn member(&self, node_id: NodeId) -> Option<&Member> {
        self.members.get(&node_id)
    }
}
