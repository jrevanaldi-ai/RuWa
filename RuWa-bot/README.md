# RuWa Bot

🤖 **Example WhatsApp Bot using RuWa Library**

A complete, production-ready WhatsApp bot example built with RuWa.

---

## 🚀 Quick Start

### 1. Run the Bot

```bash
# Navigate to bot directory
cd RuWa-bot

# Run with QR code
cargo run

# Run with pair code (phone number)
cargo run -- --phone 6281234567890

# Run with custom pair code
cargo run -- -p 6281234567890 --code ABCD1234
```

### 2. Scan QR or Enter Pair Code

- **QR Code**: Scan with WhatsApp > Linked Devices > Link a Device
- **Pair Code**: Enter the 8-digit code in WhatsApp

### 3. Bot is Ready!

Once connected, the bot will respond to commands.

---

## 📖 Available Commands

### General Commands

| Command | Description | Example |
|---------|-------------|---------|
| `!menu` | Show all commands | `!menu` |
| `!help` | Get help | `!help` |
| `!ping` | Test bot response | `!ping` |
| `!info` | Bot information | `!info` |
| `!me` | Your information | `!me` |

### Group Commands

| Command | Description | Example |
|---------|-------------|---------|
| `!group info` | Group information | `!group info` |
| `!group list` | List participants | `!group list` |
| `!group admin` | List admins | `!group admin` |

### Fun Commands

| Command | Description | Example |
|---------|-------------|---------|
| `!quote <msg>` | Quote a message | `!quote Hello` |
| `!react <emoji>` | React with emoji | `!react 👍` |
| `!delete <id>` | Delete message | `!delete MSG123` |

---

## 📁 Project Structure

```
RuWa-bot/
├── Cargo.toml          # Dependencies
├── src/
│   ├── main.rs         # Main entry point
│   ├── commands.rs     # Command handlers
│   └── config.rs       # Configuration
├── examples/
│   └── advanced.rs     # Advanced examples
└── config.toml         # Configuration file (optional)
```

---

## ⚙️ Configuration

Create `config.toml` for custom settings:

```toml
# Database path
database_path = "ruwa-bot.db"

# Log level: error, warn, info, debug, trace
log_level = "info"

# Auto-reply (not implemented yet)
auto_reply = false
auto_reply_message = "Thank you for your message!"
```

---

## 🛠️ Build Options

### Development Build

```bash
cargo build
```

### Release Build (Optimized)

```bash
cargo build --release
```

### Run with Logging

```bash
# Debug logging
RUST_LOG=debug cargo run

# Info logging (default)
RUST_LOG=info cargo run

# Only show errors
RUST_LOG=error cargo run
```

---

## 📝 Example Output

```
00:00:00 [INFO ] ruwa_bot - 🚀 Starting RuWa Bot v1.0.0
00:00:00 [INFO ] ruwa_bot - 👤 Developer: Nathan
00:00:00 [INFO ] ruwa_bot - 📱 WhatsApp Bot using RuWa library
00:00:00 [INFO ] ruwa_bot - 📦 Initializing database...
00:00:00 [INFO ] ruwa_bot - ✅ Database initialized: ruwa-bot.db
00:00:00 [INFO ] ruwa_bot - 🤖 Building bot...
00:00:01 [INFO ] ruwa_bot - 📱 QR Code received (valid for 60s)
00:00:01 [INFO ] ruwa_bot - ==================================================
00:00:01 [INFO ] ruwa_bot - [QR CODE HERE]
00:00:01 [INFO ] ruwa_bot - ==================================================
00:00:20 [INFO ] ruwa_bot - ✅ Pairing successful!
00:00:21 [INFO ] ruwa_bot - ✅ Bot connected and ready!
```

---

## 🎯 Features

- ✅ **Command Handler** - Easy to extend with new commands
- ✅ **Group Support** - Manage groups, list participants
- ✅ **Message Reactions** - React with emoji
- ✅ **Quote Messages** - Reply with context
- ✅ **Delete Messages** - Revoke messages
- ✅ **Logging** - Comprehensive logging with levels
- ✅ **Configuration** - TOML-based config file
- ✅ **Error Handling** - Proper error handling with anyhow

---

## 🔧 Customization

### Add New Command

Edit `src/commands.rs`:

```rust
// Add new command handler
async fn cmd_custom() -> Result<wa::Message> {
    Ok(wa::Message {
        conversation: Some("Custom command response".to_string()),
        ..Default::default()
    })
}

// Add to match statement in handle()
"!custom" => Self::cmd_custom().await,
```

### Change Bot Behavior

Edit `src/main.rs` event handler:

```rust
.on_event(|event, client| async move {
    match event {
        Event::Message(msg, info) => {
            // Custom message handling
        }
        _ => {}
    }
})
```

---

## 🐛 Troubleshooting

### Bot doesn't respond

1. Check if bot is connected: Look for "✅ Bot connected"
2. Check command prefix: Must start with `!`
3. Check logging: Run with `RUST_LOG=debug`

### QR code not showing

1. Delete database: `rm ruwa-bot.db`
2. Restart bot
3. Check event handler is registered

### Build errors

1. Update Rust: `rustup update`
2. Clean build: `cargo clean && cargo build`
3. Check dependencies: `cargo update`

---

## 📚 Resources

- [RuWa Documentation](https://docs.rs/ruwa)
- [RuWa GitHub](https://github.com/jrevanaldi-ai/ruwa)
- [Example Bot Source](../src/main.rs)

---

## 📬 Contact

- **WhatsApp**: +62 895-4166-02000
- **Telegram**: @AstraluneSuport
- **Email**: support@nathan.christmas

---

## ⚠️ Disclaimer

This is an **UNOFFICIAL** WhatsApp bot. Use at your own risk.
May violate WhatsApp Terms of Service.

---

**Copyright © 2026 Nathan** | Made with ❤️ using Rust
