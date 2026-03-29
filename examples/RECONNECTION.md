# Robust WebSocket Reconnection

## 🎯 Overview

RuWa sekarang dilengkapi dengan **reconnection logic yang robust** menggunakan exponential backoff dengan jitter, dirancang untuk production environment dengan:

- ✅ **Exponential Backoff**: Delay meningkat eksponensial setiap gagal
- ✅ **Jitter**: Random variance untuk mencegah thundering herd
- ✅ **Max Retry Limit**: Mencegah infinite loop
- ✅ **Connection State Machine**: Track lifecycle koneksi
- ✅ **Metrics & Monitoring**: Statistik lengkap untuk observability

---

## 📊 Connection States

```
Disconnected → Connecting → Connected
                  ↓
            Reconnecting → Failed (max retries exceeded)
```

| State | Description |
|-------|-------------|
| **Disconnected** | Initial state, never connected |
| **Connecting** | Currently attempting to connect |
| **Connected** | Successfully connected and authenticated |
| **Reconnecting** | Connection lost, will retry |
| **Failed** | Max retries exceeded, manual intervention required |

---

## 🔧 Configuration

### Default Configuration

```rust
use ruwa::reconnect::ReconnectConfig;

let config = ReconnectConfig::default();
// min_delay: 1 second
// max_delay: 5 minutes (300 seconds)
// backoff_base: 2.0 (doubling)
// max_retries: 10
// enable_jitter: true
// jitter_factor: 0.3 (±30%)
```

### Pre-configured Profiles

```rust
// Aggressive mode - for critical applications
let config = ReconnectConfig::aggressive();
// min_delay: 500ms
// max_delay: 60s
// max_retries: 20
// jitter_factor: 0.2 (±20%)

// Conservative mode - for battery-powered devices
let config = ReconnectConfig::conservative();
// min_delay: 5s
// max_delay: 600s (10 minutes)
// max_retries: 5
// jitter_factor: 0.4 (±40%)
```

### Custom Configuration

```rust
use ruwa::reconnect::ReconnectConfig;
use std::time::Duration;

let config = ReconnectConfig::new(
    Duration::from_secs(2),  // min_delay
    Duration::from_secs(120), // max_delay
    15,                       // max_retries
);
```

---

## 📖 Usage Examples

### Basic Reconnection

```rust
use ruwa::Client;
use std::sync::Arc;

async fn handle_disconnect(client: Arc<Client>) {
    // Attempt to reconnect with backoff
    match client.reconnect_with_backoff().await {
        Ok(()) => println!("✅ Reconnected successfully"),
        Err(e) => println!("❌ Reconnection failed: {}", e),
    }
}
```

### Check Connection Status

```rust
// Get current state
let state = client.get_connection_state();
println!("Connection state: {}", state);

// Get detailed metrics
let metrics = client.get_reconnect_metrics();
println!("Total attempts: {}", metrics.total_attempts);
println!("Success rate: {:.1}%", metrics.success_rate());
println!("Consecutive failures: {}", metrics.consecutive_failures);

// Get human-readable status
let status = client.get_reconnect_status();
println!("{}", status);
```

### Monitor Connection Health

```rust
use tokio::time::{interval, Duration};

async fn monitor_connection(client: Arc<Client>) {
    let mut interval = interval(Duration::from_secs(60));
    
    loop {
        interval.tick().await;
        
        if !client.is_connection_healthy() {
            log::warn!("⚠️ Connection health degraded");
            
            let metrics = client.get_reconnect_metrics();
            if metrics.consecutive_failures > 3 {
                log::error!("Too many consecutive failures, alerting admin...");
                // Send alert, trigger fallback, etc.
            }
        }
    }
}
```

### Automatic Reconnection on Disconnect

