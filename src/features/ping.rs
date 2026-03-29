//! Manual ping functionality for low-latency server connectivity testing.
//!
//! This module provides on-demand ping functionality with:
//! - High priority (bypasses normal IQ queue)
//! - Configurable timeout (default 5 seconds for fast feedback)
//! - Accurate RTT (Round-Trip Time) measurement
//! - Sub-200ms latency target

use crate::client::Client;
use crate::request::IqError;
use log::debug;
use std::time::Duration;
use wacore_ng::iq::keepalive::KeepaliveSpec;
use wacore_ng::iq::spec::IqSpec;

/// Result of a ping operation containing latency metrics.
#[derive(Debug, Clone)]
pub struct PingResult {
    /// Round-Trip Time in milliseconds (time from send to receive).
    pub rtt_ms: u64,
    /// Server timestamp from the pong response (if available).
    pub server_timestamp: Option<f64>,
    /// Whether the server time offset was updated.
    pub time_offset_updated: bool,
}

impl PingResult {
    /// Creates a new PingResult with the given RTT.
    pub fn new(rtt_ms: u64) -> Self {
        Self {
            rtt_ms,
            server_timestamp: None,
            time_offset_updated: false,
        }
    }

    /// Returns whether the ping latency is under the target threshold.
    ///
    /// # Arguments
    /// * `threshold_ms` - The latency threshold in milliseconds (default: 200ms)
    pub fn is_under_threshold(&self, threshold_ms: u64) -> bool {
        self.rtt_ms < threshold_ms
    }

    /// Returns true if the ping was successful with good latency (< 200ms).
    pub fn is_good(&self) -> bool {
        self.rtt_ms < 200
    }

    /// Returns true if the ping latency is moderate (200-500ms).
    pub fn is_moderate(&self) -> bool {
        self.rtt_ms >= 200 && self.rtt_ms < 500
    }

    /// Returns true if the ping latency is poor (>= 500ms).
    pub fn is_poor(&self) -> bool {
        self.rtt_ms >= 500
    }
}

/// Default timeout for manual ping operations (5 seconds).
/// Shorter than keepalive timeout for faster feedback.
const PING_TIMEOUT: Duration = Duration::from_secs(5);

impl Client {
    /// Sends a manual ping to the WhatsApp server and measures the latency.
    ///
    /// This function provides on-demand connectivity testing with accurate
    /// Round-Trip Time (RTT) measurement. It's optimized for low latency
    /// and should achieve sub-200ms response times under normal conditions.
    ///
    /// # Features
    /// - **High Priority**: Uses a dedicated timeout to avoid queue delays
    /// - **Accurate Timing**: Measures RTT with millisecond precision
    /// - **Auto Time Sync**: Updates server time offset from response
    /// - **Fast Feedback**: 5-second timeout (vs 20s for keepalive)
    ///
    /// # Returns
    ///
    /// * `Ok(PingResult)` - Ping succeeded with latency metrics
    /// * `Err(IqError)` - Ping failed (timeout, disconnect, etc.)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::time::Duration;
    ///
    /// # async fn example(client: ruwa::Client) -> anyhow::Result<()> {
    /// // Send a ping and check latency
    /// let ping = client.ping().await?;
    /// println!("Server RTT: {}ms", ping.rtt_ms);
    ///
    /// if ping.is_good() {
    ///     println!("✓ Connection quality is good (< 200ms)");
    /// } else if ping.is_moderate() {
    ///     println!("⚠ Connection quality is moderate (200-500ms)");
    /// } else {
    ///     println!("✗ Connection quality is poor (> 500ms)");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance
    ///
    /// Typical latency ranges:
    /// - **Excellent**: < 100ms (local/regional server)
    /// - **Good**: 100-200ms (continental)
    /// - **Moderate**: 200-500ms (intercontinental)
    /// - **Poor**: > 500ms (high latency connection)
    pub async fn ping(&self) -> Result<PingResult, IqError> {
        self.ping_with_timeout(PING_TIMEOUT).await
    }

