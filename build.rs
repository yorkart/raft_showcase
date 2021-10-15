fn main() {
    tonic_build::configure()
        .type_attribute(
            "grpc.raft_showcase.showcase.ClientRequest",
            "#[derive(Serialize, Deserialize)]",
        )
        .type_attribute(
            "grpc.raft_showcase.showcase.ClientResponse",
            "#[derive(Serialize, Deserialize)]",
        )
        .compile(&["proto/showcase.proto"], &["proto"])
        .unwrap();
    tonic_build::compile_protos("proto/raft.proto").unwrap()
}
