# Contoh Penggunaan Fitur Ping

## Ping Sederhana

```rust
use ruwa::Client;
use anyhow::Result;

async fn check_latency(client: &Client) -> Result<()> {
    // Kirim ping ke server WhatsApp
    let ping = client.ping().await?;
    
    println!("Server RTT: {}ms", ping.rtt_ms);
    
    // Cek kualitas koneksi
    if ping.is_good() {
        println!("✓ Koneksi bagus (< 200ms)");
    } else if ping.is_moderate() {
        println!("⚠ Koneksi sedang (200-500ms)");
    } else {
        println!("✗ Koneksi buruk (> 500ms)");
    }
    
    Ok(())
}
```

## Ping dengan Timeout Custom

```rust
use std::time::Duration;
use ruwa::Client;

async fn quick_ping(client: &Client) {
    // Ping dengan timeout 2 detik untuk cek cepat
    match client.ping_with_timeout(Duration::from_secs(2)).await {
        Ok(ping) => println!("Fast ping: {}ms", ping.rtt_ms),
        Err(_) => println!("Tidak ada respons dalam 2 detik"),
    }
}
```

## Ping Berulang untuk Statistik

```rust
use ruwa::Client;

async fn ping_test(client: &Client) {
    // Kirim 5 ping dengan interval 1 detik
    let stats = client.ping_multiple(5, None).await.unwrap();
    
    println!("=== Statistik Ping ===");
    println!("Min:  {}ms", stats.min_rtt_ms);
    println!("Max:  {}ms", stats.max_rtt_ms);
    println!("Avg:  {:.2}ms", stats.avg_rtt_ms);
    println!("Jitter: {:.2}ms", stats.jitter_ms);
    println!("Kualitas: {}", stats.quality_rating());
}
```

## Bot dengan Fitur Ping

```rust
use ruwa::bot::{Bot, MessageContext};
use ruwa::Client;

#[tokio::main]
async fn main() {
    let mut bot = Bot::builder()
        .with_text_handler(|ctx| async move {
            if ctx.message_text().contains("!ping") {
                // Kirim ping ke server
                match ctx.client().ping().await {
                    Ok(ping) => {
                        let emoji = if ping.is_good() { "✓" } 
                                   else if ping.is_moderate() { "⚠" } 
                                   else { "✗" };
                        
                        ctx.reply(format!(
                            "{} Pong! Latency: {}ms\nKualitas: {}",
                            emoji,
                            ping.rtt_ms,
                            if ping.is_good() { "Bagus" }
                            else if ping.is_moderate() { "Sedang" }
                            else { "Buruk" }
                        )).await;
                    }
                    Err(e) => {
                        ctx.reply(format!("❌ Ping gagal: {}", e)).await;
                    }
                }
            }
        })
        .build();
    
    bot.start().await.unwrap();
}
```

## Monitoring Koneksi Berkala

```rust
use ruwa::Client;
use tokio::time::{interval, Duration};

async fn monitor_connection(client: Client) {
    let mut interval = interval(Duration::from_secs(60)); // Cek setiap 60 detik
    
    loop {
        interval.tick().await;
        
        match client.ping().await {
            Ok(ping) => {
                if ping.rtt_ms > 500 {
                    log::warn!("Latency tinggi: {}ms", ping.rtt_ms);
                } else {
                    log::info!("Latency normal: {}ms", ping.rtt_ms);
                }
            }
            Err(e) => {
                log::error!("Ping gagal: {}", e);
                // Mungkin perlu reconnect
                break;
            }
        }
    }
}
```

## Tips Optimasi Latency

1. **Gunakan timeout pendek** untuk deteksi cepat:
   ```rust
   client.ping_with_timeout(Duration::from_secs(3)).await
   ```

2. **Monitor jitter** untuk stabilitas:
   ```rust
   let stats = client.ping_multiple(10, None).await?;
   if stats.jitter_ms > 50.0 {
       // Koneksi tidak stabil
   }
   ```

3. **Check sebelum operasi penting**:
   ```rust
   if client.ping().await?.is_good() {
       // Lanjutkan kirim pesan penting
   }
   ```

4. **Avoid ping flood** - jangan ping terlalu sering:
   ```rust
   // ❌ Jangan lakukan ini
   loop {
       client.ping().await?;
       tokio::time::sleep(Duration::from_millis(100)).await;
   }
   
   // ✅ Lakukan ini
   let mut interval = tokio::time::interval(Duration::from_secs(30));
   loop {
       interval.tick().await;
       client.ping().await?;
   }
   ```

## Target Latency

| Kategori | RTT | Keterangan |
|----------|-----|------------|
| Excellent | < 100ms | Server lokal/regional |
| Good | 100-200ms | Server continental |
| Moderate | 200-500ms | Server intercontinental |
| Poor | > 500ms | Koneksi bermasalah |

Fitur ping ini dioptimalkan untuk mencapai **< 200ms** pada kondisi jaringan normal.
