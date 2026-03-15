# RuWa

> **WhatsApp Web API dalam Rust** - High-performance, async library untuk WhatsApp Web

[![Crates.io](https://img.shields.io/crates/v/ruwa.svg?style=for-the-badge)](https://crates.io/crates/ruwa)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Rust CI](https://img.shields.io/github/actions/workflow/status/jrevanaldi-ai/ruwa/main.yml?style=for-the-badge)](https://github.com/jrevanaldi-ai/ruwa/actions)

---

<div align="center">

## 📬 Kontak Developer

[![WhatsApp](https://img.shields.io/badge/WhatsApp-25D366?style=for-the-badge&logo=whatsapp&logoColor=white)](https://wa.me/62895416602000)
[![Instagram](https://img.shields.io/badge/Instagram-E4405F?style=for-the-badge&logo=instagram&logoColor=white)](https://instagram.com/chrisjoo_uww)
[![Telegram](https://img.shields.io/badge/Telegram-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white)](https://t.me/AstraluneSuport)
[![Email](https://img.shields.io/badge/Email-D14836?style=for-the-badge&logo=gmail&logoColor=white)](mailto:support@nathan.christmas)

> 🤖 **Note**: Dokumentasi dibuat dengan AI. Ada kesalahan? Hubungi developer!

</div>

---

## 📑 Daftar Isi

<details>
<summary><b>Klik untuk expand</b></summary>

1. [✨ Fitur](#-fitur)
2. [🚀 Quick Start](#-quick-start)
3. [📦 Instalasi](#-instalasi)
4. [🏗️ Arsitektur](#-arsitektur)
5. [📖 Panduan](#-panduan)
   - [Autentikasi](#autentikasi)
   - [Kirim Pesan](#kirim-pesan)
   - [Media](#media)
   - [Grup](#grup)
6. [❓ FAQ](#-faq)
7. [⚠️ Disclaimer](#-disclaimer)

</details>

---

## ✨ Fitur

<table>
<tr>
<td width="50%">

### 🔐 Autentikasi
- QR code pairing
- Pair code (8 digit)
- Auto-reconnect
- Multi-device (4 devices)

### 💬 Messaging
- E2E encryption
- 1-on-1 & group chats
- Edit, react, quote
- Delete for everyone

### 📸 Media
- Upload/download
- Image, video, audio, doc
- Auto encryption

</td>
<td width="50%">

### 👥 Grup
- Create & manage
- Participant management
- Group settings
- Invite links

### 📡 Presence
- Online/offline status
- Typing indicators
- Read receipts

### 🔄 Sync
- AppState sync
- History sync
- Offline messages

</td>
</tr>
</table>

---

## 🚀 Quick Start

### Contoh Minimalis

```rust
use ruwa::bot::Bot;
use ruwa::store::SqliteStore;
use wacore::types::events::Event;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = SqliteStore::new("whatsapp.db").await?;
    
    Bot::builder()
        .with_backend(backend)
        .on_event(|event, _client| async move {
            if let Event::Message(msg, info) = event {
                println!("Pesan dari {}: {:?}", info.source.sender, msg);
            }
        })
        .build()
        .await?
        .run()
        .await?
        .await?;
    
    Ok(())
}
```

### Jalankan Demo

```bash
cargo run                          # QR code
cargo run -- -p 6281234567890     # Pair code + QR
```

---

## 📦 Instalasi

### 1. Tambahkan ke Cargo.toml

```toml
[dependencies]
ruwa = "1.0.0"
ruwa-sqlite-storage = "1.0.0"
ruwa-tokio-transport = "1.0.0"
ruwa-ureq-http-client = "1.0.0"
tokio = { version = "1.48", features = ["full"] }
```

### 2. Requirements

| Requirement | Versi | Keterangan |
|------------|-------|------------|
| **Rust** | 1.70+ | Stable channel |
| **protoc** | 3.25.3+ | Protocol Buffers |
| **SQLite** | Latest | Storage backend |

### 3. Install Rust

```bash
# Linux/Mac
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Termux/Android
pkg install rust
```

---

## 🏗️ Arsitektur

```
┌─────────────────────────────────────────┐
│         Bot API / Client                │
│  - Event handlers                       │
│  - High-level features                  │
├─────────────────────────────────────────┤
│         Stanza Handlers                 │
│  - Message, Receipt, IQ, Notification   │
├─────────────────────────────────────────┤
│         Core Protocol (wacore)          │
│  - Signal Protocol encryption           │
│  - Noise Protocol handshake             │
│  - AppState synchronization             │
├─────────────────────────────────────────┤
│         Binary Protocol                 │
│  - Marshal/Unmarshal nodes              │
├─────────────────────────────────────────┤
│         Transport & Storage             │
│  - WebSocket, HTTP, SQLite              │
└─────────────────────────────────────────┘
```

### Workspace Crates

| Crate | Deskripsi |
|-------|-----------|
| `ruwa` | Crate utama |
| `wacore` | Core protocol |
| `waproto` | Protobuf definitions |
| `ruwa-sqlite-storage` | SQLite backend |
| `ruwa-tokio-transport` | WebSocket transport |
| `ruwa-ureq-http-client` | HTTP client |

---

## 📖 Panduan

### Autentikasi

#### QR Code
```rust
.on_event(|event, _client| async move {
    if let Event::PairingQrCode { code, timeout } = event {
        println!("Scan QR ({} detik):\n{}", timeout.as_secs(), code);
    }
})
```

#### Pair Code
```rust
use ruwa::bot::{Bot, PairCodeOptions};

Bot::builder()
    .with_pair_code(PairCodeOptions {
        phone_number: "6281234567890".to_string(),
        ..Default::default()
    })
```

---

### Kirim Pesan

#### Teks
```rust
use ruwa::Jid;
use waproto::whatsapp as wa;

let to: Jid = "6281234567890@s.whatsapp.net".parse()?;
client.send_message(to, wa::Message {
    conversation: Some("Halo!".to_string()),
    ..Default::default()
}).await?;
```

#### Quote/Reply
```rust
use wacore::proto_helpers::build_quote_context;

let context = build_quote_context(&msg_id, &sender, &chat, &quoted_msg);
client.send_message(chat, wa::Message {
    extended_text_message: Some(Box::new(
        wa::message::ExtendedTextMessage {
            text: Some("Reply".to_string()),
            context_info: Some(Box::new(context)),
            ..Default::default()
        }
    )),
    ..Default::default()
}).await?;
```

#### Reaction
```rust
use chrono::Utc;

client.send_message(chat, wa::Message {
    reaction_message: Some(wa::message::ReactionMessage {
        key: Some(message_key),
        text: Some("👍".to_string()),
        sender_timestamp_ms: Some(Utc::now().timestamp_millis()),
        ..Default::default()
    }),
    ..Default::default()
}).await?;
```

---

### Media

#### Upload & Kirim Gambar
```rust
use ruwa::upload::UploadResponse;
use ruwa::download::MediaType;

// Upload
let data = std::fs::read("foto.jpg")?;
let upload = client.upload(data, MediaType::Image).await?;

// Kirim
client.send_message(chat, wa::Message {
    image_message: Some(Box::new(wa::message::ImageMessage {
        url: Some(upload.url),
        direct_path: Some(upload.direct_path),
        media_key: Some(upload.media_key),
        mimetype: Some("image/jpeg".to_string()),
        caption: Some("Caption".to_string()),
        ..Default::default()
    })),
    ..Default::default()
}).await?;
```

---

### Grup

```rust
use ruwa::features::{Groups, GroupCreateOptions};

// Buat grup
let result = client.groups().create(
    "Nama Grup",
    vec!["6281234567890@s.whatsapp.net".parse()?],
    GroupCreateOptions::default(),
).await?;

// Manage grup
let groups = client.groups();
groups.add_participants(jid, vec![new_jid]).await?;
groups.remove_participants(jid, vec![old_jid]).await?;
groups.promote_participants(jid, vec![admin_jid]).await?;
groups.set_subject(jid, "Nama Baru").await?;
groups.set_description(jid, "Deskripsi").await?;
groups.leave(jid).await?;
```

---

## ❓ FAQ

<details>
<summary><b>Apakah ini resmi dari WhatsApp?</b></summary>

❌ **Tidak**. Ini adalah unofficial library hasil reverse engineering WhatsApp Web. Gunakan dengan risiko sendiri.
</details>

<details>
<summary><b>Akun bisa di-banned?</b></summary>

⚠️ **Ada kemungkinan**. WhatsApp dapat detect client tidak resmi. Hanya gunakan untuk testing/development.
</details>

<details>
<summary><b>Support multi-device?</b></summary>

✅ **Ya**. Support hingga 4 perangkat companion seperti WhatsApp Web linked devices.
</details>

<details>
<summary><b>Bisa untuk bot WhatsApp?</b></summary>

✅ **Sangat bisa**! Library ini dirancang khusus untuk memudahkan pembuatan bot.
</details>

<details>
<summary><b>Build error SIMD?</b></summary>

🔧 Pastikan menggunakan **Rust stable** dan feature SIMD sudah di-disable di Cargo.toml.
</details>

---

## ⚠️ Disclaimer

> **PENTING**: Ini adalah client **TIDAK RESMI**. Penggunaan dapat melanggar Terms of Service Meta/WhatsApp dan berpotensi mengakibatkan **suspend/ban akun**.

**Gunakan hanya untuk:**
- ✅ Testing & development
- ✅ Educational purposes
- ✅ Research

**JANGAN gunakan untuk:**
- ❌ Spam atau abuse
- ❌ Aktivitas ilegal
- ❌ Pelanggaran privasi
- ❌ Commercial use tanpa izin

---

## 📚 Resources

| Link | Deskripsi |
|------|-----------|
| [📖 API Docs](https://docs.rs/ruwa) | Dokumentasi API lengkap |
| [💻 GitHub](https://github.com/jrevanaldi-ai/ruwa) | Source code & issues |
| [📦 Crates.io](https://crates.io/crates/ruwa) | Package registry |

---

## 🙏 Acknowledgements

Terima kasih kepada:

- [**whatsmeow**](https://github.com/tulir/whatsmeow) - Go WhatsApp library
- [**Baileys**](https://github.com/WhiskeySockets/Baileys) - TypeScript WhatsApp library
- **WhatsApp Web** - Referensi protokol

---

<div align="center">

**Copyright © 2026 Nathan**

[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)

</div>
