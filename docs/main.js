import './style.css'

// Documentation content
const docs = {
  introduction: `
# 🦀 RuWa - WhatsApp Web API

**RuWa** adalah library WhatsApp Web API yang ditulis dalam Rust, dirancang untuk performa tinggi dan reliability production-grade.

<div class="feature-grid">
  <div class="feature-card">
    <h3>⚡ Performa Tinggi</h3>
    <p>Dibangun dengan Rust untuk kecepatan maksimal</p>
    <ul>
      <li>Memory usage < 50MB</li>
      <li>Startup time < 5s</li>
      <li>Message latency < 200ms</li>
    </ul>
  </div>
  <div class="feature-card">
    <h3>🔐 Secure by Default</h3>
    <p>End-to-end encryption dengan Signal Protocol</p>
    <ul>
      <li>AES-256-GCM encryption</li>
      <li>Curve25519 key agreement</li>
      <li>Double Ratchet algorithm</li>
    </ul>
  </div>
  <div class="feature-card">
    <h3>🛠️ Production Ready</h3>
    <p>Fitur lengkap untuk use case enterprise</p>
    <ul>
      <li>Auto reconnection</li>
      <li>Session backup/restore</li>
      <li>Rate limiting</li>
    </ul>
  </div>
</div>

## 📦 Installation

\`\`\`bash
# Add to your Cargo.toml
cargo add ruwa ruwa-sqlite-storage ruwa-tokio-transport ruwa-ureq-http-client
cargo add tokio --features full
\`\`\`

## 🚀 Quick Start

\`\`\`rust
use ruwa::bot::Bot;
use ruwa::store::SqliteStore;
use ruwa_tokio_transport::TokioWebSocketTransportFactory;
use ruwa_ureq_http_client::UreqHttpClient;
use std::sync::Arc;
use wacore_ng::types::events::Event;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Setup storage
    let store = SqliteStore::new("whatsapp.db").await?;

    // 2. Build bot
    let bot = Bot::builder()
        .with_backend(Arc::new(store))
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .on_event(|event, client| async move {
            match event {
                Event::Message(msg, info) => {
                    println!("Received: {:?}", msg);
                }
                Event::PairSuccess(_) => {
                    println!("✅ Pairing successful!");
                }
                _ => {}
            }
        })
        .build()
        .await?;

    // 3. Start bot
    bot.start().await?;
    Ok(())
}
\`\`\`

<div class="info-box warning">
  <strong>⚠️ Note:</strong> Pastikan Anda memiliki WhatsApp account yang aktif untuk pairing.
</div>
`,

  reconnection: `
# 🔄 Robust Reconnection System

Sistem reconnection yang production-ready dengan exponential backoff dan jitter.

## Features

- ✅ **Exponential Backoff**: Delay meningkat eksponensial (1s → 2s → 4s → ... → 5min)
- ✅ **Adaptive Jitter**: Random variance ±30% untuk prevent thundering herd
- ✅ **Max Retry Limit**: Configurable limit untuk avoid infinite loops
- ✅ **Connection State Machine**: 5-state lifecycle tracking
- ✅ **Real-time Metrics**: Success rate, uptime, failure tracking

## Usage

\`\`\`rust
use ruwa::Client;
use std::sync::Arc;

// Reconnect dengan backoff
match client.reconnect_with_backoff().await {
    Ok(()) => println!("✅ Reconnected!"),
    Err(e) => println!("❌ Max retries: {}", e),
}

// Get metrics
let metrics = client.get_reconnect_metrics();
println!("Success rate: {:.1}%", metrics.success_rate());
println!("Consecutive failures: {}", metrics.consecutive_failures);

// Check health
if client.is_connection_healthy() {
    println!("Connection is healthy");
}
\`\`\`

## Configuration

\`\`\`rust
use ruwa::reconnect::ReconnectConfig;
use std::time::Duration;

// Aggressive mode (critical apps)
let config = ReconnectConfig::aggressive();
// min_delay: 500ms, max_delay: 60s, max_retries: 20

// Conservative mode (battery devices)
let config = ReconnectConfig::conservative();
// min_delay: 5s, max_delay: 600s, max_retries: 5

// Custom config
let config = ReconnectConfig::new(
    Duration::from_secs(2),  // min_delay
    Duration::from_secs(120), // max_delay
    15,                       // max_retries
);
\`\`\`

## Backoff Progression

| Attempt | Delay | With Jitter (±30%) |
|---------|-------|-------------------|
| 1 | 1s | 0.7s - 1.3s |
| 2 | 2s | 1.4s - 2.6s |
| 3 | 4s | 2.8s - 5.2s |
| 4 | 8s | 5.6s - 10.4s |
| 5 | 16s | 11.2s - 20.8s |
| 6+ | 300s (cap) | 210s - 390s |

<div class="info-box info">
  <strong>💡 Tip:</strong> Gunakan mode aggressive untuk critical applications seperti payment notifications.
</div>
`,

  backup: `
# 💾 Session Backup & Restore

Backup dan restore session WhatsApp dengan enkripsi AES-256-GCM.

## Features

- ✅ **AES-256-GCM Encryption** dengan PBKDF2 key derivation
- ✅ **Gzip Compression** untuk hemat storage
- ✅ **Auto Backup Scheduler** dengan rotation
- ✅ **Backup Metadata** (timestamp, checksum, phone masked)
- ✅ **Builder Pattern API** untuk ease of use

## Manual Backup

\`\`\`rust
use ruwa::Client;
use std::sync::Arc;

// Backup dengan enkripsi
let result = client.backup_session()
    .to_file("session_backup.enc")
    .with_password("secure_password_123")
    .compress(true)
    .run()
    .await?;

println!("Backup completed!");
println!("Size: {} bytes", result.size_bytes);
println!("Encrypted: {}", result.encrypted);
\`\`\`

## Manual Restore

\`\`\`rust
// Restore dari backup
let result = client.restore_session()
    .from_file("session_backup.enc")
    .with_password("secure_password_123")
    .run()
    .await?;

println!("Restore completed!");
println!("Items restored: {}", result.items_restored);
\`\`\`

## Auto Backup

\`\`\`rust
use std::time::Duration;

// Enable auto backup setiap 1 jam
client.enable_auto_backup()
    .interval(Duration::from_secs(3600))
    .to_directory("./backups")
    .keep_last(10)  // Keep last 10 backups
    .with_password("secure_password")
    .start()
    .await?;

// Disable auto backup
client.disable_auto_backup().await;

// Get statistics
let stats = client.get_backup_stats().await;
println!("Total backups: {}", stats.total_backups);
println!("Last backup: {:?}", stats.last_backup);
\`\`\`

## Security

| Feature | Implementation |
|---------|---------------|
| Encryption | AES-256-GCM |
| Key Derivation | PBKDF2-HMAC-SHA256 |
| Iterations | 100,000 |
| Salt | 16 bytes random |
| Nonce | 12 bytes random |

<div class="info-box warning">
  <strong>⚠️ Important:</strong> Password TIDAK disimpan di backup. Lupa password = data hilang permanen!
</div>
`,

  ping: `
# 🏓 Ping Monitoring

Test latency koneksi ke server WhatsApp dengan akurasi tinggi.

## Features

- ✅ **Low-latency ping** (target < 200ms)
- ✅ **Custom timeout** untuk quick checks
- ✅ **Ping statistics** (min/max/avg/jitter)
- ✅ **Quality metrics** (good/moderate/poor)

## Basic Ping

\`\`\`rust
use ruwa::Client;

// Simple ping
let ping = client.ping().await?;
println!("RTT: {}ms", ping.rtt_ms);

// Check quality
if ping.is_good() {
    println!("✅ Connection is good (< 200ms)");
} else if ping.is_moderate() {
    println!("⚠️ Connection is moderate (200-500ms)");
} else {
    println!("❌ Connection is poor (> 500ms)");
}
\`\`\`

## Ping with Statistics

\`\`\`rust
use std::time::Duration;

// Multiple pings for statistics
let stats = client.ping_multiple(5, Some(Duration::from_secs(1))).await?;

println!("=== Ping Statistics ===");
println!("Min:  {}ms", stats.min_rtt_ms);
println!("Max:  {}ms", stats.max_rtt_ms);
println!("Avg:  {:.2}ms", stats.avg_rtt_ms);
println!("Jitter: {:.2}ms", stats.jitter_ms);
println!("Quality: {}", stats.quality_rating());
\`\`\`

## Custom Timeout

\`\`\`rust
use std::time::Duration;

// Quick ping with 2s timeout
match client.ping_with_timeout(Duration::from_secs(2)).await {
    Ok(ping) => println!("Fast ping: {}ms", ping.rtt_ms),
    Err(_) => println!("No response within 2 seconds"),
}
\`\`\`

## Quality Ratings

| Rating | RTT Range | Description |
|--------|-----------|-------------|
| Excellent | < 100ms | Local/regional server |
| Good | 100-200ms | Continental |
| Moderate | 200-500ms | Intercontinental |
| Poor | > 500ms | High latency |

<div class="info-box success">
  <strong>✅ Best Practice:</strong> Monitor ping secara berkala untuk detect network issues early.
</div>
`,

  messaging: `
# 💬 Messaging

Kirim dan terima pesan WhatsApp dengan berbagai tipe konten.

## Send Text Message

\`\`\`rust
use ruwa::types::jid::Jid;

// Send text message
client.send_text_message()
    .to(Jid::parse("6281234567890@s.whatsapp.net")?)
    .text("Hello, World! 👋")
    .await?;
\`\`\`

## Send Media

\`\`\`rust
// Send image
client.send_image_message()
    .to(jid)
    .image(image_bytes)
    .caption("Check this out!")
    .await?;

// Send document
client.send_document_message()
    .to(jid)
    .document(doc_bytes)
    .filename("report.pdf")
    .mime_type("application/pdf")
    .await?;

// Send video
client.send_video_message()
    .to(jid)
    .video(video_bytes)
    .caption("Funny video")
    .await?;
\`\`\`

## Receive Messages

\`\`\`rust
bot.on_event(|event, client| async move {
    if let Event::Message(msg, info) = event {
        match msg {
            Message { conversation: Some(text), .. } => {
                println!("Text: {}", text);
            }
            Message { image_message: Some(img), .. } => {
                println!("Image received");
                // Download media
                let media = client.download_media(&img).await?;
            }
            _ => {}
        }
    }
});
\`\`\`

## Message Features

| Feature | Status | Example |
|---------|--------|---------|
| Text messages | ✅ | \`conversation: Some("Hello")\` |
| Image messages | ✅ | \`image_message: Some(...)\` |
| Video messages | ✅ | \`video_message: Some(...)\` |
| Audio messages | ✅ | \`audio_message: Some(...)\` |
| Document messages | ✅ | \`document_message: Some(...)\` |
| Location messages | ✅ | \`location_message: Some(...)\` |
| Contact messages | ✅ | \`contact_message: Some(...)\` |
| Reply/Quote | ✅ | \`context_info: Some(...)\` |
| Reactions | ✅ | \`reaction_message: Some(...)\` |
| Edit messages | ✅ | \`edit_message(id, new_msg)\` |
| Delete messages | ✅ | \`revoke_message(id)\` |
`,

  groups: `
# 👥 Group Management

Manage WhatsApp groups dengan fitur lengkap.

## Create Group

\`\`\`rust
use ruwa::features::{GroupCreateOptions, Groups};

let result = client.groups().create()
    .subject("My New Group")
    .participants(vec![
        Jid::parse("6281234567890@s.whatsapp.net")?,
        Jid::parse("6281234567891@s.whatsapp.net")?,
    ])
    .description("Welcome to our group!")
    .await?;

println!("Group created: {:?}", result.jid);
\`\`\`

## Group Operations

\`\`\`rust
// List all groups
let groups = client.groups().list().await?;
for group in groups {
    println!("Group: {} ({})", group.subject, group.jid);
}

// Get metadata
let metadata = client.groups()
    .get_metadata(group_jid)
    .await?;
println!("Participants: {}", metadata.participants.len());

// Add participants
client.groups()
    .add_participants(group_jid, new_participants)
    .await?;

// Remove participants
client.groups()
    .remove_participants(group_jid, participants_to_remove)
    .await?;

// Promote to admin
client.groups()
    .promote_participants(group_jid, admins)
    .await?;

// Update subject
client.groups()
    .set_subject(group_jid, "New Group Name")
    .await?;

// Update description
client.groups()
    .set_description(group_jid, "New description")
    .await?;

// Get invite link
let link = client.groups()
    .get_invite_link(group_jid)
    .await?;
\`\`\`

## Group Features

| Feature | Status | Method |
|---------|--------|--------|
| Create group | ✅ | \`groups().create()\` |
| List groups | ✅ | \`groups().list()\` |
| Get metadata | ✅ | \`groups().get_metadata()\` |
| Add participants | ✅ | \`groups().add_participants()\` |
| Remove participants | ✅ | \`groups().remove_participants()\` |
| Promote to admin | ✅ | \`groups().promote_participants()\` |
| Demote admin | ✅ | \`groups().demote_participants()\` |
| Update subject | ✅ | \`groups().set_subject()\` |
| Update description | ✅ | \`groups().set_description()\` |
| Invite link | ✅ | \`groups().get_invite_link()\` |
| Leave group | ✅ | \`groups().leave()\` |
`,

  api: `
# 📚 API Reference

## Client Methods

### Connection

\`\`\`rust
// Connect
client.connect().await?;

// Disconnect
client.disconnect().await;

// Reconnect with backoff
client.reconnect_with_backoff().await?;

// Check connection
if client.is_connected() {
    println!("Connected!");
}
\`\`\`

### Messaging

\`\`\`rust
// Send text
client.send_text_message()
    .to(jid)
    .text("Hello")
    .await?;

// Send image
client.send_image_message()
    .to(jid)
    .image(bytes)
    .caption("Image")
    .await?;

// Download media
let media = client.download_media(&message).await?;
\`\`\`

### Groups

\`\`\`rust
// Create
client.groups().create()
    .subject("Group")
    .participants(vec![jid1, jid2])
    .await?;

// List
let groups = client.groups().list().await?;

// Get metadata
let metadata = client.groups()
    .get_metadata(group_jid)
    .await?;
\`\`\`

### Session Management

\`\`\`rust
// Backup
client.backup_session()
    .to_file("backup.enc")
    .with_password("secret")
    .run()
    .await?;

// Restore
client.restore_session()
    .from_file("backup.enc")
    .with_password("secret")
    .run()
    .await?;

// Auto backup
client.enable_auto_backup()
    .interval(Duration::from_secs(3600))
    .to_directory("./backups")
    .keep_last(10)
    .start()
    .await?;
\`\`\`

### Monitoring

\`\`\`rust
// Ping
let ping = client.ping().await?;
println!("RTT: {}ms", ping.rtt_ms);

// Statistics
let stats = client.ping_multiple(5, None).await?;
println!("Avg: {:.2}ms", stats.avg_rtt_ms);

// Backup stats
let backup_stats = client.get_backup_stats().await;
println!("Total: {}", backup_stats.total_backups);
\`\`\`

## Bot Builder

\`\`\`rust
use ruwa::bot::Bot;
use std::sync::Arc;

let bot = Bot::builder()
    .with_backend(Arc::new(store))
    .with_transport_factory(transport)
    .with_http_client(http)
    .on_event(|event, client| async move {
        match event {
            Event::Message(msg, info) => {
                // Handle message
            }
            Event::PairSuccess(_) => {
                // Handle pairing success
            }
            _ => {}
        }
    })
    .build()
    .await?;

bot.start().await?;
\`\`\`

## Event Types

\`\`\`rust
pub enum Event {
    Connected(Connected),
    PairingQrCode { code: String, timeout: Duration },
    PairingCode { code: String, timeout: Duration },
    PairSuccess(PairSuccess),
    Message(Box<Message>, MessageInfo),
    Receipt(Receipt),
    Presence(Presence),
    ChatState(ChatStateEvent),
    LoggedOut(LoggedOut),
    // ... more events
}
\`\`\`
`
};