```rust
use ruwa::bot::Bot;
use ruwa::store::SqliteStore;
use std::sync::Arc;

async fn setup_bot_with_reconnect() -> anyhow::Result<()> {
    let store = SqliteStore::new("bot.db").await?;
    
    let bot = Bot::builder()
        .with_backend(Arc::new(store))
        // ... other config
        .on_event(|event, client| async move {
            use ruwa::types::events::Event;
            
            match event {
                Event::Disconnected(_) => {
                    log::warn!("Disconnected, attempting to reconnect...");
                    
                    // Try reconnect with backoff
                    if let Err(e) = client.reconnect_with_backoff().await {
                        log::error!("Reconnection failed: {}", e);
                        // Alert admin, shutdown, etc.
                    }
                }
                
                Event::Connected(_) => {
                    log::info!("✅ Connected successfully");
                    
                    // Log connection metrics
                    let metrics = client.get_reconnect_metrics();
                    log::info!("Connection uptime: {:?}", metrics.uptime());
                }
                
                _ => {}
            }
        })
        .build()
        .await?;
    
    bot.start().await?;
    Ok(())
}
```

---

## 📈 Metrics Explained

### ReconnectMetrics Fields

| Field | Type | Description |
|-------|------|-------------|
| `total_attempts` | u64 | Total connection attempts |
| `successful_connections` | u64 | Successful connections |
| `failed_attempts` | u64 | Failed attempts |
| `consecutive_failures` | u32 | Current streak of failures |
| `max_consecutive_failures` | u32 | Worst failure streak observed |
| `total_reconnect_time_secs` | u64 | Total time spent reconnecting |
| `last_successful_connection` | Option<u64> | Unix timestamp of last success |
| `last_attempt` | Option<u64> | Unix timestamp of last attempt |

### Calculated Metrics

```rust
let metrics = client.get_reconnect_metrics();

// Success rate percentage
let rate = metrics.success_rate(); // e.g., 85.5%

// Uptime since last successful connection
if let Some(uptime) = metrics.uptime() {
    println!("Uptime: {} hours", uptime.as_secs() / 3600);
}

// Health check
if metrics.consecutive_failures > 5 {
    // Connection is unhealthy
}
```

---

## ⚙️ Backoff Algorithm

### Exponential Backoff Formula

```
delay = min(min_delay * (backoff_base ^ attempt), max_delay)
```

### Example Progression (Default Config)

| Attempt | Calculation | Delay |
|---------|-------------|-------|
| 1 | 1s × 2⁰ | 1s |
| 2 | 1s × 2¹ | 2s |
| 3 | 1s × 2² | 4s |
| 4 | 1s × 2³ | 8s |
| 5 | 1s × 2⁴ | 16s |
| 6 | 1s × 2⁵ | 32s |
| 7 | 1s × 2⁶ | 64s |
| 8+ | Capped | 300s (5 min) |

### With Jitter (±30%)

```
actual_delay = delay ± (delay × 0.3)
```

For attempt 3 (4s base):
- Range: 2.8s - 5.2s
- Prevents synchronized reconnection storms

---

## 🎯 Best Practices

### 1. Monitor Connection Health

```rust
// Check health periodically
if !client.is_connection_healthy() {
    // Trigger alerts, fallback mechanisms, etc.
}
```

### 2. Log Reconnection Events

```rust
let status = client.get_reconnect_status();
log::info!("Connection status: {}", status);
```

### 3. Use Appropriate Config

- **Critical apps** (payment, alerts): Use `aggressive()`
- **Normal apps**: Use `default()`
- **Battery devices**: Use `conservative()`

### 4. Handle Max Retries

```rust
match client.reconnect_with_backoff().await {
    Ok(()) => { /* Success */ }
    Err(e) => {
        // Max retries exceeded - alert admin
        // Consider shutdown or manual intervention
    }
}
```

### 5. Don't Spam Reconnect

