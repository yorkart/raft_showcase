fn main() {
    tonic_build::configure()
        .type_attribute(
            "grpc.raft_showcase.client.ClientRequest",
            "#[derive(Serialize, Deserialize)]",
        )
        .type_attribute(
            "grpc.raft_showcase.client.ClientResponse",
            "#[derive(Serialize, Deserialize)]",
        )
        .compile(&["proto/client.proto"], &["proto"])
        .unwrap();
    tonic_build::compile_protos("proto/echo.proto").unwrap();
    tonic_build::compile_protos("proto/raft.proto").unwrap()
}
