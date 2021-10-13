// #![type_length_limit = "16511410"]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate async_trait;

// mod raft;
mod raft_v2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    info!("a");

    crate::raft_v2::bootstrap::raft_main().await;

    tokio::signal::ctrl_c().await?;
    Ok(())
}
