# Session Backup & Restore - Dokumentasi Lengkap

## 🎯 Overview

Sistem **Session Backup & Restore** memungkinkan Anda untuk:
- ✅ Backup session WhatsApp terenkripsi
- ✅ Restore session ke server/device lain
- ✅ Auto backup berkala dengan rotation
- ✅ AES-256-GCM encryption dengan password
- ✅ Compression untuk hemat storage

---

## 🚀 Quick Start

### Backup Manual

```rust
use ruwa::Client;
use std::sync::Arc;

let client: Arc<Client> = /* ... */;

// Backup dengan enkripsi
let result = client.backup_session()
    .to_file("session_backup.enc")
    .with_password("secure_password_123")
    .run()
    .await?;

println!("Backup completed: {:?}", result.file_path);
println!("Size: {} bytes", result.size_bytes);
println!("Encrypted: {}", result.encrypted);
```

### Restore Manual

```rust
// Restore dari backup
let result = client.restore_session()
    .from_file("session_backup.enc")
    .with_password("secure_password_123")
    .run()
    .await?;

println!("Restore completed!");
println!("Items restored: {}", result.items_restored);
println!("Warnings: {}", result.warnings.len());
```

### Auto Backup (Scheduled)

```rust
use std::time::Duration;

// Enable auto backup setiap 1 jam, simpan 10 backup terakhir
client.enable_auto_backup()
    .interval(Duration::from_secs(3600))  // 1 jam
    .to_directory("./backups")
    .keep_last(10)
    .with_password("secure_password")
    .start()
    .await?;

// Disable auto backup
client.disable_auto_backup().await;
```

---

## 📖 API Reference

### BackupBuilder

```rust
client.backup_session()
    .to_file("path/to/backup.enc")      // Required: destination file
    .with_password("secret")            // Optional: encryption password
    .with_tags(vec!["daily".to_string()]) // Optional: metadata tags
    .compress(true)                     // Optional: enable compression (default: true)
    .run()
    .await?
```

### RestoreBuilder

```rust
client.restore_session()
    .from_file("path/to/backup.enc")    // Required: source file
    .with_password("secret")            // Optional: decryption password
    .skip_version_check(false)          // Optional: skip version validation
    .run()
    .await?
```

### AutoBackupBuilder

```rust
client.enable_auto_backup()
    .interval(Duration::from_secs(3600)) // Required: backup interval
    .to_directory("./backups")           // Required: backup directory
    .keep_last(10)                       // Optional: retain last N backups (default: 10)
    .with_prefix("session")              // Optional: filename prefix
    .with_password("secret")             // Optional: encryption password
    .no_compression()                    // Optional: disable compression
    .start()
    .await?
```

---

## 🔐 Security Features

### Encryption

- **Algorithm**: AES-256-GCM
- **Key Derivation**: PBKDF2-HMAC-SHA256
- **Iterations**: 100,000
- **Salt**: 16 bytes random per backup
- **Nonce**: 12 bytes random per encryption

### File Format

```
[16 bytes salt][12 bytes nonce][ciphertext][16 bytes auth tag]
```

### Best Practices

```rust
// ✅ GOOD: Strong password
client.backup_session()
    .with_password("X9#mK2$pL5@nQ8*v")

// ❌ BAD: Weak password
client.backup_session()
    .with_password("123456")

// ✅ GOOD: Store password in environment variable
let password = std::env::var("BACKUP_PASSWORD")?;
client.backup_session()
    .with_password(password)

// ✅ GOOD: Use different passwords per environment
#[cfg(debug_assertions)]
let password = "dev_backup_password";
#[cfg(not(debug_assertions))]
let password = std::env::var("PROD_BACKUP_PASSWORD")?;
```

---

## 📊 Backup Metadata

Setiap backup menyimpan metadata:

```rust
let result = client.backup_session().run().await?;

println!("Created: {}", result.metadata.created_at);
println!("RuWa Version: {}", result.metadata.ruwa_version);
println!("Phone (masked): {}", result.metadata.phone_masked);
println!("Size: {} bytes", result.metadata.size_bytes);
println!("Checksum: {}", result.metadata.checksum);
println!("Tags: {:?}", result.metadata.tags);
```

---

## 🔄 Auto Backup Rotation

Auto backup otomatis cleanup backup lama:

```rust
// Config: keep last 10 backups
client.enable_auto_backup()
    .keep_last(10)
    .start()
    .await?;

// Backup files:
// session_20260329_100000.enc  (deleted when 11th backup created)
// session_20260329_110000.enc
// session_20260329_120000.enc
// ...
// session_20260329_190000.enc  (newest)
```

---

## 📈 Statistics & Monitoring

```rust
// Get backup statistics
let stats = client.get_backup_stats().await;

println!("Total backups: {}", stats.total_backups);
println!("Last backup: {:?}", stats.last_backup);
println!("Auto-backup enabled: {}", stats.auto_backup_enabled);

// Check uptime since last backup
if let Some(uptime) = stats.last_backup.and_then(|last| {
    Some(chrono::Utc::now().signed_duration_since(last))
}) {
    println!("Time since last backup: {:?}", uptime);
}
```

---

## 🛠️ Troubleshooting

### Backup Gagal

**Problem**: `Permission denied`

**Solution**:
```rust
// Pastikan directory ada dan writable
std::fs::create_dir_all("./backups")?;

// Atau gunakan absolute path
client.backup_session()
    .to_file("/home/user/backups/session.enc")
```

