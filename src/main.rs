// #![type_length_limit = "16511410"]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
// #[macro_use]
// extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate async_trait;

use std::env::args;
use std::str::FromStr;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod grpc;
mod raft_v3;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();
    let node_id = u64::from_str(args[1].as_str())?;

    let filter = tracing_subscriber::filter::Targets::new()
        .with_default(tracing_core::Level::TRACE)
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

    crate::raft_v3::bootstrap::raft_main(node_id).await;

    tokio::signal::ctrl_c().await?;
    Ok(())
}
