pub mod pb {
    tonic::include_proto!("grpc.raft_showcase.raft");
    tonic::include_proto!("grpc.raft_showcase.showcase");
}
pub mod client;
pub mod server;
pub mod utils;
