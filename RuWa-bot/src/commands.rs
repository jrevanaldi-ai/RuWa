use anyhow::Result;
use ruwa::bot::MessageContext;
use ruwa::features::{Chatstate, ChatStateType};
use wacore_ng::proto_helpers::MessageExt;
use waproto_ng::whatsapp as wa;

/// Command handler for WhatsApp bot
pub struct CommandHandler;

impl CommandHandler {
    /// Handle incoming message and process commands
    pub async fn handle(ctx: MessageContext<'_>) -> Result<()> {
        // Get message text
        let Some(text) = ctx.message.text_content() else {
            return Ok(()); // Ignore non-text messages
        };

        let text = text.trim();
        
        // Ignore if not a command
        if !text.starts_with('!') {
            return Ok(());
        }

        // Parse command
        let parts: Vec<&str> = text.split_whitespace().collect();
        let command = parts[0].to_lowercase();
        let args = &parts[1..];

        info!("📥 Command received: {} from {}", command, ctx.info.source.sender);

        // Send typing indicator
        let _ = ctx
            .client
            .chatstate()
            .send_chatstate(ctx.info.source.chat.clone(), ChatStateType::Composing)
            .await;

        // Process command
        let response = match command.as_str() {
            "!ping" => Self::cmd_ping().await,
            "!menu" | "!commands" => Self::cmd_menu().await,
            "!help" => Self::cmd_help().await,
            "!info" => Self::cmd_info(&ctx).await,
            "!group" => Self::cmd_group(&ctx, args).await,
            "!me" => Self::cmd_me(&ctx).await,
            "!quote" | "!q" => Self::cmd_quote(&ctx, args).await,
            "!react" => Self::cmd_react(&ctx, args).await,
            "!delete" => Self::cmd_delete(&ctx, args).await,
            _ => Self::cmd_unknown().await,
        };

        // Send response
        if let Ok(msg) = response {
            let _ = ctx
                .client
                .send_message(ctx.info.source.chat.clone(), msg)
                .await;
        }

        Ok(())
    }

    /// !ping - Test bot response
    async fn cmd_ping() -> Result<wa::Message> {
        let start = std::time::Instant::now();
        
        Ok(wa::Message {
            conversation: Some(format!(
                "🏓 Pong!\n⏱️  Response time: {:?}",
                start.elapsed()
            )),
            ..Default::default()
        })
    }

    /// !menu - Show all commands
    async fn cmd_menu() -> Result<wa::Message> {
        let menu = r#"
╔═══════════════════════════╗
   🤖 *RuWa Bot Menu*
╚═══════════════════════════╝

*📌 General Commands:*
├ !menu - Show this menu
├ !help - Get help
├ !ping - Test bot response
├ !info - Bot information
└ !me - Your info

*👥 Group Commands:*
├ !group info - Group information
├ !group list - List participants
└ !group admin - List admins

*✨ Fun Commands:*
├ !quote <msg> - Quote message
└ !react <emoji> - React to message

*⚙️ Admin Commands:*
└ !delete <msg_id> - Delete message

─────────────────────────────
*RuWa Bot v1.0.0*
Made with ❤️ using Rust
"#;

        Ok(wa::Message {
            conversation: Some(menu.to_string()),
            ..Default::default()
        })
    }

    /// !help - Get help
    async fn cmd_help() -> Result<wa::Message> {
        let help = r#"
*🤖 RuWa Bot Help*

*How to use:*
1. Type ! followed by command
2. Example: !ping
3. Bot will respond automatically

*Available commands:*
• !menu - Show all commands
• !ping - Test response time
• !info - Bot information
• !group - Group management

*Need more help?*
Contact: @AstraluneSuport (Telegram)

*Version:* 1.0.0
*Powered by:* RuWa Library
"#;

        Ok(wa::Message {
            conversation: Some(help.to_string()),
            ..Default::default()
        })
    }

    /// !info - Bot information
    async fn cmd_info(_ctx: &MessageContext<'_>) -> Result<wa::Message> {
        let info = r#"
*🤖 RuWa Bot Information*

*Name:* RuWa Bot
*Version:* 1.0.0
*Library:* RuWa (Rust WhatsApp API)
*Developer:* Nathan

*Features:*
✅ Fast response (Rust)
✅ Low memory usage
✅ Multi-device support
✅ E2E encryption

*Links:*
• GitHub: github.com/jrevanaldi-ai/ruwa
• Crates: crates.io/crates/ruwa

*Contact:*
• WhatsApp: +62 895-4166-02000
• Telegram: @AstraluneSuport
• Email: support@nathan.christmas
"#;

        Ok(wa::Message {
            conversation: Some(info.to_string()),
            ..Default::default()
        })
    }

    /// !me - Show sender info
    async fn cmd_me(ctx: &MessageContext<'_>) -> Result<wa::Message> {
        let info = ctx.info.clone();
        
        let me = format!(
            r#"
*👤 Your Information*

*Sender:* {}
*Chat:* {}
*Message ID:* {}
*Timestamp:* {}
*Is Group:* {}
*Is From Me:* {}
"#,
            info.source.sender,
            info.source.chat,
            info.id,
            info.timestamp,
            if info.source.is_group { "Yes" } else { "No" },
            if info.source.is_from_me { "Yes" } else { "No" },
        );

        Ok(wa::Message {
            conversation: Some(me),
            ..Default::default()
        })
    }

