















use log::{debug, error, info};
use rand::Rng;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    
    Disconnected,
    
    Connecting,
    
    Connected,
    
    Reconnecting,
    
    Failed,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Disconnected => write!(f, "Disconnected"),
            ConnectionState::Connecting => write!(f, "Connecting"),
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Reconnecting => write!(f, "Reconnecting"),
            ConnectionState::Failed => write!(f, "Failed"),
        }
    }
}


#[derive(Debug, Clone)]
pub struct ReconnectMetrics {
    
    pub total_attempts: u64,
    
    pub successful_connections: u64,
    
    pub failed_attempts: u64,
    
    pub consecutive_failures: u32,
    
    pub max_consecutive_failures: u32,
    
    pub total_reconnect_time_secs: u64,
    
    pub last_successful_connection: Option<u64>,
    
    pub last_attempt: Option<u64>,
}

impl Default for ReconnectMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconnectMetrics {
    pub fn new() -> Self {
        Self {
            total_attempts: 0,
            successful_connections: 0,
            failed_attempts: 0,
            consecutive_failures: 0,
            max_consecutive_failures: 0,
            total_reconnect_time_secs: 0,
            last_successful_connection: None,
            last_attempt: None,
        }
    }

    
    pub fn record_success(&mut self) {
        self.total_attempts += 1;
        self.successful_connections += 1;
        self.consecutive_failures = 0;
        self.last_successful_connection = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
        );
    }

    
    pub fn record_failure(&mut self) {
        self.total_attempts += 1;
        self.failed_attempts += 1;
        self.consecutive_failures += 1;
        self.max_consecutive_failures = self.max_consecutive_failures.max(self.consecutive_failures);
        self.last_attempt = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
        );
    }

    
    pub fn record_reconnect_time(&mut self, duration: Duration) {
        self.total_reconnect_time_secs += duration.as_secs();
    }

    
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            return 0.0;
        }
        (self.successful_connections as f64 / self.total_attempts as f64) * 100.0
    }

    
    pub fn uptime(&self) -> Option<Duration> {
        self.last_successful_connection.map(|last_success| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs();
            Duration::from_secs(now.saturating_sub(last_success))
        })
    }
}


#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    
    pub min_delay: Duration,
    
    pub max_delay: Duration,
    
    pub backoff_base: f64,
    
    
    pub max_retries: u32,
    
    pub enable_jitter: bool,
    
    pub jitter_factor: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            min_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300), 
            backoff_base: 2.0,
            max_retries: 10,
            enable_jitter: true,
            jitter_factor: 0.3,
        }
    }
}

impl ReconnectConfig {
    
    pub fn new(
        min_delay: Duration,
        max_delay: Duration,
        max_retries: u32,
    ) -> Self {
        Self {
            min_delay,
            max_delay,
            max_retries,
            ..Default::default()
        }
    }

    
    pub fn aggressive() -> Self {
        Self {
            min_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            max_retries: 20,
            jitter_factor: 0.2,
            ..Default::default()
        }
    }

    
    pub fn conservative() -> Self {
        Self {
            min_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(600), 
            max_retries: 5,
            jitter_factor: 0.4,
            ..Default::default()
        }
    }
}


pub struct ReconnectManager {
    config: ReconnectConfig,
    metrics: Arc<parking_lot::Mutex<ReconnectMetrics>>,
    current_attempt: AtomicU32,
    state: Arc<parking_lot::RwLock<ConnectionState>>,
    last_reconnect_start: AtomicU64,
}

impl ReconnectManager {
    
