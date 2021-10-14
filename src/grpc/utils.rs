use async_raft::raft::{
    AppendEntriesRequest, AppendEntriesResponse, ConflictOpt, Entry, EntryConfigChange,
    EntryNormal, EntryPayload, EntrySnapshotPointer, InstallSnapshotRequest,
    InstallSnapshotResponse, MembershipConfig, VoteRequest, VoteResponse,
};
use std::collections::HashSet;

use crate::grpc::pb;
use crate::raft::ClientRequest;

pub fn vote_request_to_pb(request: VoteRequest) -> pb::VoteRequest {
    pb::VoteRequest {
        term: request.term,
        candidate_id: request.candidate_id,
        last_log_index: request.last_log_index,
        last_log_term: request.last_log_term,
    }
}
pub fn vote_request_to_raft(request: pb::VoteRequest) -> VoteRequest {
    VoteRequest {
        term: request.term,
        candidate_id: request.candidate_id,
        last_log_index: request.last_log_index,
        last_log_term: request.last_log_term,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn vote_response_to_pb(response: VoteResponse) -> pb::VoteResponse {
    pb::VoteResponse {
        term: response.term,
        vote_granted: response.vote_granted,
    }
}

pub fn vote_response_to_raft(response: pb::VoteResponse) -> VoteResponse {
    VoteResponse {
        term: response.term,
        vote_granted: response.vote_granted,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn install_snapshot_request_to_pb(
    request: InstallSnapshotRequest,
) -> pb::InstallSnapshotRequest {
    pb::InstallSnapshotRequest {
        term: request.term,
        leader_id: request.leader_id,
        last_included_index: request.last_included_index,
        last_included_term: request.last_included_term,
        offset: request.offset,
        data: request.data.clone(),
        done: request.done,
    }
}

pub fn install_snapshot_request_to_raft(
    request: pb::InstallSnapshotRequest,
) -> InstallSnapshotRequest {
    InstallSnapshotRequest {
        term: request.term,
        leader_id: request.leader_id,
        last_included_index: request.last_included_index,
        last_included_term: request.last_included_term,
        offset: request.offset,
        data: request.data.clone(),
        done: request.done,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn install_snapshot_response_to_pb(
    response: InstallSnapshotResponse,
) -> pb::InstallSnapshotResponse {
    pb::InstallSnapshotResponse {
        term: response.term,
    }
}

pub fn install_snapshot_response_to_raft(
    response: pb::InstallSnapshotResponse,
) -> InstallSnapshotResponse {
    InstallSnapshotResponse {
        term: response.term,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn membership_config_to_pb(mc: MembershipConfig) -> pb::MembershipConfig {
    let members = mc.members.iter().map(|m| *m).collect();
    let members_after_consensus = mc
        .members_after_consensus
        .map(|mac| mac.iter().map(|m| *m).collect())
        .unwrap_or_default();

    pb::MembershipConfig {
        members,
        members_after_consensus,
    }
}

pub fn membership_config_to_raft(mc: pb::MembershipConfig) -> MembershipConfig {
    let mut members = HashSet::new();
    mc.members.iter().for_each(|m| {
        members.insert(*m);
    });

    let members_after_consensus = if mc.members_after_consensus.len() > 0 {
        let mut members_after_consensus = HashSet::new();
        mc.members_after_consensus.iter().for_each(|m| {
            members_after_consensus.insert(*m);
        });
        Some(members_after_consensus)
    } else {
        None
    };

    MembershipConfig {
        members,
        members_after_consensus,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn entry_to_pb(entry: Entry<ClientRequest>) -> pb::Entry {
    let Entry {
        term,
        index,
        payload,
    } = entry;

    let payload = match payload {
        EntryPayload::Blank => None,
        EntryPayload::Normal(en) => {
            let en = pb::EntryNormal {
                data: serde_json::to_string(&en.data).unwrap().into_bytes(),
            };
            Some(pb::entry::Payload::EntryNormal(en))
        }
        EntryPayload::ConfigChange(cc) => {
            let membership = Some(membership_config_to_pb(cc.membership));
            let cc = pb::EntryConfigChange { membership };
            Some(pb::entry::Payload::EntryConfigChange(cc))
        }
        EntryPayload::SnapshotPointer(sp) => {
            let EntrySnapshotPointer { id, membership } = sp;
            let membership = Some(membership_config_to_pb(membership));
            let esp = pb::EntrySnapshotPointer { id, membership };
            Some(pb::entry::Payload::EntrySnapshotPointer(esp))
        }
    };

    pb::Entry {
        term,
        index,
        payload,
    }
}

pub fn entry_to_raft(entry: pb::Entry) -> Entry<ClientRequest> {
    let pb::Entry {
        term,
        index,
        payload,
    } = entry;
    let payload = match payload {
        Some(pb::entry::Payload::EntryNormal(en)) => EntryPayload::Normal(EntryNormal {
            data: serde_json::from_slice::<ClientRequest>(en.data.as_slice()).unwrap(),
        }),
        Some(pb::entry::Payload::EntryConfigChange(cc)) => {
            let membership = membership_config_to_raft(cc.membership.unwrap());
            EntryPayload::ConfigChange(EntryConfigChange { membership })
        }
        Some(pb::entry::Payload::EntrySnapshotPointer(esp)) => {
            let pb::EntrySnapshotPointer { id, membership } = esp;
            let membership = membership_config_to_raft(membership.unwrap());
            EntryPayload::SnapshotPointer(EntrySnapshotPointer { id, membership })
        }
        None => EntryPayload::Blank,
    };
    Entry {
        term,
        index,
        payload,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn append_entries_request_to_pb(
    request: AppendEntriesRequest<ClientRequest>,
) -> pb::AppendEntriesRequest {
    let AppendEntriesRequest {
        term,
        leader_id,
        prev_log_index,
        prev_log_term,
        entries,
        leader_commit,
    } = request;

    let entries = entries
        .into_iter()
        .map(|entry| entry_to_pb(entry))
        .collect();

    pb::AppendEntriesRequest {
        term,
        leader_id,
        prev_log_index,
        prev_log_term,
        entries,
        leader_commit,
    }
}

pub fn append_entries_request_to_raft(
    request: pb::AppendEntriesRequest,
) -> AppendEntriesRequest<ClientRequest> {
    let pb::AppendEntriesRequest {
        term,
        leader_id,
        prev_log_index,
        prev_log_term,
        entries,
        leader_commit,
    } = request;

    let entries = entries
        .into_iter()
        .map(|entry| entry_to_raft(entry))
        .collect();

    AppendEntriesRequest {
        term,
        leader_id,
        prev_log_index,
        prev_log_term,
        entries,
        leader_commit,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn append_entries_response_to_pb(response: AppendEntriesResponse) -> pb::AppendEntriesResponse {
    let AppendEntriesResponse {
        term,
        success,
        conflict_opt,
    } = response;

    pb::AppendEntriesResponse {
        term,
        success,
        conflict_opt: conflict_opt.map(|co| pb::ConflictOpt {
            term: co.term,
            index: co.index,
        }),
    }
}

pub fn append_entries_response_to_raft(
    response: pb::AppendEntriesResponse,
) -> AppendEntriesResponse {
    let pb::AppendEntriesResponse {
        term,
        success,
        conflict_opt,
    } = response;

    AppendEntriesResponse {
        term,
        success,
        conflict_opt: conflict_opt.map(|co| ConflictOpt {
            term: co.term,
            index: co.index,
        }),
    }
}