    /// !group - Group management
    async fn cmd_group(ctx: &MessageContext<'_>, args: &[&str]) -> Result<wa::Message> {
        // Check if in group
        if !ctx.info.source.is_group {
            return Ok(wa::Message {
                conversation: Some("❌ This command can only be used in groups".to_string()),
                ..Default::default()
            });
        }

        let subcommand = args.first().unwrap_or(&"info");

        match *subcommand {
            "info" => {
                // Get group metadata
                match ctx.client.groups().get_metadata(ctx.info.source.chat.clone()).await {
                    Ok(metadata) => {
                        let info = format!(
                            r#"
*👥 Group Information*

*Name:* {}
*ID:* {}
*Owner:* {}
*Created:* {}
*Participants:* {}
*Admins:* {}
*Description:* {}
"#,
                            metadata.subject,
                            ctx.info.source.chat,
                            metadata.owner.unwrap_or_default(),
                            metadata.creation,
                            metadata.participants.len(),
                            metadata.participants.iter().filter(|p| p.is_admin).count(),
                            metadata.description.as_deref().unwrap_or("No description"),
                        );

                        Ok(wa::Message {
                            conversation: Some(info),
                            ..Default::default()
                        })
                    }
                    Err(e) => Ok(wa::Message {
                        conversation: Some(format!("❌ Error: {}", e)),
                        ..Default::default()
                    }),
                }
            }
            "list" => {
                // List participants
                match ctx.client.groups().get_metadata(ctx.info.source.chat.clone()).await {
                    Ok(metadata) => {
                        let mut list = String::from("*👥 Participants:*\n\n");
                        for (i, p) in metadata.participants.iter().enumerate() {
                            list.push_str(&format!("{}. {} {}\n", i + 1, p.jid, if p.is_admin { "👑" } else { "" }));
                        }
                        Ok(wa::Message {
                            conversation: Some(list),
                            ..Default::default()
                        })
                    }
                    Err(e) => Ok(wa::Message {
                        conversation: Some(format!("❌ Error: {}", e)),
                        ..Default::default()
                    }),
                }
            }
            "admin" => {
                // List admins only
                match ctx.client.groups().get_metadata(ctx.info.source.chat.clone()).await {
                    Ok(metadata) => {
                        let admins: Vec<_> = metadata.participants.iter().filter(|p| p.is_admin).collect();
                        let mut list = String::from("*👑 Admins:*\n\n");
                        for (i, admin) in admins.iter().enumerate() {
                            list.push_str(&format!("{}. {}\n", i + 1, admin.jid));
                        }
                        Ok(wa::Message {
                            conversation: Some(list),
                            ..Default::default()
                        })
                    }
                    Err(e) => Ok(wa::Message {
                        conversation: Some(format!("❌ Error: {}", e)),
                        ..Default::default()
                    }),
                }
            }
            _ => Ok(wa::Message {
                conversation: Some("❌ Unknown subcommand. Use: !group info | list | admin".to_string()),
                ..Default::default()
            }),
        }
    }

    /// !quote - Quote a message
    async fn cmd_quote(ctx: &MessageContext<'_>, args: &[&str]) -> Result<wa::Message> {
        if args.is_empty() {
            return Ok(wa::Message {
                conversation: Some("❌ Usage: !quote <message>".to_string()),
                ..Default::default()
            });
        }

        let quote_text = args.join(" ");
        
        // Build quote context from original message
        let context_info = ctx.build_quote_context();

        Ok(wa::Message {
            extended_text_message: Some(Box::new(wa::message::ExtendedTextMessage {
                text: Some(quote_text),
                context_info: Some(Box::new(context_info)),
                ..Default::default()
            })),
            ..Default::default()
        })
    }

    /// !react - React to message with emoji
    async fn cmd_react(ctx: &MessageContext<'_>, args: &[&str]) -> Result<wa::Message> {
        let emoji = args.first().unwrap_or(&"👍").to_string();

        use chrono::Utc;
        
        let reaction = wa::message::ReactionMessage {
            key: Some(wa::MessageKey {
                remote_jid: Some(ctx.info.source.chat.to_string()),
                id: Some(ctx.info.id.clone()),
                from_me: Some(ctx.info.source.is_from_me),
                participant: if ctx.info.source.is_group {
                    Some(ctx.info.source.sender.to_string())
                } else {
                    None
                },
            }),
            text: Some(emoji),
            sender_timestamp_ms: Some(Utc::now().timestamp_millis()),
            ..Default::default()
        };

        Ok(wa::Message {
            reaction_message: Some(reaction),
            ..Default::default()
        })
    }

    /// !delete - Delete message (revoke)
    async fn cmd_delete(ctx: &MessageContext<'_>, args: &[&str]) -> Result<wa::Message> {
        let msg_id = args.first().unwrap_or(&"");
        
        if msg_id.is_empty() {
            return Ok(wa::Message {
                conversation: Some("❌ Usage: !delete <message_id>".to_string()),
                ..Default::default()
            });
        }

        use ruwa::send::RevokeType;

        match ctx
            .client
            .revoke_message(
                ctx.info.source.chat.clone(),
                msg_id.to_string(),
                RevokeType::Sender,
            )
            .await
        {
            Ok(_) => Ok(wa::Message {
                conversation: Some("✅ Message deleted".to_string()),
                ..Default::default()
            }),
            Err(e) => Ok(wa::Message {
                conversation: Some(format!("❌ Failed to delete: {}", e)),
                ..Default::default()
            }),
        }
    }

    /// Unknown command handler
    async fn cmd_unknown() -> Result<wa::Message> {
        Ok(wa::Message {
            conversation: Some(
                "❌ Unknown command. Type !menu to see all available commands.".to_string(),
            ),
            ..Default::default()
        })
    }
}

// Helper macro for logging
macro_rules! info {
    ($($arg:tt)*) => {
        log::info!($($arg)*)
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        log::error!($($arg)*)
    };
}
