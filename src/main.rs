// #![type_length_limit = "16511410"]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate async_trait;

use std::env::args;
use std::str::FromStr;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod client;
mod grpc;
mod raft;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();
    match args[1].as_str() {
        "raft" => as_raft(args.as_slice()).await?,
        "client" => as_client(args.as_slice()).await,
        _ => panic!("unknown args {}", args[0].as_str()),
    }

    Ok(())
}

/// script: ./raft_showcase raft 1
async fn as_raft(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let node_id = u64::from_str(args[2].as_str())?;

    let filter = tracing_subscriber::filter::Targets::new()
        .with_default(tracing_core::Level::DEBUG)
        .with_target("hyper", tracing_core::Level::INFO)
        .with_target("h2", tracing_core::Level::INFO)
        .with_target("tower", tracing_core::Level::INFO)
        .with_target("want", tracing_core::Level::INFO)
        .with_target("mio", tracing_core::Level::INFO)
        .with_target("tokio_util::codec::framed_impl", tracing_core::Level::INFO);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    crate::raft::bootstrap::raft_main(node_id).await;

    tokio::signal::ctrl_c().await?;
    Ok(())
}

/// script: ./raft_showcase client "http://127.0.0.1:35501" "my data"
async fn as_client(args: &[String]) {
    let leader_addr = args[2].as_str();
    let status = args[3].as_str();

    crate::grpc::client::client_write(leader_addr, status)
        .await
        .unwrap();
}
