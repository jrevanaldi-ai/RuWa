mod commands;
mod config;

use anyhow::Result;
use chrono::Local;
use commands::CommandHandler;
use config::Config;
use env_logger::Env;
use log::{error, info, warn};
use ruwa::bot::{Bot, MessageContext};
use ruwa::store::SqliteStore;
use ruwa_tokio_transport::TokioWebSocketTransportFactory;
use ruwa_ureq_http_client::UreqHttpClient;
use std::sync::Arc;
use wacore::types::events::Event;
use waproto::whatsapp as wa;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{} [{:<5}] {} - {}",
                Local::now().format("%H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();

    info!("🚀 Starting RuWa Bot v1.0.0");
    info!("👤 Developer: Nathan");
    info!("📱 WhatsApp Bot using RuWa library");

    // Load configuration
    let config = Config::load();

    // Initialize storage
    info!("📦 Initializing database...");
    let store = SqliteStore::new(&config.database_path).await?;
    info!("✅ Database initialized: {}", config.database_path);

    // Build bot
    info!("🤖 Building bot...");
    let mut bot = Bot::builder()
        .with_backend(Arc::new(store))
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .on_event(move |event, client| {
            let config = config.clone();
            async move {
                match event {
                    // QR Code event
                    Event::PairingQrCode { code, timeout } => {
                        info!("📱 QR Code received (valid for {}s)", timeout.as_secs());
                        info!("\n{}", "=".repeat(50));
                        info!("{}", code);
                        info!("{}\n", "=".repeat(50));
                        info!("Scan this QR code with your WhatsApp app");
                        info!("WhatsApp > Linked Devices > Link a Device");
                    }

                    // Pair Code event
                    Event::PairingCode { code, timeout } => {
                        info!("🔢 Pair Code received (valid for {}s)", timeout.as_secs());
                        info!("\n{}", "=".repeat(50));
                        info!("  CODE: {}", code);
                        info!("{}\n", "=".repeat(50));
                        info!("Enter this code in WhatsApp:");
                        info!("WhatsApp > Linked Devices > Link a Device");
                        info!("> Link with phone number instead");
                    }

                    // Pairing success
                    Event::PairSuccess(info) => {
                        info!("✅ Pairing successful!");
                        info!("   JID: {}", info.jid);
                        info!("   Push Name: {}", info.push_name);
                    }

                    // Connected event
                    Event::Connected(_) => {
                        info!("✅ Bot connected and ready!");
                        info!("   Commands available:");
                        info!("   - !menu - Show all commands");
                        info!("   - !ping - Test bot response");
                        info!("   - !help - Get help");
                        info!("   - !group - Group management");
                    }

                    // Message received
                    Event::Message(msg, info) => {
                        let ctx = MessageContext {
                            message: msg,
                            info,
                            client,
                        };

                        // Handle commands
                        if let Err(e) = CommandHandler::handle(ctx).await {
                            error!("❌ Error handling command: {}", e);
                        }
                    }

                    // Receipt event
                    Event::Receipt(receipt) => {
                        info!(
                            "📬 Receipt for {:?} - Type: {:?}",
                            receipt.message_ids, receipt.r#type
                        );
                    }

                    // Disconnected
                    Event::Disconnected(reason) => {
                        warn!("⚠️ Bot disconnected: {:?}", reason);
                    }

                    // Logged out
                    Event::LoggedOut(_) => {
                        warn!("⚠️ Bot logged out!");
                        error!("Please restart the bot to re-pair");
                    }

                    // Other events
                    _ => {
                        // Debug: Uncomment to see all events
                        // debug!("Event: {:?}", event);
                    }
                }
            }
        })
        .build()
        .await?;

    info!("🤖 Bot built successfully!");

    // Run bot
    info!("🏃 Starting bot...");
    match bot.run().await {
        Ok(handle) => {
            info!("✅ Bot is running!");
            
            // Wait for bot to finish
            handle.await?;
        }
        Err(e) => {
            error!("❌ Failed to run bot: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
