# RuWa

> **WhatsApp Web API dalam Rust** - Library paling lengkap untuk membuat bot WhatsApp

<div align="center">

[![Version](https://img.shields.io/crates/v/ruwa.svg?style=for-the-badge&logo=rust)](https://crates.io/crates/ruwa)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Build](https://img.shields.io/github/actions/workflow/status/jrevanaldi-ai/ruwa/main.yml?style=for-the-badge&logo=github)](https://github.com/jrevanaldi-ai/ruwa/actions)
[![Downloads](https://img.shields.io/crates/d/ruwa?style=for-the-badge&color=orange)](https://crates.io/crates/ruwa)

</div>

---

<div align="center">

## 📬 Kontak Developer

[![WhatsApp](https://img.shields.io/badge/WhatsApp-25D366?style=for-the-badge&logo=whatsapp&logoColor=white)](https://wa.me/62895416602000)
[![Instagram](https://img.shields.io/badge/Instagram-E4405F?style=for-the-badge&logo=instagram&logoColor=white)](https://instagram.com/chrisjoo_uww)
[![Telegram](https://img.shields.io/badge/Telegram-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white)](https://t.me/AstraluneSuport)
[![Email](https://img.shields.io/badge/Email-D14836?style=for-the-badge&logo=gmail&logoColor=white)](mailto:support@nathan.christmas)
[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/jrevanaldi-ai/ruwa)

> 🤖 **Note**: Dokumentasi dibuat dengan AI. Found a bug? Contact developer!

</div>

---

## 🎯 Kenapa RuWa?

| Feature | RuWa | Library Lain |
|---------|------|--------------|
| **Performance** | ⚡ Rust (native speed) | 🐢 Node.js/Python |
| **Memory Usage** | 💾 < 50MB | 📦 200MB+ |
| **Type Safety** | ✅ Compile-time checks | ❌ Runtime errors |
| **E2E Encryption** | ✅ Signal Protocol | ✅ Varies |
| **Multi-device** | ✅ Up to 4 devices | ⚠️ Limited |
| **Documentation** | 📖 Complete | ⚠️ Incomplete |

---

## 📑 Table of Contents

<details>
<summary><b>📖 Klik untuk expand semua section</b></summary>

1. [🚀 Quick Start](#-quick-start) - Mulai dalam 5 menit
2. [📦 Instalasi Lengkap](#-instalasi-lengkap) - Step-by-step
3. [✨ Fitur Lengkap](#-fitur-lengkap) - Semua yang tersedia
4. [🔐 Autentikasi](#-autentikasi) - QR & Pair code
5. [💬 Messaging](#-messaging) - Kirim & terima pesan
6. [📸 Media](#-media) - Upload & download
7. [👥 Grup Management](#-grup-management)
8. [📊 Presence & Receipts](#-presence--receipts)
9. [⚙️ Advanced Usage](#-advanced-usage)
10. [🐛 Troubleshooting](#-troubleshooting)
11. [❓ FAQ](#-faq)
12. [⚠️ Disclaimer](#-disclaimer)

</details>

---

## 🚀 Quick Start

### Mulai dalam 5 Menit

```bash
# 1. Clone atau buat project baru
cargo new my-whatsapp-bot
cd my-whatsapp-bot

# 2. Tambahkan dependencies
cargo add ruwa ruwa-sqlite-storage ruwa-tokio-transport ruwa-ureq-http-client
cargo add tokio --features full
```

### Contoh Bot Sederhana

```rust
// src/main.rs
use ruwa::bot::Bot;
use ruwa::store::SqliteStore;
use ruwa_tokio_transport::TokioWebSocketTransportFactory;
use ruwa_ureq_http_client::UreqHttpClient;
use wacore::types::events::Event;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup storage
    let store = SqliteStore::new("whatsapp.db").await?;
    
    // 2. Build bot
    let mut bot = Bot::builder()
        .with_backend(store)
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .on_event(|event, client| async move {
            match event {
                // QR Code received
                Event::PairingQrCode { code, timeout } => {
                    println!("📱 Scan QR code (valid {} detik):", timeout.as_secs());
                    println!("{}", code);
                }
                
                // Pair code received
                Event::PairingCode { code, timeout } => {
                    println!("🔢 Masukkan kode ini: {}", code);
                }
                
                // Message received
                Event::Message(msg, info) => {
                    println!("💬 Pesan dari {}", info.source.sender);
                    
                    // Auto reply "ping" dengan "pong"
                    if let Some(text) = msg.text_content() {
                        if text.to_lowercase() == "ping" {
                            let _ = client.send_message(
                                info.source.chat,
                                waproto::whatsapp::Message {
                                    conversation: Some("🏓 Pong!".to_string()),
                                    ..Default::default()
                                }
                            ).await;
                        }
                    }
                }
                
                // Connected
                Event::Connected(_) => {
                    println!("✅ Bot connected & ready!");
                }
                
                _ => {}
            }
        })
        .build()
        .await?;
    
    // 3. Run bot
    bot.run().await?.await?;
    
    Ok(())
}
```

### Jalankan

```bash
# Run dengan QR code
cargo run

# Run dengan pair code (nomor telepon)
cargo run -- --phone 6281234567890

# Run dengan custom pair code
cargo run -- -p 6281234567890 --code ABCD1234
```

---

## 📦 Instalasi Lengkap

### 1. Cargo.toml

```toml
[package]
name = "my-whatsapp-bot"
version = "0.1.0"
edition = "2021"

[dependencies]
# RuWa core
ruwa = "1.0.0"

# Storage (pilih salah satu)
ruwa-sqlite-storage = "1.0.0"  # SQLite (recommended)

# Transport
ruwa-tokio-transport = "1.0.0"  # WebSocket

# HTTP Client
ruwa-ureq-http-client = "1.0.0"

# Async runtime
tokio = { version = "1.48", features = ["full"] }

# Utilities
log = "0.4"
env_logger = "0.11"
chrono = "0.4"
```

### 2. System Requirements

```bash
# Install Rust (Linux/Mac)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Rust (Windows)
# Download dari https://rustup.rs/

# Install Rust (Termux/Android)
pkg install rust

# Install protoc (Protocol Buffers)
# Ubuntu/Debian
sudo apt install protobuf-compiler

# macOS
brew install protobuf

# Windows
# Download dari https://github.com/protocolbuffers/protobuf/releases

# Verify installation
rustc --version    # Should be 1.70+
cargo --version
protoc --version
```

### 3. Build & Run

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run
cargo run

# Run with logging
RUST_LOG=info cargo run
```

---

## ✨ Fitur Lengkap

### 🔐 Autentikasi

| Method | Deskripsi | Status |
|--------|-----------|--------|
| **QR Code** | Scan QR dari WhatsApp | ✅ Ready |
| **Pair Code** | 8-digit code via phone | ✅ Ready |
| **Session Persist** | Auto-reconnect | ✅ Ready |
| **Multi-device** | Link up to 4 devices | ✅ Ready |

### 💬 Messaging

| Feature | Status | Example |
|---------|--------|---------|
| Text messages | ✅ | `conversation: Some("Hello")` |
| Image messages | ✅ | `image_message: Some(...)` |
| Video messages | ✅ | `video_message: Some(...)` |
| Audio messages | ✅ | `audio_message: Some(...)` |
| Document messages | ✅ | `document_message: Some(...)` |
| Location messages | ✅ | `location_message: Some(...)` |
| Contact messages | ✅ | `contact_message: Some(...)` |
| Sticker messages | ⚠️ | Partial support |
| Edit messages | ✅ | `edit_message(id, new_msg)` |
| Delete messages | ✅ | `revoke_message(id)` |
| Reply/Quote | ✅ | `context_info: Some(...)` |
| Reactions | ✅ | `reaction_message: Some(...)` |

### 📸 Media

| Type | Upload | Download | Status |
|------|--------|----------|--------|
| Images | ✅ | ✅ | Full support |
| Videos | ✅ | ✅ | Full support |
| Audio | ✅ | ✅ | Full support |
| Documents | ✅ | ✅ | Full support |
| GIFs | ✅ | ✅ | Full support |
| Stickers | ⚠️ | ⚠️ | Partial |

### 👥 Grup

| Feature | Status | Method |
|---------|--------|--------|
| Create group | ✅ | `groups().create()` |
| List groups | ✅ | `groups().list()` |
| Get metadata | ✅ | `groups().get_metadata()` |
| Add participants | ✅ | `groups().add_participants()` |
| Remove participants | ✅ | `groups().remove_participants()` |
| Promote to admin | ✅ | `groups().promote_participants()` |
| Demote admin | ✅ | `groups().demote_participants()` |
| Update subject | ✅ | `groups().set_subject()` |
| Update description | ✅ | `groups().set_description()` |
| Set ephemeral | ✅ | `groups().set_ephemeral()` |
| Invite link | ✅ | `groups().get_invite_link()` |
| Leave group | ✅ | `groups().leave()` |

### 📊 Presence & Receipts

| Feature | Status |
|---------|--------|
| Set online/offline | ✅ |
| Subscribe presence | ✅ |
| Typing indicator | ✅ |
| Recording indicator | ✅ |
| Read receipts | ✅ |
| Delivery receipts | ✅ |
| Played receipts (audio) | ✅ |

---

## 🔐 Autentikasi Detail

### Method 1: QR Code (Default)

```rust
Bot::builder()
    .with_backend(store)
    .on_event(|event, _client| async move {
        if let Event::PairingQrCode { code, timeout } = event {
            println!("QR Code valid untuk {} detik", timeout.as_secs());
            println!("{}", code);
            // QR akan auto-rotate jika expired
        }
        
        if let Event::PairSuccess(_) = event {
            println!("✅ Pairing berhasil!");
        }
    })
    .build()
    .await?
```

### Method 2: Pair Code (Phone Number)

```rust
use ruwa::bot::{Bot, PairCodeOptions};
use ruwa::pair_code::PlatformId;

Bot::builder()
    .with_backend(store)
    .with_pair_code(PairCodeOptions {
        phone_number: "6281234567890".to_string(),
        custom_code: None,  // Atau Some("ABCD1234") untuk custom
        platform: PlatformId::Chrome,  // Chrome, Firefox, Edge, Safari
    })
    .on_event(|event, _client| async move {
        if let Event::PairingCode { code, timeout } = event {
            println!("Masukkan kode di WhatsApp:");
            println!("WhatsApp > Linked Devices > Link a Device");
            println!("> Link with phone number instead");
            println!("Kode: {}", code);
        }
    })
    .build()
    .await?
```

### Method 3: Session Persist (Auto-login)

```rust
// Session otomatis tersimpan di database
// Restart aplikasi = auto reconnect tanpa pairing ulang

let bot = Bot::builder()
    .with_backend(SqliteStore::new("whatsapp.db").await?)
    .build()
    .await?;

// Jika session masih valid, langsung connected
// Jika expired, akan trigger QR/Pair code
```

---

## 💬 Messaging Detail

### Kirim Pesan Teks

```rust
use ruwa::Jid;
use waproto::whatsapp as wa;

let chat_id: Jid = "6281234567890@s.whatsapp.net".parse()?;

let msg_id = client.send_message(chat_id, wa::Message {
    conversation: Some("Hello, World!".to_string()),
    ..Default::default()
}).await?;

println!("Pesan terkirim dengan ID: {}", msg_id);
```

### Reply/Quote Pesan

```rust
use wacore::proto_helpers::build_quote_context;

// Build quote context
let quote_context = build_quote_context(
    &original_msg_id,      // ID pesan yang di-quote
    &original_sender_jid,  // JID pengirim pesan original
    &chat_jid,             // JID chat
    &original_message      // Pesan original
);

// Kirim reply
client.send_message(chat_jid, wa::Message {
    extended_text_message: Some(Box::new(
        wa::message::ExtendedTextMessage {
            text: Some("Ini balasan".to_string()),
            context_info: Some(Box::new(quote_context)),
            ..Default::default()
        }
    )),
    ..Default::default()
}).await?;
```

### Reaction (Emoji)

```rust
use chrono::Utc;

client.send_message(chat_jid, wa::Message {
    reaction_message: Some(wa::message::ReactionMessage {
        key: Some(wa::MessageKey {
            remote_jid: Some(chat_jid.to_string()),
            id: Some(msg_id.to_string()),
            from_me: Some(false),
            participant: None,
        }),
        text: Some("👍".to_string()),
        sender_timestamp_ms: Some(Utc::now().timestamp_millis()),
        ..Default::default()
    }),
    ..Default::default()
}).await?;
```

### Edit Pesan

```rust
let new_message = wa::Message {
    conversation: Some("Pesan yang sudah diedit".to_string()),
    ..Default::default()
};

client.edit_message(
    chat_jid,
    original_msg_id,
    new_message
).await?;
```

### Delete Pesan (Revoke)

```rust
use ruwa::send::RevokeType;

// Delete pesan sendiri
client.revoke_message(
    chat_jid,
    msg_id.to_string(),
    RevokeType::Sender,
).await?;

// Admin delete pesan user lain (di grup)
client.revoke_message(
    group_jid,
    msg_id.to_string(),
    RevokeType::Admin { 
        original_sender: sender_jid 
    },
).await?;
```

### Forward Pesan

```rust
// Forward ke chat lain
client.send_message(
    target_chat_jid,
    original_message.clone()  // Clone pesan original
).await?;
```

---

## 📸 Media Detail

### Upload & Kirim Gambar

```rust
use ruwa::download::MediaType;

// 1. Baca file
let image_data = std::fs::read("foto.jpg")?;

// 2. Upload ke WhatsApp server
let upload = client.upload(image_data, MediaType::Image).await?;

// 3. Build message
let message = wa::Message {
    image_message: Some(Box::new(wa::message::ImageMessage {
        url: Some(upload.url),
        direct_path: Some(upload.direct_path),
        media_key: Some(upload.media_key),
        mimetype: Some("image/jpeg".to_string()),
        file_enc_sha256: Some(upload.file_enc_sha256),
        file_sha256: Some(upload.file_sha256),
        file_length: Some(upload.file_length),
        caption: Some("Caption untuk gambar".to_string()),
        ..Default::default()
    })),
    ..Default::default()
};

// 4. Kirim
client.send_message(chat_jid, message).await?;
```

### Download Media

```rust
use ruwa::download::{Downloadable, MediaType};
use std::io::Cursor;

// Dapatkan message yang punya media
if let Some(image_msg) = &message.image_message {
    // Download ke buffer
    let mut buffer = Cursor::new(Vec::new());
    client.download_to_file(image_msg, &mut buffer).await?;
    
    // Simpan ke file
    let data = buffer.into_inner();
    std::fs::write("downloaded.jpg", &data)?;
    
    println!("Media berhasil didownload: {} bytes", data.len());
}
```

### Upload Video

```rust
let video_data = std::fs::read("video.mp4")?;

let upload = client.upload(video_data, MediaType::Video).await?;

client.send_message(chat_jid, wa::Message {
    video_message: Some(Box::new(wa::message::VideoMessage {
        url: Some(upload.url),
        direct_path: Some(upload.direct_path),
        media_key: Some(upload.media_key),
        mimetype: Some("video/mp4".to_string()),
        file_enc_sha256: Some(upload.file_enc_sha256),
        file_sha256: Some(upload.file_sha256),
        file_length: Some(upload.file_length),
        caption: Some("Video caption".to_string()),
        gif_playback: Some(false),
        ..Default::default()
    })),
    ..Default::default()
}).await?;
```

### Upload Audio/Voice Note

```rust
let audio_data = std::fs::read("audio.ogg")?;

let upload = client.upload(audio_data, MediaType::Audio).await?;

client.send_message(chat_jid, wa::Message {
    audio_message: Some(Box::new(wa::message::AudioMessage {
        url: Some(upload.url),
        direct_path: Some(upload.direct_path),
        media_key: Some(upload.media_key),
        mimetype: Some("audio/ogg; codecs=opus".to_string()),
        file_enc_sha256: Some(upload.file_enc_sha256),
        file_sha256: Some(upload.file_sha256),
        file_length: Some(upload.file_length),
        seconds: Some(30),  // Durasi dalam detik
        ..Default::default()
    })),
    ..Default::default()
}).await?;
```

### Upload Document

```rust
let doc_data = std::fs::read("document.pdf")?;

let upload = client.upload(doc_data, MediaType::Document).await?;

client.send_message(chat_jid, wa::Message {
    document_message: Some(Box::new(wa::message::DocumentMessage {
        url: Some(upload.url),
        direct_path: Some(upload.direct_path),
        media_key: Some(upload.media_key),
        mimetype: Some("application/pdf".to_string()),
        file_enc_sha256: Some(upload.file_enc_sha256),
        file_sha256: Some(upload.file_sha256),
        file_length: Some(upload.file_length),
        title: Some("document.pdf".to_string()),
        caption: Some("Document caption".to_string()),
        ..Default::default()
    })),
    ..Default::default()
}).await?;
```

---

## 👥 Grup Management Detail

### Buat Grup Baru

```rust
use ruwa::features::{Groups, GroupCreateOptions};

let participants = vec![
    "6281234567890@s.whatsapp.net".parse()?,
    "6289876543210@s.whatsapp.net".parse()?,
];

let result = client.groups().create(
    "Nama Grup",
    participants,
    GroupCreateOptions {
        description: Some("Deskripsi grup".to_string()),
        ephemeral: Some(86400),  // 24 jam
        ..Default::default()
    }
).await?;

println!("Grup dibuat: {}", result.jid);
```

### Dapatkan Info Grup

```rust
let metadata = client.groups().get_metadata(group_jid).await?;

println!("Nama: {}", metadata.subject);
println!("Deskripsi: {:?}", metadata.description);
println!("Owner: {}", metadata.owner);
println!("Created: {}", metadata.creation);

// List participants
for participant in &metadata.participants {
    println!("  - {} (Admin: {})", participant.jid, participant.is_admin);
}
```

### Manage Participants

```rust
let groups = client.groups();

// Add participants
groups.add_participants(group_jid, vec![
    "6281234567890@s.whatsapp.net".parse()?,
]).await?;

// Remove participants
groups.remove_participants(group_jid, vec![
    "6289876543210@s.whatsapp.net".parse()?,
]).await?;

// Promote to admin
groups.promote_participants(group_jid, vec![
    "6281234567890@s.whatsapp.net".parse()?,
]).await?;

// Demote admin
groups.demote_participants(group_jid, vec![
    "6289876543210@s.whatsapp.net".parse()?,
]).await?;
```

### Update Group Settings

```rust
let groups = client.groups();

// Update nama
groups.set_subject(group_jid, "Nama Grup Baru").await?;

// Update deskripsi
groups.set_description(group_jid, "Deskripsi baru").await?;

// Set disappearing messages (24 jam)
groups.set_ephemeral(group_jid, Some(86400)).await?;

// Disable disappearing messages
groups.set_ephemeral(group_jid, None).await?;

// Get invite link
let invite_link = groups.get_invite_link(group_jid).await?;
println!("Invite link: {}", invite_link);

// Revoke invite link
groups.revoke_invite_link(group_jid).await?;

// Leave group
groups.leave(group_jid).await?;
```

---

## 📊 Presence & Receipts

### Set Presence

```rust
// Set online
client.presence().set_available(true).await?;

// Set offline
client.presence().set_available(false).await?;

// Subscribe presence kontak
client.presence().subscribe_presence(contact_jid).await?;

// Unsubscribe
client.presence().unsubscribe_presence(contact_jid).await?;
```

### Typing Indicator

```rust
use ruwa::features::{Chatstate, ChatStateType};

// Mulai mengetik
client.chatstate().send_chatstate(
    chat_jid,
    ChatStateType::Composing,
).await?;

// Stop mengetik (auto setelah 3 detik)
client.chatstate().send_chatstate(
    chat_jid,
    ChatStateType::Paused,
).await?;

// Recording audio
client.chatstate().send_chatstate(
    chat_jid,
    ChatStateType::Recording,
).await?;
```

### Handle Receipts

```rust
.on_event(|event, _client| async move {
    if let Event::Receipt(receipt) = event {
        match receipt.r#type {
            // Pesan terkirim ke server
            wa::receipt::Type::Delivery => {
                println!("Pesan {:?} delivered", receipt.message_ids);
            }
            // Pesan sudah dibaca
            wa::receipt::Type::Read => {
                println!("Pesan {:?} read", receipt.message_ids);
            }
            // Audio sudah diputar
            wa::receipt::Type::Played => {
                println!("Audio {:?} played", receipt.message_ids);
            }
            _ => {}
        }
    }
})
```

---

## ⚙️ Advanced Usage

### Custom Event Handler

```rust
use ruwa::bot::Bot;
use std::sync::Arc;

let bot = Bot::builder()
    .with_backend(store)
    .on_event(|event, client| async move {
        match event {
            Event::Message(msg, info) => {
                // Handle message
                tokio::spawn(async move {
                    // Process in background
                });
            }
            _ => {}
        }
    })
    .build()
    .await?;
```

### Multiple Clients

```rust
// Run multiple bots/accounts
let bot1 = Bot::builder()
    .with_backend(SqliteStore::new("account1.db").await?)
    .build()
    .await?;

let bot2 = Bot::builder()
    .with_backend(SqliteStore::new("account2.db").await?)
    .build()
    .await?;

// Run concurrently
tokio::try_join!(bot1.run(), bot2.run())?;
```

### Custom Cache Config

```rust
use ruwa::{CacheConfig, CacheEntryConfig};
use std::time::Duration;

let cache_config = CacheConfig {
    group_cache: CacheEntryConfig::new()
        .with_ttl(Duration::from_secs(3600))
        .with_capacity(1000),
    device_cache: CacheEntryConfig::new()
        .with_ttl(Duration::from_secs(3600))
        .with_capacity(5000),
    ..Default::default()
};

let bot = Bot::builder()
    .with_backend(store)
    .with_cache_config(cache_config)
    .build()
    .await?;
```

---

## 🐛 Troubleshooting

### Build Errors

#### Error: SIMD feature requires nightly

**Problem:**
```
error[E0554]: `#![feature]` may not be used on the stable release channel
```

**Solution:**
Feature SIMD sudah di-disable by default. Pastikan tidak enable SIMD di Cargo.toml:

```toml
# ❌ Jangan enable SIMD
ruwa = { version = "1.0.0", features = ["simd"] }

# ✅ Gunakan default (tanpa SIMD)
ruwa = "1.0.0"
```

#### Error: protoc not found

**Problem:**
```
failed to execute command: protoc: No such file or directory
```

**Solution:**
```bash
# Ubuntu/Debian
sudo apt install protobuf-compiler

# macOS
brew install protobuf

# Verify
protoc --version
```

### Runtime Errors

#### QR Code tidak muncul

**Problem:** Event `PairingQrCode` tidak trigger

**Solution:**
1. Pastikan session lama dihapus: `rm whatsapp.db`
2. Check event handler sudah benar
3. Enable logging: `RUST_LOG=debug cargo run`

#### Connection failed

**Problem:**
```
Error: Connection refused
```

**Solution:**
1. Check internet connection
2. Check firewall tidak block WebSocket
3. Session expired, delete database dan pair ulang

#### Message tidak terkirim

**Problem:** `send_message` return error

**Solution:**
1. Pastikan recipient JID benar: `number@s.whatsapp.net`
2. Check session sudah established (tunggu retry receipt)
3. Untuk grup, pastikan bot masih member

### Performance Issues

#### Memory usage tinggi

**Solution:**
```toml
# Build release untuk optimasi
cargo build --release

# Profile release di Cargo.toml
[profile.release]
opt-level = 3
lto = true
```

#### Slow message processing

**Solution:**
```rust
// Process messages async
.on_event(|event, client| async move {
    if let Event::Message(msg, info) = event {
        let client_clone = client.clone();
        tokio::spawn(async move {
            // Process in background
            process_message(client_clone, msg, info).await;
        });
    }
})
```

---

## ❓ FAQ

<details>
<summary><b>📱 Apakah ini resmi dari WhatsApp?</b></summary>

❌ **TIDAK**. Ini adalah **unofficial library** hasil reverse engineering WhatsApp Web. Penggunaan dapat melanggar ToS WhatsApp.
</details>

<details>
<summary><b>⚠️ Apakah akun bisa di-banned?</b></summary>

⚠️ **YA, ada kemungkinan**. WhatsApp dapat detect client tidak resmi dan melakukan suspend/ban. 

**Tips untuk minimize risk:**
- Jangan spam
- Jangan kirim message massal
- Gunakan delay antar message
- Hanya untuk testing/development
</details>

<details>
<summary><b>💻 Support platform apa saja?</b></summary>

✅ **Support semua platform:**
- Linux
- macOS
- Windows
- Android (via Termux)
</details>

<details>
<summary><b>📦 Berapa ukuran binary?</b></summary>

📊 **Ukuran approximate:**
- Debug build: ~300MB
- Release build: ~50MB (dengan LTO)
</details>

<details>
<summary><b>🔄 Apakah support multi-device?</b></summary>

✅ **Ya**, support hingga 4 perangkat companion (seperti WhatsApp Web linked devices).
</details>

<details>
<summary><b>🤖 Bisa untuk bot WhatsApp?</b></summary>

✅ **Sangat bisa!** Library ini dirancang khusus untuk memudahkan pembuatan bot WhatsApp dengan berbagai fitur lengkap.
</details>

<details>
<summary><b>📚 Apakah ada contoh bot?</b></summary>

✅ Lihat folder `examples/` di repository atau contoh di section [Quick Start](#-quick-start).
</details>

<details>
<summary><b>🐛 Bagaimana cara report bug?</b></summary>

📬 Hubungi developer via kontak di atas atau buat issue di GitHub.
</details>

---

## ⚠️ Disclaimer

> **⚠️ PENTING: Gunakan dengan risiko sendiri!**

### Legal Notice

Library ini adalah **UNOFFICIAL** implementasi WhatsApp Web API. Penggunaan library ini dapat:

- ❌ Melanggar [WhatsApp Terms of Service](https://www.whatsapp.com/legal/terms-of-service)
- ❌ Melanggar [WhatsApp Business Policy](https://www.whatsapp.com/legal/business-policy)
- ⚠️ Mengakibatkan **suspend** atau **ban** akun WhatsApp Anda

### Recommended Usage

**✅ GUNAKAN HANYA UNTUK:**
- Testing & development
- Educational purposes
- Research
- Personal projects (dengan risiko sendiri)

**❌ JANGAN GUNAKAN UNTUK:**
- Spam atau bulk messaging
- Scam atau fraud
- Aktivitas ilegal
- Pelanggaran privasi orang lain
- Commercial use tanpa izin dari Meta

### No Warranty

Library ini disediakan **"AS IS"** tanpa warranty apapun. Developer tidak bertanggung jawab atas:
- Kerugian yang timbul dari penggunaan
- Suspend/ban akun
- Kehilangan data
- Masalah hukum yang mungkin timbul

---

## 📚 Resources

| Resource | Link |
|----------|------|
| 📖 API Documentation | [docs.rs/ruwa](https://docs.rs/ruwa) |
| 💻 Source Code | [github.com/jrevanaldi-ai/ruwa](https://github.com/jrevanaldi-ai/ruwa) |
| 📦 Package Registry | [crates.io/crates/ruwa](https://crates.io/crates/ruwa) |
| 🐛 Issue Tracker | [GitHub Issues](https://github.com/jrevanaldi-ai/ruwa/issues) |
| 💬 Discussions | [GitHub Discussions](https://github.com/jrevanaldi-ai/ruwa/discussions) |

---

## 🙏 Acknowledgements

Terima kasih kepada project yang menginspirasi:

- [**whatsmeow**](https://github.com/tulir/whatsmeow) - Go WhatsApp library (tulir)
- [**Baileys**](https://github.com/WhiskeySockets/Baileys) - TypeScript WhatsApp library
- **WhatsApp Web** - Sebagai referensi protokol

- *Thanks To https://github.com/jlucaso1/whatsapp-rust/ sebagai base utama dari project ini*
---

<div align="center">

### 📬 Still Have Questions?

[![WhatsApp](https://img.shields.io/badge/WhatsApp-25D366?style=for-the-badge&logo=whatsapp&logoColor=white)](https://wa.me/62895416602000)
[![Telegram](https://img.shields.io/badge/Telegram-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white)](https://t.me/AstraluneSuport)
[![Email](https://img.shields.io/badge/Email-D14836?style=for-the-badge&logo=gmail&logoColor=white)](mailto:support@nathan.christmas)

---

**Copyright © 2026 Nathan**

[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)

Made with ❤️ using Rust

</div>