    /// Sends a manual ping with a custom timeout.
    ///
    /// This variant allows you to specify a custom timeout for the ping
    /// operation. Use shorter timeouts for faster failure detection,
    /// or longer timeouts for unreliable connections.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for a response
    ///
    /// # Returns
    ///
    /// * `Ok(PingResult)` - Ping succeeded with latency metrics
    /// * `Err(IqError)` - Ping failed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::time::Duration;
    ///
    /// # async fn example(client: ruwa::Client) -> anyhow::Result<()> {
    /// // Use a very short timeout for quick connectivity check
    /// match client.ping_with_timeout(Duration::from_secs(2)).await {
    ///     Ok(ping) => println!("Fast ping: {}ms", ping.rtt_ms),
    ///     Err(_) => println!("No response within 2 seconds"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ping_with_timeout(&self, timeout: Duration) -> Result<PingResult, IqError> {
        if !self.is_connected() {
            return Err(IqError::NotConnected);
        }

        debug!(target: "Client/Ping", "Sending manual ping");

        let start_ms = chrono::Utc::now().timestamp_millis();
        
        // Build ping IQ with custom timeout
        let iq = KeepaliveSpec::with_timeout(timeout).build_iq();
        
        // Send and wait for response
        match self.send_iq(iq).await {
            Ok(response_node) => {
                let end_ms = chrono::Utc::now().timestamp_millis();
                let rtt_ms = (end_ms - start_ms) as u64;
                
                debug!(target: "Client/Ping", "Received pong (RTT: {}ms)", rtt_ms);
                
                // Update server time offset with RTT compensation
                // WA Web: onClockSkewUpdate — Math.round((startTime + rtt/2) / 1000 - serverTime)
                self.unified_session.update_server_time_offset_with_rtt(
                    &response_node,
                    start_ms,
                    rtt_ms as i64,
                );
                
                // Extract server timestamp from response if available
                let server_timestamp = response_node.attrs.get("t")
                    .and_then(|t| t.to_string().parse::<f64>().ok());
                
                Ok(PingResult {
                    rtt_ms,
                    server_timestamp,
                    time_offset_updated: true,
                })
            }
            Err(e) => {
                debug!(target: "Client/Ping", "Ping failed: {:?}", e);
                Err(e)
            }
        }
    }

    /// Sends multiple pings and returns statistics.
    ///
    /// This function sends a series of pings and calculates statistics
    /// like min, max, average, and jitter. Useful for connection quality
    /// monitoring and diagnostics.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of pings to send (default: 4)
    /// * `interval` - Delay between pings (default: 1 second)
    ///
    /// # Returns
    ///
    /// * `Ok(PingStatistics)` - Statistics from the ping series
    /// * `Err(IqError)` - One of the pings failed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example(client: ruwa::Client) -> anyhow::Result<()> {
    /// // Send 5 pings with 1 second intervals
    /// let stats = client.ping_multiple(5, None).await?;
    /// println!("Ping statistics:");
    /// println!("  Min: {}ms", stats.min_rtt_ms);
    /// println!("  Max: {}ms", stats.max_rtt_ms);
    /// println!("  Avg: {}ms", stats.avg_rtt_ms);
    /// println!("  Jitter: {}ms", stats.jitter_ms);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ping_multiple(
        &self,
        count: usize,
        interval: Option<Duration>,
    ) -> Result<PingStatistics, IqError> {
        let interval = interval.unwrap_or(Duration::from_secs(1));
        let mut results = Vec::with_capacity(count);
        
        for i in 0..count {
            if i > 0 {
                tokio::time::sleep(interval).await;
            }
            
            let result = self.ping().await?;
            results.push(result);
        }
        
        Ok(PingStatistics::from_results(&results))
    }
}