### Restore Gagal

**Problem**: `Invalid password`

**Solution**:
```rust
// Pastikan password sama dengan saat backup
// Password case-sensitive!

// Test decrypt dulu
match client.restore_session()
    .from_file("backup.enc")
    .with_password(password)
    .run()
    .await
{
    Ok(_) => println!("Password correct"),
    Err(e) => eprintln!("Wrong password: {}", e),
}
```

### Auto Backup Tidak Jalan

**Problem**: Auto backup tidak ter-trigger

**Solution**:
```rust
// Check status
let stats = client.get_backup_stats().await;
if !stats.auto_backup_enabled {
    println!("Auto-backup not enabled!");
}

// Re-enable
client.enable_auto_backup()
    .interval(Duration::from_secs(3600))
    .to_directory("./backups")
    .start()
    .await?;
```

---

## 💾 Storage Requirements

### Backup Size

| Session Type | Uncompressed | Compressed |
|--------------|--------------|------------|
| New session  | ~500 bytes   | ~300 bytes |
| Active (1 day) | ~2 KB     | ~1 KB      |
| Active (1 week) | ~10 KB   | ~5 KB      |
| Active (1 month) | ~50 KB  | ~25 KB     |

### Storage Calculation

Dengan auto backup setiap jam, keep last 10:
```
10 backups × 5 KB = 50 KB total storage
```

---

## 🎯 Use Cases

### 1. Disaster Recovery

```rust
// Backup sebelum update
client.backup_session()
    .to_file("pre_update_backup.enc")
    .with_password(env::var("BACKUP_PASSWORD")?)
    .run()
    .await?;

// Do update...
// If fails, restore:
client.restore_session()
    .from_file("pre_update_backup.enc")
    .with_password(env::var("BACKUP_PASSWORD")?)
    .run()
    .await?;
```

### 2. Migration ke Server Baru

```rust
// Server lama: backup
let backup = client.backup_session()
    .to_file("/tmp/session_backup.enc")
    .with_password("migration_password")
    .run()
    .await?;

// Copy file ke server baru (scp, rsync, etc)
// std::process::Command::new("scp")...

// Server baru: restore
new_client.restore_session()
    .from_file("/tmp/session_backup.enc")
    .with_password("migration_password")
    .run()
    .await?;
```

### 3. Multi-Environment Sync

```rust
// Backup production
prod_client.backup_session()
    .to_file("./backups/prod_backup.enc")
    .run()
    .await?;

// Restore ke staging untuk testing
staging_client.restore_session()
    .from_file("./backups/prod_backup.enc")
    .run()
    .await?;
```

---

## ⚠️ Important Notes

1. **Password Management**:
   - ⚠️ Password TIDAK disimpan di backup
   - ⚠️ Lupa password = data hilang permanen
   - ✅ Gunakan password manager

2. **Backup Frequency**:
   - ✅ Auto backup setiap 1-6 jam recommended
   - ⚠️ Terlalu sering = boros storage
   - ⚠️ Terlalu jarang = risk data loss

3. **Security**:
   - ✅ Selalu gunakan encryption untuk production
   - ✅ Simpan backup di secure location
   - ✅ Rotate backup passwords periodically

4. **Testing**:
   - ✅ Test restore procedure regularly
   - ✅ Verify backup integrity
   - ✅ Document restore process

---

## 📝 Examples

### Complete Example with Error Handling

```rust
use ruwa::Client;
use std::{sync::Arc, time::Duration, env};
use anyhow::Result;

async fn setup_backup_system(client: Arc<Client>) -> Result<()> {
    // Get password from environment
    let password = env::var("SESSION_BACKUP_PASSWORD")
        .unwrap_or_else(|_| "default_dev_password".to_string());

    // Enable auto backup
    client.enable_auto_backup()
        .interval(Duration::from_secs(3600))  // Every hour
        .to_directory("./backups")
        .keep_last(24)  // Keep 1 day of backups
        .with_password(&password)
        .with_prefix("whatsapp_session")
        .start()
        .await?;

    println!("Auto-backup enabled successfully!");

    // Do initial backup
    let result = client.backup_session()
        .to_file("./backups/initial_backup.enc")
        .with_password(&password)
        .with_tags(vec!["initial".to_string()])
        .run()
        .await?;

    println!("Initial backup completed: {} bytes", result.size_bytes);

    Ok(())
}

async fn restore_from_latest_backup(client: Arc<Client>) -> Result<()> {
    // Find latest backup file
    let mut entries: Vec<_> = std::fs::read_dir("./backups")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "enc"))
        .collect();

    entries.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH));

    let latest = entries.last().ok_or_else(|| anyhow::anyhow!("No backups found"))?;

    println!("Restoring from: {:?}", latest.path());

    let password = env::var("SESSION_BACKUP_PASSWORD")
        .unwrap_or_else(|_| "default_dev_password".to_string());

    client.restore_session()
        .from_file(latest.path())
        .with_password(&password)
        .run()
        .await?;

    println!("Restore completed successfully!");

    Ok(())
}
```

---

## 📚 Related Documentation

- [Reconnection System](RECONNECTION.md) - Robust WebSocket reconnection
- [Ping Feature](ping_example.md) - Connection latency testing
- [README](../README.md) - Complete RuWa documentation

---

**Implemented**: March 29, 2026  
**Version**: 1.1.0  
**Status**: ✅ Production Ready  
**Tests**: 4 unit tests included
