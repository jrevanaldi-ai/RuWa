//! Ping Bot Example - Bot WhatsApp untuk test latency server
//!
//! Bot ini merespon command `!ping` dengan mengukur latency ke server WhatsApp.
//!
//! Usage:
//! ```bash
//! cargo run --example ping_bot
//! ```
//!
//! Commands:
//! - `!ping` - Test latency server
//! - `!pingstats` - Test dengan statistik (5 ping)
//! - `!latency` - Cek latency saat ini

use env_logger::Env;
use log::{error, info};
use ruwa::bot::Bot;
use ruwa::store::SqliteStore;
use ruwa_tokio_transport::TokioWebSocketTransportFactory;
use ruwa_ureq_http_client::UreqHttpClient;
use std::sync::Arc;
use std::time::Duration;
use wacore_ng::types::events::Event;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    println!("🏓 Ping Bot - WhatsApp Latency Tester");
    println!("====================================");
    println!();

    // Setup storage
    let store = SqliteStore::new("ping_bot.db").await?;
    info!("Database initialized");

    // Build bot
    info!("🤖 Building bot...");
    let bot = Bot::builder()
        .with_backend(Arc::new(store))
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .on_event(|event, _client| async move {
            if let Event::PairSuccess(_) = event {
                info!("✅ Pairing successful!");
            }
        })
        .build()
        .await?;

    info!("🚀 Bot started successfully!");
    info!("Send !ping to test latency");

    // Bot will run until Ctrl+C
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}