```rust
// ❌ Bad: Infinite reconnect loop
loop {
    client.reconnect_with_backoff().await.ok();
}

// ✅ Good: Respect max retries
if let Err(e) = client.reconnect_with_backoff().await {
    log::error!("Max retries exceeded: {}", e);
    // Stop trying, alert admin
}
```

---

## 🔍 Troubleshooting

### High Consecutive Failures

**Problem**: `consecutive_failures > 5`

**Possible Causes**:
- Network connectivity issues
- WhatsApp server downtime
- Authentication/credential problems
- Firewall blocking

**Solutions**:
1. Check network connectivity
2. Verify credentials/session
3. Check WhatsApp status
4. Review firewall rules

### Low Success Rate

**Problem**: `success_rate() < 50%`

**Possible Causes**:
- Unstable network
- Server-side rate limiting
- Configuration issues

**Solutions**:
1. Switch to more conservative config
2. Increase `max_delay`
3. Add jitter to prevent thundering herd

### Frequent Reconnections

**Problem**: Connection drops often

**Possible Causes**:
- Keepalive timeout too long
- Network instability
- Server-side issues

**Solutions**:
1. Check keepalive configuration
2. Monitor network quality
3. Consider using `aggressive()` mode

---

## 📊 Example Output

```
Connection Status: Connected
Attempts: 15 (Success: 12, Failed: 3)
Success Rate: 80.0%
Consecutive Failures: 0/10
Total Reconnect Time: 2m 15s
Current Attempt: #0
```

---

## 🚀 Performance Impact

- **Memory**: ~1KB per client for metrics
- **CPU**: Negligible (state machine + simple math)
- **Network**: Reduced reconnect storms via jitter
- **Battery**: Improved via conservative mode

---

## 📝 Migration Guide

### From Simple Reconnect

**Before**:
```rust
client.reconnect_immediately().await;
```

**After**:
```rust
// Option 1: Immediate (same as before)
client.reconnect_immediately().await;

// Option 2: With backoff (recommended)
client.reconnect_with_backoff().await?;
```

---

## 📚 API Reference

### Client Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `reconnect_with_backoff()` | `Result<(), Error>` | Reconnect with exponential delay |
| `reconnect_immediately()` | `()` | Reconnect without delay |
| `get_reconnect_metrics()` | `ReconnectMetrics` | Get connection statistics |
| `get_connection_state()` | `ConnectionState` | Get current state |
| `get_reconnect_status()` | `String` | Human-readable status |
| `is_connection_healthy()` | `bool` | Check health |
| `configure_reconnect()` | `()` | Update config (placeholder) |

---

## 🧪 Testing

```rust
#[cfg(test)]
mod tests {
    use ruwa::reconnect::*;
    
    #[test]
    fn test_backoff_calculation() {
        let manager = ReconnectManager::with_defaults();
        
        // First attempt: 1s
        assert_eq!(manager.calculate_delay(), Duration::from_secs(1));
        
        // Fifth attempt: 16s
        manager.current_attempt.store(4, Ordering::Relaxed);
        assert_eq!(manager.calculate_delay(), Duration::from_secs(16));
    }
    
    #[test]
    fn test_jitter() {
        let config = ReconnectConfig {
            min_delay: Duration::from_secs(10),
            jitter_factor: 0.3,
            enable_jitter: true,
            ..Default::default()
        };
        
        let manager = ReconnectManager::new(config);
        let delay = manager.calculate_delay();
        
        // Should be 7-13s (10s ± 30%)
        assert!(delay >= Duration::from_secs(7));
        assert!(delay <= Duration::from_secs(13));
    }
}
```

---

## 📖 Related

- [Ping Feature Documentation](ping_example.md) - Test connection latency
- [Keepalive Implementation](../src/keepalive.rs) - Connection health monitoring
- [Transport Layer](../src/transport.rs) - WebSocket transport

---

**Implemented**: March 29, 2026  
**Version**: 1.0.0  
**Status**: ✅ Production Ready