/// Statistics from a series of ping operations.
#[derive(Debug, Clone)]
pub struct PingStatistics {
    /// Number of pings sent.
    pub count: usize,
    /// Minimum RTT in milliseconds.
    pub min_rtt_ms: u64,
    /// Maximum RTT in milliseconds.
    pub max_rtt_ms: u64,
    /// Average RTT in milliseconds.
    pub avg_rtt_ms: f64,
    /// Jitter (variance) in milliseconds.
    pub jitter_ms: f64,
    /// Number of successful pings.
    pub success_count: usize,
    /// Number of failed pings.
    pub fail_count: usize,
}

impl PingStatistics {
    /// Creates statistics from a slice of ping results.
    pub fn from_results(results: &[PingResult]) -> Self {
        if results.is_empty() {
            return Self {
                count: 0,
                min_rtt_ms: 0,
                max_rtt_ms: 0,
                avg_rtt_ms: 0.0,
                jitter_ms: 0.0,
                success_count: 0,
                fail_count: 0,
            };
        }
        
        let count = results.len();
        let rtts: Vec<u64> = results.iter().map(|r| r.rtt_ms).collect();
        
        let min_rtt_ms = *rtts.iter().min().unwrap_or(&0);
        let max_rtt_ms = *rtts.iter().max().unwrap_or(&0);
        let avg_rtt_ms = rtts.iter().sum::<u64>() as f64 / count as f64;
        
        // Calculate jitter (standard deviation)
        let variance = rtts.iter()
            .map(|&rtt| (rtt as f64 - avg_rtt_ms).powi(2))
            .sum::<f64>() / count as f64;
        let jitter_ms = variance.sqrt();
        
        Self {
            count,
            min_rtt_ms,
            max_rtt_ms,
            avg_rtt_ms,
            jitter_ms,
            success_count: count,
            fail_count: 0,
        }
    }
    
    /// Returns whether the connection quality is good (avg < 200ms).
    pub fn is_good(&self) -> bool {
        self.avg_rtt_ms < 200.0
    }
    
    /// Returns a quality rating string based on average RTT.
    pub fn quality_rating(&self) -> &'static str {
        if self.avg_rtt_ms < 100.0 {
            "Excellent"
        } else if self.avg_rtt_ms < 200.0 {
            "Good"
        } else if self.avg_rtt_ms < 500.0 {
            "Moderate"
        } else {
            "Poor"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ping_result_is_good() {
        let ping = PingResult::new(150);
        assert!(ping.is_good());
        assert!(ping.is_under_threshold(200));
    }
    
    #[test]
    fn test_ping_result_is_moderate() {
        let ping = PingResult::new(350);
        assert!(ping.is_moderate());
        assert!(!ping.is_good());
    }
    
    #[test]
    fn test_ping_result_is_poor() {
        let ping = PingResult::new(600);
        assert!(ping.is_poor());
        assert!(!ping.is_moderate());
    }
    
    #[test]
    fn test_ping_statistics_from_results() {
        let results = vec![
            PingResult::new(100),
            PingResult::new(150),
            PingResult::new(200),
            PingResult::new(150),
        ];
        
        let stats = PingStatistics::from_results(&results);
        
        assert_eq!(stats.count, 4);
        assert_eq!(stats.min_rtt_ms, 100);
        assert_eq!(stats.max_rtt_ms, 200);
        assert!((stats.avg_rtt_ms - 150.0).abs() < 0.01);
        assert!(stats.jitter_ms > 0.0);
        assert!(stats.is_good());
    }
    
    #[test]
    fn test_ping_statistics_quality_rating() {
        let excellent = PingStatistics::from_results(&[PingResult::new(50)]);
        assert_eq!(excellent.quality_rating(), "Excellent");
        
        let good = PingStatistics::from_results(&[PingResult::new(150)]);
        assert_eq!(good.quality_rating(), "Good");
        
        let moderate = PingStatistics::from_results(&[PingResult::new(300)]);
        assert_eq!(moderate.quality_rating(), "Moderate");
        
        let poor = PingStatistics::from_results(&[PingResult::new(600)]);
        assert_eq!(poor.quality_rating(), "Poor");
    }
}
