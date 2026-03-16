// Advanced RuWa Bot Example
// This example shows more complex bot features

use ruwa::bot::Bot;
use ruwa::store::SqliteStore;
use ruwa_tokio_transport::TokioWebSocketTransportFactory;
use ruwa_ureq_http_client::UreqHttpClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use wacore::types::events::Event;
use waproto::whatsapp as wa;

/// Advanced bot with state management
struct AdvancedBot {
    // Store user sessions
    user_sessions: Arc<RwLock<HashMap<String, UserSession>>>,
}

#[derive(Debug, Clone)]
struct UserSession {
    state: String,
    data: HashMap<String, String>,
}

impl AdvancedBot {
    fn new() -> Self {
        Self {
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let store = SqliteStore::new("advanced-bot.db").await?;

        let bot = Bot::builder()
            .with_backend(store)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new())
            .on_event(|event, client| async move {
                match event {
                    Event::PairingQrCode { code, timeout } => {
                        println!("QR Code ({}s):\n{}", timeout.as_secs(), code);
                    }
                    Event::Message(msg, info) => {
                        // Handle message with state
                        self.handle_message(msg, info, client).await;
                    }
                    Event::Connected(_) => {
                        println!("✅ Advanced bot connected!");
                    }
                    _ => {}
                }
            })
            .build()
            .await?;

        bot.run().await?.await?;
        Ok(())
    }

    async fn handle_message(
        &self,
        msg: wa::Message,
        info: wacore::types::message::MessageInfo,
        client: Arc<ruwa::client::Client>,
    ) {
        let Some(text) = msg.text_content() else {
            return;
        };

        let user_id = info.source.sender.to_string();

        // Get or create user session
        let mut sessions = self.user_sessions.write().await;
        let session = sessions
            .entry(user_id.clone())
            .or_insert_with(|| UserSession {
                state: "idle".to_string(),
                data: HashMap::new(),
            });

        // State machine handling
        match session.state.as_str() {
            "idle" => {
                // Handle commands in idle state
                match text {
                    "!start" => {
                        session.state = "started".to_string();
                        let _ = client
                            .send_message(
                                info.source.chat,
                                wa::Message {
                                    conversation: Some(
                                        "🎮 Game started! Choose: !rock, !paper, or !scissors"
                                            .to_string(),
                                    ),
                                    ..Default::default()
                                },
                            )
                            .await;
                    }
                    "!status" => {
                        let _ = client
                            .send_message(
                                info.source.chat,
                                wa::Message {
                                    conversation: Some(format!(
                                        "📊 Your status:\nState: {}\nData: {:?}",
                                        session.state, session.data
                                    )),
                                    ..Default::default()
                                },
                            )
                            .await;
                    }
                    _ => {}
                }
            }
            "started" => {
                // Handle game commands
                match text {
                    "!rock" | "!paper" | "!scissors" => {
                        use rand::Rng;
                        let choices = ["!rock", "!paper", "!scissors"];
                        let bot_choice = choices[rand::rng().random_range(0..3)];

                        let result = match (text, bot_choice) {
                            (a, b) if a == b => "🤝 Draw!",
                            ("!rock", "!scissors")
                            | ("!paper", "!rock")
                            | ("!scissors", "!paper") => "🎉 You win!",
                            _ => "😔 You lose!",
                        };

                        let _ = client
                            .send_message(
                                info.source.chat,
                                wa::Message {
                                    conversation: Some(format!(
                                        "You: {}\nBot: {}\n{}",
                                        text, bot_choice, result
                                    )),
                                    ..Default::default()
                                },
                            )
                            .await;

                        session.state = "idle".to_string();
                    }
                    "!quit" => {
                        session.state = "idle".to_string();
                        let _ = client
                            .send_message(
                                info.source.chat,
                                wa::Message {
                                    conversation: Some("👋 Game ended.".to_string()),
                                    ..Default::default()
                                },
                            )
                            .await;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced RuWa Bot");
    println!("This example shows state management");

    let bot = AdvancedBot::new();
    bot.run().await?;

    Ok(())
}