    pub fn new(config: ReconnectConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(parking_lot::Mutex::new(ReconnectMetrics::new())),
            current_attempt: AtomicU32::new(0),
            state: Arc::new(parking_lot::RwLock::new(ConnectionState::Disconnected)),
            last_reconnect_start: AtomicU64::new(0),
        }
    }

    
    pub fn with_defaults() -> Self {
        Self::new(ReconnectConfig::default())
    }

    
    pub fn state(&self) -> ConnectionState {
        *self.state.read()
    }

    
    pub fn set_state(&self, state: ConnectionState) {
        info!("Connection state changed: {}", state);
        *self.state.write() = state;
    }

    
    pub fn metrics(&self) -> ReconnectMetrics {
        self.metrics.lock().clone()
    }

    
    pub fn should_reconnect(&self) -> bool {
        if self.config.max_retries == 0 {
            return true; 
        }

        let metrics = self.metrics.lock();
        metrics.consecutive_failures < self.config.max_retries
    }

    
    pub fn calculate_delay(&self) -> Duration {
        let attempt = self.current_attempt.load(Ordering::Relaxed);
        
        
        let exponential_delay = self.config.min_delay.as_secs_f64()
            * self.config.backoff_base.powi(attempt as i32);
        
        
        let capped_delay = exponential_delay.min(self.config.max_delay.as_secs_f64());
        
        
        let final_delay = if self.config.enable_jitter {
            let jitter_range = (capped_delay * self.config.jitter_factor) as u64;
            let jitter = if jitter_range > 0 {
                rand::rng().random_range(0..jitter_range * 2) as f64 - jitter_range as f64
            } else {
                0.0
            };
            (capped_delay + jitter).max(self.config.min_delay.as_secs_f64())
        } else {
            capped_delay
        };
        
        Duration::from_secs_f64(final_delay)
    }

    
    pub fn start_reconnect(&self) {
        self.current_attempt.fetch_add(1, Ordering::Relaxed);
        self.set_state(ConnectionState::Reconnecting);
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        self.last_reconnect_start.store(timestamp, Ordering::Relaxed);
        
        let mut metrics = self.metrics.lock();
        metrics.record_failure();
    }

    
    pub fn record_success(&self) {
        self.current_attempt.store(0, Ordering::Relaxed);
        self.set_state(ConnectionState::Connected);
        
        
        let start = self.last_reconnect_start.load(Ordering::Relaxed);
        if start > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs();
            let duration = Duration::from_secs(now.saturating_sub(start));
            self.metrics.lock().record_reconnect_time(duration);
        }
        
        self.metrics.lock().record_success();
    }

    
    pub fn record_failure(&self) {
        self.set_state(ConnectionState::Reconnecting);
        
        if !self.should_reconnect() {
            self.set_state(ConnectionState::Failed);
            error!(
                "Max reconnection attempts exceeded ({})",
                self.config.max_retries
            );
        }
    }

    
    pub async fn wait_before_retry(&self) {
        let delay = self.calculate_delay();
        debug!(
            "Waiting {:?} before reconnection attempt #{}",
            delay,
            self.current_attempt.load(Ordering::Relaxed)
        );
        sleep(delay).await;
    }

    
    pub fn reset(&self) {
        self.current_attempt.store(0, Ordering::Relaxed);
        self.set_state(ConnectionState::Disconnected);
        *self.metrics.lock() = ReconnectMetrics::new();
    }

    
    pub fn is_healthy(&self) -> bool {
        let metrics = self.metrics.lock();
        
        
        
        
        
        let state = *self.state.read();
        
        if state == ConnectionState::Failed {
            return false;
        }
        
        if metrics.consecutive_failures > 5 {
            return false;
        }
        
        if metrics.total_attempts > 10 && metrics.success_rate() < 50.0 {
            return false;
        }
        
        true
    }

    
    pub fn status_report(&self) -> String {
        let metrics = self.metrics.lock();
        let state = *self.state.read();
        
        format!(
            "Connection Status: {}\n\
             Attempts: {} (Success: {}, Failed: {})\n\
             Success Rate: {:.1}%\n\
             Consecutive Failures: {}/{}\n\
             Total Reconnect Time: {}m {}s\n\
             Current Attempt: #{}",
            state,
            metrics.total_attempts,
            metrics.successful_connections,
            metrics.failed_attempts,
            metrics.success_rate(),
            metrics.consecutive_failures,
            self.config.max_retries,
            metrics.total_reconnect_time_secs / 60,
            metrics.total_reconnect_time_secs % 60,
            self.current_attempt.load(Ordering::Relaxed)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let mut metrics = ReconnectMetrics::new();
        
        metrics.record_success();
        assert_eq!(metrics.total_attempts, 1);
        assert_eq!(metrics.successful_connections, 1);
        assert_eq!(metrics.success_rate(), 100.0);
        
        metrics.record_failure();
        metrics.record_failure();
        assert_eq!(metrics.consecutive_failures, 2);
        assert_eq!(metrics.failed_attempts, 2);
        assert!((metrics.success_rate() - 33.3).abs() < 1.0);
    }

    #[test]
    fn test_exponential_backoff() {
        let config = ReconnectConfig {
            min_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_base: 2.0,
            enable_jitter: false,
            ..Default::default()
        };
        
        let manager = ReconnectManager::new(config);
        
        
        manager.current_attempt.store(0, Ordering::Relaxed);
        assert_eq!(manager.calculate_delay(), Duration::from_secs(1));
        
        
        manager.current_attempt.store(1, Ordering::Relaxed);
        assert_eq!(manager.calculate_delay(), Duration::from_secs(2));
        
        
        manager.current_attempt.store(2, Ordering::Relaxed);
        assert_eq!(manager.calculate_delay(), Duration::from_secs(4));
        
        
        manager.current_attempt.store(9, Ordering::Relaxed);
        assert_eq!(manager.calculate_delay(), Duration::from_secs(60));
    }

    #[test]
    fn test_jitter() {
        let config = ReconnectConfig {
            min_delay: Duration::from_secs(10),
            max_delay: Duration::from_secs(60),
            jitter_factor: 0.3,
            enable_jitter: true,
            ..Default::default()
        };
        
        let manager = ReconnectManager::new(config);
        manager.current_attempt.store(0, Ordering::Relaxed);
        
        
        let delay = manager.calculate_delay();
        assert!(delay >= Duration::from_secs(7));
        assert!(delay <= Duration::from_secs(13));
    }

    #[test]
    fn test_should_reconnect() {
        let config = ReconnectConfig {
            max_retries: 3,
            ..Default::default()
        };
        
        let manager = ReconnectManager::new(config);
        
        manager.metrics.lock().consecutive_failures = 2;
        assert!(manager.should_reconnect());
        
        manager.metrics.lock().consecutive_failures = 3;
        assert!(!manager.should_reconnect());
        assert_eq!(manager.state(), ConnectionState::Failed);
    }

    #[test]
    fn test_unlimited_retries() {
        let config = ReconnectConfig {
            max_retries: 0, 
            ..Default::default()
        };
        
        let manager = ReconnectManager::new(config);
        manager.metrics.lock().consecutive_failures = 100;
        
        assert!(manager.should_reconnect());
    }

    #[test]
    fn test_state_transitions() {
        let manager = ReconnectManager::with_defaults();
        
        assert_eq!(manager.state(), ConnectionState::Disconnected);
        
        manager.start_reconnect();
        assert_eq!(manager.state(), ConnectionState::Reconnecting);
        
        manager.record_success();
        assert_eq!(manager.state(), ConnectionState::Connected);
        
        manager.record_failure();
        assert_eq!(manager.state(), ConnectionState::Reconnecting);
    }
}
