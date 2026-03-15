# RuWa

[![Crates.io](https://img.shields.io/crates/v/ruwa.svg)](https://crates.io/crates/ruwa)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust CI](https://github.com/jrevanaldi-ai/ruwa/actions/workflows/main.yml/badge.svg)](https://github.com/jrevanaldi-ai/ruwa/actions)

**Implementasi WhatsApp Web API dalam Rust yang berkinerja tinggi dan async.** Terinspirasi oleh [whatsmeow](https://github.com/tulir/whatsmeow) (Go) dan [Baileys](https://github.com/WhiskeySockets/Baileys) (TypeScript).

> 🤖 **Catatan Dokumentasi**: Dokumentasi ini dibuat dengan bantuan AI. Jika Anda menemukan kesalahan atau ada yang kurang jelas, jangan ragu untuk menghubungi developer.

---

## 📬 Kontak & Support

Untuk pertanyaan lebih spesifik tentang library RuWa, bug reports, feature requests, atau kolaborasi, silakan hubungi developer:

<div align="center">

[![WhatsApp](https://img.shields.io/badge/WhatsApp-25D366?style=for-the-badge&logo=whatsapp&logoColor=white)](https://wa.me/62895416602000)
[![Instagram](https://img.shields.io/badge/Instagram-E4405F?style=for-the-badge&logo=instagram&logoColor=white)](https://instagram.com/chrisjoo_uww)
[![Telegram](https://img.shields.io/badge/Telegram-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white)](https://t.me/AstraluneSuport)
[![Email](https://img.shields.io/badge/Email-D14836?style=for-the-badge&logo=gmail&logoColor=white)](mailto:support@nathan.christmas)

</div>

---

## 📋 Daftar Isi

- [Kontak & Support](#-kontak--support)
- [Fitur](#-fitur)
- [Quick Start](#-quick-start)
- [Instalasi](#-instalasi)
- [Arsitektur Proyek](#-arsitektur-proyek)
- [Panduan Penggunaan](#-panduan-penggunaan)
  - [Autentikasi](#autentikasi)
  - [Mengirim Pesan](#mengirim-pesan)
  - [Menerima Pesan](#menerima-pesan)
  - [Media](#media)
  - [Grup](#grup)
  - [Kontak & Presence](#kontak--presence)
- [Struktur Proyek](#-struktur-proyek)
- [Konvensi Penting](#-konvensi-penting)
- [Testing](#-testing)
- [Custom Backend](#-custom-backend)
- [Troubleshooting](#-troubleshooting)
- [FAQ](#-faq-frequently-asked-questions)
- [Disclaimer](#-disclaimer)
- [Acknowledgements](#-acknowledgements)

---

## ✨ Fitur

### 🔐 Autentikasi
- ✅ QR code pairing dengan rotasi otomatis
- ✅ Pair code (kode 8 digit) untuk linking via nomor telepon
- ✅ Session persisten dengan auto-reconnect
- ✅ Multi-device support (hingga 4 perangkat companion)
- ✅ Edge routing untuk koneksi optimal

### 💬 Messaging
- ✅ Pesan E2E encrypted (Signal Protocol)
- ✅ Chat personal 1-on-1 dan grup
- ✅ Edit pesan, reactions (emoji), quotes/replies
- ✅ Delete message for everyone (revoke)
- ✅ Delivery, read, dan played receipts
- ✅ Retry otomatis dengan backoff
- ✅ PDO (Peer Data Operation) fallback

### 📸 Media
- ✅ Upload dan download: image, video, document, audio, GIF
- ✅ Enkripsi/dekripsi otomatis
- ✅ Buffer pooling untuk performa optimal
- ✅ Retry download pada kegagalan

### 👥 Grup & Kontak
- ✅ CRUD grup: create, list, metadata, delete
- ✅ Manage participants: add, remove, promote, demote
- ✅ Group settings: subject, description, announcement mode, locked, ephemeral
- ✅ Invite link dan membership approval
- ✅ Cek nomor WhatsApp, profile pictures, user info
- ✅ LID-PN mapping cache untuk resolusi device

### 📡 Presence & Chat State
- ✅ Set online/offline presence
- ✅ Subscribe/unsubscribe presence kontak
- ✅ Typing indicators: composing, recording, paused
- ✅ TTL chatstate (3 detik) seperti WhatsApp Web

### 🔄 Sync
- ✅ AppState sync: contacts, groups, chat actions
- ✅ History sync: initial, recent, full
- ✅ LT Hash integrity verification
- ✅ Offline message queue dan delivery

### 🛡️ Fitur Tambahan
- ✅ Blocklist (block/unblock contacts)
- ✅ Privacy settings
- ✅ Spam reporting
- ✅ Status/stories (text dan media)
- ✅ Disappearing messages
- ✅ Business notifications

---

## 🚀 Quick Start

### Contoh Minimalis

```rust
use std::sync::Arc;
use ruwa::bot::Bot;
use ruwa::store::SqliteStore;
use ruwa_tokio_transport::TokioWebSocketTransportFactory;
use ruwa_ureq_http_client::UreqHttpClient;
use wacore::types::events::Event;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inisialisasi storage SQLite
    let backend = Arc::new(SqliteStore::new("whatsapp.db").await?);

    // Build bot dengan event handler
    let mut bot = Bot::builder()
        .with_backend(backend)
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .on_event(|event, client| async move {
            match event {
                Event::PairingQrCode { code, timeout } => {
                    println!("QR Code (valid {} detik):\n{}", timeout.as_secs(), code);
                }
                Event::Message(msg, info) => {
                    println!("Pesan dari {}: {:?}", info.source.sender, msg);
                }
                Event::Connected(_) => {
                    println!("✅ Terhubung!");
                }
                _ => {}
            }
        })
        .build()
        .await?;

    // Jalankan bot
    bot.run().await?.await?;
    Ok(())
}
```

### Jalankan Demo Bot

```bash
# QR code pairing saja
cargo run

# Pair code + QR code (konkuren)
cargo run -- --phone 6281234567890

# Dengan custom pair code 8 digit
cargo run -- -p 6281234567890 --code ABCD1234
```

---

## 📦 Instalasi

Tambahkan ke `Cargo.toml`:

```toml
[dependencies]
ruwa = "1.0.0"
ruwa-sqlite-storage = "1.0.0"
ruwa-tokio-transport = "1.0.0"
ruwa-ureq-http-client = "1.0.0"
tokio = { version = "1.48", features = ["full"] }
```

### Requirements

- **Rust**: 1.70+ (stable)
- **protoc**: Protocol Buffers compiler (v3.25.3+)
- **SQLite**: Untuk storage backend

Install toolchain:

```bash
# Install Rust (jika belum terinstall)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Untuk pengguna Termux/Android:
pkg install rust
```

---

## 🏗️ Arsitektur Proyek

### Layer Stack

```
┌─────────────────────────────────────────────────┐
│  Bot API / Client (src/bot.rs, client.rs)       │
│  - BotBuilder pattern                           │
│  - Event handlers                               │
│  - High-level features (groups, contacts, dll)  │
├─────────────────────────────────────────────────┤
│  Stanza Handlers (src/handlers/)                │
│  - Router dispatch untuk XML stanzas            │
│  - Message, Receipt, IQ, Notification handlers  │
├─────────────────────────────────────────────────┤
│  Core Protocol (wacore/)                        │
│  - ProtocolNode & IqSpec traits                 │
│  - Signal Protocol encryption (libsignal/)      │
│  - Noise Protocol handshake (noise/)            │
│  - AppState synchronization (appstate/)         │
├─────────────────────────────────────────────────┤
│  Binary Protocol (wacore/binary/)               │
│  - Marshal/Unmarshal nodes WhatsApp             │
│  - Token mapping & nibble encoding              │
├─────────────────────────────────────────────────┤
│  Transport & Storage                            │
│  - WebSocket (tokio-transport/)                 │
│  - HTTP client (ureq-client/)                   │
│  - SQLite persistence (sqlite-storage/)         │
└─────────────────────────────────────────────────┘
```

### Workspace Crates

| Crate | Deskripsi |
|-------|-----------|
| `ruwa` | Crate utama dengan Tokio runtime dan high-level API |
| `wacore` | Core protocol (platform-agnostic, no_std compatible) |
| `wacore/binary` | WhatsApp binary protocol encoding/decoding |
| `wacore/libsignal` | Implementasi Signal Protocol |
| `wacore/noise` | Noise protocol untuk handshake |
| `wacore/appstate` | AppState synchronization dengan LT Hash |
| `wacore/derive` | Procedural macros (ProtocolNode, IqSpec, StringEnum) |
| `waproto` | Protocol Buffers definitions (whatsapp.proto) |
| `sqlite-storage` | SQLite backend dengan Diesel ORM |
| `tokio-transport` | WebSocket transport dengan Tokio |
| `ureq-client` | HTTP client menggunakan ureq |

---

## 📖 Panduan Penggunaan

### Autentikasi

#### QR Code Pairing

```rust
use ruwa::bot::Bot;

let mut bot = Bot::builder()
    .with_backend(backend)
    .on_event(|event, _client| async move {
        if let Event::PairingQrCode { code, timeout } = event {
            println!("Scan QR code ini (valid {} detik):", timeout.as_secs());
            println!("{}", code);
        }
    })
    .build()
    .await?;

bot.run().await?.await?;
```

#### Pair Code (Phone Number Linking)

```rust
use ruwa::bot::{Bot, PairCodeOptions};

let mut bot = Bot::builder()
    .with_backend(backend)
    .with_pair_code(PairCodeOptions {
        phone_number: "6281234567890".to_string(),
        custom_code: None, // Atau Some("ABCD1234".to_string())
        ..Default::default()
    })
    .on_event(|event, _client| async move {
        if let Event::PairingCode { code, timeout } = event {
            println!("Masukkan kode ini di WhatsApp (valid {} detik):", timeout.as_secs());
            println!("WhatsApp > Perangkat Tertaut > Tautkan Perangkat");
            println!("> Tautkan dengan nomor telepon saja");
            println!(">>> {} <<<", code);
        }
    })
    .build()
    .await?;
```

#### Session Persisten

Session disimpan otomatis di SQLite. Restart aplikasi akan reconnect otomatis tanpa perlu pairing ulang.

```rust
// Session akan otomatis reconnect jika masih valid
let bot = Bot::builder()
    .with_backend(backend)
    .build()
    .await?;
```

---

### Mengirim Pesan

#### Pesan Teks

```rust
use waproto::whatsapp as wa;
use wacore_binary::jid::Jid;

// Kirim pesan teks
let to: Jid = "6281234567890@s.whatsapp.net".parse()?;
let message = wa::Message {
    conversation: Some("Halo dari WhatsApp-Rust!".to_string()),
    ..Default::default()
};

let msg_id = client.send_message(to, message).await?;
println!("Pesan terkirim dengan ID: {}", msg_id);
```

#### Quote/Reply Pesan

```rust
use waproto::whatsapp as wa;
use wacore::proto_helpers::build_quote_context;

// Build context untuk quote
let quoted_msg = /* pesan yang di-quote */;
let context_info = build_quote_context(
    &quoted_msg_id,
    &quoted_sender_jid,
    &chat_jid,
    &quoted_msg
);

let message = wa::Message {
    extended_text_message: Some(Box::new(
        wa::message::ExtendedTextMessage {
            text: Some("Ini balasan".to_string()),
            context_info: Some(Box::new(context_info)),
            ..Default::default()
        }
    )),
    ..Default::default()
};

client.send_message(chat_jid, message).await?;
```

#### Reaction (Emoji)

```rust
use chrono::Utc;

let reaction = wa::message::ReactionMessage {
    key: Some(wa::MessageKey {
        remote_jid: Some(chat_jid.to_string()),
        id: Some(msg_id.to_string()),
        from_me: Some(false),
        participant: None,
    }),
    text: Some("👍".to_string()),
    sender_timestamp_ms: Some(Utc::now().timestamp_millis()),
    ..Default::default()
};

let message = wa::Message {
    reaction_message: Some(reaction),
    ..Default::default()
};

client.send_message(chat_jid, message).await?;
```

#### Edit Pesan

```rust
let original_msg_id = "ABC123";
let new_message = wa::Message {
    conversation: Some("Pesan yang sudah diedit".to_string()),
    ..Default::default()
};

client.edit_message(chat_jid, original_msg_id, new_message).await?;
```

#### Delete Message (Revoke)

```rust
use ruwa::send::RevokeType;

// Sender delete pesan sendiri
client.revoke_message(
    chat_jid,
    msg_id.to_string(),
    RevokeType::Sender,
).await?;

// Admin delete pesan user lain di grup
client.revoke_message(
    group_jid,
    msg_id.to_string(),
    RevokeType::Admin { original_sender: sender_jid },
).await?;
```

---

### Menerima Pesan

```rust
use wacore::types::events::Event;
use wacore::proto_helpers::MessageExt;

let bot = Bot::builder()
    .with_backend(backend)
    .on_event(|event, client| async move {
        match event {
            Event::Message(msg, info) => {
                println!("Pesan dari: {}", info.source.sender);
                println!("Chat: {}", info.source.chat);
                println!("ID: {}", info.id);
                
                // Ambil teks pesan
                if let Some(text) = msg.text_content() {
                    println!("Isi: {}", text);
                }
                
                // Cek apakah grup
                if info.source.is_group {
                    println!("Ini pesan grup");
                }
            }
            Event::Receipt(receipt) => {
                println!("Receipt untuk pesan {:?}: {:?}", 
                    receipt.message_ids, receipt.r#type);
            }
            _ => {}
        }
    })
    .build()
    .await?;
```

---

### Media

#### Upload Media

```rust
use wacore::download::MediaType;
use ruwa::upload::UploadResponse;

// Baca file
let image_data = std::fs::read("foto.jpg")?;

// Upload
let response: UploadResponse = client
    .upload(image_data, MediaType::Image)
    .await?;

println!("Media URL: {}", response.url);
println!("Direct path: {}", response.direct_path);
```

#### Download Media

```rust
use wacore::download::{Downloadable, MediaType};
use std::io::Cursor;

// Download ke buffer
let mut buffer = Cursor::new(Vec::new());
client.download_to_file(&image_message, &mut buffer).await?;

let data = buffer.into_inner();
std::fs::write("downloaded.jpg", &data)?;
```

#### Kirim Gambar

```rust
// Upload dulu
let image_data = std::fs::read("foto.jpg")?;
let upload = client.upload(image_data, MediaType::Image).await?;

// Build message
let message = wa::Message {
    image_message: Some(Box::new(wa::message::ImageMessage {
        url: Some(upload.url),
        direct_path: Some(upload.direct_path),
        media_key: Some(upload.media_key),
        mimetype: Some("image/jpeg".to_string()),
        file_enc_sha256: Some(upload.file_enc_sha256),
        file_sha256: Some(upload.file_sha256),
        file_length: Some(upload.file_length),
        caption: Some("Ini caption".to_string()),
        ..Default::default()
    })),
    ..Default::default()
};

client.send_message(chat_jid, message).await?;
```

---

### Grup

#### Buat Grup

```rust
use ruwa::features::{Groups, GroupCreateOptions};

let participants = vec![
    "6281234567890@s.whatsapp.net".parse()?,
    "6289876543210@s.whatsapp.net".parse()?,
];

let result = client.groups().create(
    "Nama Grup",
    participants,
    GroupCreateOptions::default(),
).await?;

println!("Grup dibuat: {}", result.jid);
```

#### Manage Grup

```rust
let groups = client.groups();

// List semua grup
let my_groups = groups.list().await?;

// Dapatkan metadata grup
let metadata = groups.get_metadata(group_jid).await?;
println!("Nama: {}", metadata.subject);
println!("Deskripsi: {:?}", metadata.description);
println!("Admin: {:?}", metadata.participants.iter()
    .filter(|p| p.is_admin).collect::<Vec<_>>());

// Tambah participant
groups.add_participants(group_jid, vec![new_jid]).await?;

// Remove participant
groups.remove_participants(group_jid, vec![jid_to_remove]).await?;

// Promote ke admin
groups.promote_participants(group_jid, vec![jid_to_promote]).await?;

// Demote admin
groups.demote_participants(group_jid, vec![jid_to_demote]).await?;

// Update nama grup
groups.set_subject(group_jid, "Nama Baru").await?;

// Update deskripsi
groups.set_description(group_jid, "Deskripsi baru").await?;

// Set ephemeral (disappearing messages)
groups.set_ephemeral(group_jid, Some(86400)).await?; // 24 jam

// Keluar dari grup
groups.leave(group_jid).await?;
```

---

### Kontak & Presence

#### Cek Nomor WhatsApp

```rust
use ruwa::features::Contacts;

let numbers = vec!["6281234567890", "6289876543210"];
let results = client.contacts().is_on_whatsapp(numbers).await?;

for result in results {
    if result.is_in {
        println!("{} ada di WhatsApp (JID: {})", result.query, result.jid);
    } else {
        println!("{} tidak ada di WhatsApp", result.query);
    }
}
```

#### Profile Picture

```rust
// Dapatkan URL profile picture
let pp_url = client.contacts()
    .get_profile_picture_url(user_jid, Some(64)) // preview size
    .await?;

// Upload profile picture baru
let image_data = std::fs::read("profile.jpg")?;
client.profile().set_profile_picture(image_data).await?;

// Hapus profile picture
client.profile().remove_profile_picture().await?;
```

#### Presence & Typing

```rust
use ruwa::features::{Presence, Chatstate, ChatStateType};

// Set online/offline
client.presence().set_available(true).await?;

// Subscribe presence kontak
client.presence().subscribe_presence(contact_jid).await?;

// Kirim typing indicator
client.chatstate().send_chatstate(
    chat_jid,
    ChatStateType::Composing, // atau Recording, Paused
).await?;

// Otomatis stop setelah 3 detik (TTL)
```

---

## 📁 Struktur Proyek

```
ruwa/
├── Cargo.toml                    # Workspace manifest
├── README.md                     # Dokumentasi ini
├── rust-toolchain.toml           # Toolchain configuration
├── src/                          # Crate utama
│   ├── lib.rs                    # Library exports
│   ├── main.rs                   # Demo bot
│   ├── client.rs                 # Client utama (4.854 baris)
│   ├── bot.rs                    # Bot builder pattern
│   ├── send.rs                   # Kirim pesan & encryption
│   ├── message.rs                # Proses pesan incoming
│   ├── download.rs               # Download media
│   ├── upload.rs                 # Upload media
│   ├── pair.rs                   # QR pairing
│   ├── pair_code.rs              # Pair code authentication
│   ├── receipt.rs                # Receipt handling
│   ├── retry.rs                  # Retry logic
│   ├── prekeys.rs                # Prekey management
│   ├── handlers/                 # Stanza handlers
│   │   ├── router.rs             # Dispatch stanzas
│   │   ├── message.rs            # Handle <message>
│   │   ├── receipt.rs            # Handle <receipt>
│   │   ├── iq.rs                 # Handle <iq>
│   │   ├── notification.rs       # Handle <notification>
│   │   └── ...
│   ├── features/                 # High-level API
│   │   ├── groups.rs             # Grup operations
│   │   ├── contacts.rs           # Kontak & profile
│   │   ├── presence.rs           # Presence management
│   │   ├── chatstate.rs          # Typing indicators
│   │   ├── blocking.rs           # Blocklist
│   │   ├── profile.rs            # Profile settings
│   │   └── status.rs             # Status/stories
│   └── store/                    # Storage layer
│       ├── persistence_manager.rs
│       ├── signal_adapter.rs
│       └── ...
├── wacore/                       # Core protocol
│   ├── src/
│   ├── binary/                   # Binary protocol
│   ├── libsignal/                # Signal Protocol
│   ├── noise/                    # Noise handshake
│   ├── appstate/                 # AppState sync
│   └── derive/                   # Procedural macros
├── waproto/                      # Protocol Buffers
│   └── src/whatsapp.proto        # 5.547 baris definitions
├── storages/sqlite-storage/      # SQLite backend
├── transports/tokio-transport/   # WebSocket transport
├── http_clients/ureq-client/     # HTTP client
└── tests/e2e/                    # End-to-end tests
```

---

## ⚠️ Konvensi Penting

### 1. State Management

**JANGAN** modifikasi Device state langsung. Gunakan `DeviceCommand`:

```rust
// ❌ SALAH
device.push_name = "Nama Baru".to_string();

// ✅ BENAR
persistence_manager
    .process_command(DeviceCommand::SetPushName("Nama Baru".to_string()))
    .await;
```

### 2. IQ Requests

Gunakan pattern `client.execute()`:

```rust
// Pattern yang benar
let response = client.execute(Spec::new(&jid)).await?;

// IqSpec constructors pakai &Jid (bukan Jid) untuk hindari clone
```

### 3. Concurrency

- Semua I/O menggunakan Tokio
- Blocking I/O dibungkus `tokio::task::spawn_blocking`
- Gunakan `Client::chat_locks` untuk serialisasi operasi per-chat

### 4. Error Handling

```rust
// Gunakan thiserror untuk typed errors
// Gunakan anyhow untuk multi-failure functions
// JANGAN gunakan .unwrap() di luar tests
```

---

## 🧪 Testing

### Build & Verify

```bash
cargo fmt
cargo clippy --all-targets
cargo test --all
cargo test -p e2e-tests    # Requires mock server
```

### E2E Tests

Test E2E berjalan dengan mock server WhatsApp:

```bash
# Set mock server URL (optional)
export MOCK_SERVER_URL="wss://127.0.0.1:8080/ws/chat"

# Jalankan E2E tests
cargo test -p e2e-tests
```

Test files terbagi per domain untuk parallel execution:
- `connection.rs` - Connect/reconnect
- `messaging.rs` - Send/receive messages
- `groups.rs` - Group operations
- `media.rs` - Upload/download
- `presence.rs` - Typing indicators
- `profile.rs` - Profile settings
- `receipts.rs` - Receipt delivery
- Dan lainnya...

---

## 🔧 Custom Backend

Anda dapat implementasi storage, transport, atau HTTP client sendiri:

### Custom Storage

```rust
use ruwa::store::traits::Backend;

struct MyCustomStore {
    // Your implementation
}

#[async_trait::async_trait]
impl Backend for MyCustomStore {
    // Implement semua trait methods
}
```

### Custom Transport

```rust
use ruwa::transport::{Transport, TransportFactory};

struct MyTransportFactory;

impl TransportFactory for MyTransportFactory {
    fn create(&self) -> Pin<Box<dyn Future<Output = Result<Box<dyn Transport>>>>> {
        // Your transport creation logic
    }
}
```

Lihat implementasi default di `storages/sqlite-storage/`, `transports/tokio-transport/`, dan `http_clients/ureq-client/` sebagai referensi.

---

## 🔍 Troubleshooting

### Koneksi Gagal

1. **Cek internet**: Pastikan koneksi stabil
2. **Session expired**: Hapus `whatsapp.db` dan pairing ulang
3. **Firewall**: Pastikan WebSocket tidak diblokir

### Pesan Tidak Terkirim

1. **Session tidak ditemukan**: Tunggu retry receipt atau PDO
2. **Recipient offline**: Pesan akan dikirim saat online
3. **Invalid JID**: Pastikan format JID benar (`number@s.whatsapp.net`)

### Media Gagal Upload/Download

1. **File terlalu besar**: WhatsApp batasi ukuran media
2. **Network error**: Cek koneksi dan retry
3. **Media key invalid**: Re-upload media

### Pairing Gagal

1. **QR code expired**: QR valid 20-60 detik, akan rotate otomatis
2. **Pair code invalid**: Pastikan 8 digit benar (Crockford Base32)
3. **Session conflict**: Logout dari device lain jika perlu

---

## ⚠️ Disclaimer

> **PENTING**: Ini adalah client **TIDAK RESMI**. Penggunaan dapat melanggar Terms of Service Meta/WhatsApp dan berpotensi mengakibatkan **suspensi atau ban akun**.

Gunakan dengan risiko sendiri. Jangan gunakan untuk:
- Spam atau abuse
- Aktivitas ilegal
- Pelanggaran privasi orang lain
- Penggunaan komersial tanpa izin

Proyek ini untuk **tujuan edukasi dan riset** saja.

---

## 🙏 Acknowledgements

Proyek ini terinspirasi dan belajar dari:

- **[whatsmeow](https://github.com/tulir/whatsmeow)** - Implementasi WhatsApp Go yang luar biasa
- **[Baileys](https://github.com/WhiskeySockets/Baileys)** - Library WhatsApp TypeScript yang komprehensif
- **WhatsApp Web** - Sebagai referensi protokol dan behavior

---

## ❓ FAQ (Frequently Asked Questions)

### Q: Apakah library ini resmi dari WhatsApp?
**A:** Tidak, ini adalah **unofficial library** yang dibuat berdasarkan reverse engineering WhatsApp Web. Gunakan dengan risiko sendiri.

### Q: Apakah akun saya bisa di-banned?
**A:** Ada kemungkinan. WhatsApp dapat mendeteksi client tidak resmi dan melakukan suspend/ban akun. Gunakan hanya untuk testing atau development.

### Q: Apakah support multi-device?
**A:** Ya, support hingga 4 perangkat companion (seperti WhatsApp Web linked devices).

### Q: Kenapa build gagal dengan error SIMD?
**A:** Pastikan Anda menggunakan Rust stable dan feature SIMD sudah di-disable. Lihat bagian [Instalasi](#-instalasi) untuk detail.

### Q: Bagaimana cara report bug?
**A:** Silakan hubungi developer melalui kontak yang tersedia di atas atau buat issue di GitHub.

### Q: Apakah ada group support?
**A:** Ya, semua fitur grup tersedia: create, manage participants, settings, dll.

### Q: Bisa untuk bot WhatsApp?
**A:** Sangat bisa! Library ini dirancang untuk memudahkan pembuatan bot WhatsApp.

---

## 📚 Resources

- **Dokumentasi API**: [docs.rs/ruwa](https://docs.rs/ruwa)
- **GitHub**: [github.com/jrevanaldi-ai/ruwa](https://github.com/jrevanaldi-ai/ruwa)
- **Crates.io**: [crates.io/crates/ruwa](https://crates.io/crates/ruwa)

---

## 📄 License

MIT License - Lihat file [LICENSE](LICENSE) untuk detail.

**Copyright (c) 2026 Nathan**

---

## 🤝 Contributing

Kontribusi sangat diterima! Silakan:

1. Fork repository
2. Buat feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push ke branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

### Guidelines

- Ikuti code style dengan `cargo fmt`
- Pastikan `cargo clippy --all-targets` clean
- Tambahkan tests untuk fitur baru
- Update dokumentasi jika perlu

---

## 📊 Statistik Proyek

| Metrik | Nilai |
|--------|-------|
| Total baris kode | ~50.000+ |
| Crates dalam workspace | 14 |
| Tabel database | 16 |
| Test files E2E | 15 |
| Versi WhatsApp Web | 2.3000.x |
| Versi crate | 0.3.0 |

---

**Dibuat dengan ❤️ menggunakan Rust**
