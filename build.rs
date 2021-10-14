fn main() {
    tonic_build::compile_protos("proto/echo.proto").unwrap();
    tonic_build::compile_protos("proto/raft.proto").unwrap();
}
