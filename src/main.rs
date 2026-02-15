//! SSE (Server-Sent Events) capability provider for wasmCloud
//!
//! This provider connects to remote SSE servers and forwards received events
//! to wasmCloud components via wRPC. It implements unidirectional communication
//! (receiving only) with automatic reconnection and message size limits.

mod config;
mod provider;
mod sse;

use provider::SseProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    SseProvider::run().await?;
    eprintln!("SSE provider exiting");
    Ok(())
}