// Render documentation
function renderDoc(section) {
  const content = document.getElementById('content');
  content.innerHTML = marked.parse(docs[section] || docs.introduction);
  
  // Update active nav
  document.querySelectorAll('.nav-group a').forEach(link => {
    link.classList.remove('active');
    if (link.getAttribute('href') === \`#\${section}\`) {
      link.classList.add('active');
    }
  });
}

// Simple markdown parser
function marked(text) {
  return text
    .replace(/^# (.*$)/gim, '<h1>$1</h1>')
    .replace(/^## (.*$)/gim, '<h2>$1</h2>')
    .replace(/^### (.*$)/gim, '<h3>$1</h3>')
    .replace(/\*\*(.*)\*\*/gim, '<strong>$1</strong>')
    .replace(/\*(.*)\*/gim, '<em>$1</em>')
    .replace(/\n/gim, '<br>')
    .replace(/\`\`\`rust/gim, '<pre><code class="language-rust">')
    .replace(/\`\`\`/gim, '</code></pre>')
    .replace(/\| (.*) \|/gim, '<tr><td>$1</td></tr>')
    .replace(/<tr><td>(.*)<\/td><\/tr><tr><td>(.*)<\/td><\/tr>/gim, '<tr><td>$1</td><td>$2</td></tr>')
    .replace(/<table>/gim, '<table><tr>')
    .replace(/<\/table>/gim, '</tr></table>');
}

// Mobile menu toggle
document.addEventListener('DOMContentLoaded', () => {
  const menuToggle = document.getElementById('menuToggle');
  const sidebar = document.querySelector('.sidebar');
  
  menuToggle?.addEventListener('click', () => {
    sidebar.classList.toggle('open');
  });
  
  // Initial render
  renderDoc('introduction');
  
  // Search functionality
  const searchInput = document.getElementById('searchInput');
  searchInput?.addEventListener('input', (e) => {
    const query = e.target.value.toLowerCase();
    // Simple search implementation
    console.log('Searching for:', query);
  });
});

// Handle hash changes
window.addEventListener('hashchange', () => {
  const hash = window.location.hash.slice(1);
  if (hash && docs[hash]) {
    renderDoc(hash);
  }
});
