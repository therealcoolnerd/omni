use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Rate limiter for package operations to prevent abuse
#[derive(Debug, Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<RateLimiterInner>>,
}

#[derive(Debug)]
struct RateLimiterInner {
    operations: HashMap<String, Vec<Instant>>,
    max_operations_per_minute: usize,
    max_operations_per_hour: usize,
    cleanup_last_run: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter with default limits
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RateLimiterInner {
                operations: HashMap::new(),
                max_operations_per_minute: 10, // 10 operations per minute
                max_operations_per_hour: 100,  // 100 operations per hour
                cleanup_last_run: Instant::now(),
            })),
        }
    }

    /// Create a rate limiter with custom limits
    pub fn with_limits(per_minute: usize, per_hour: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RateLimiterInner {
                operations: HashMap::new(),
                max_operations_per_minute: per_minute,
                max_operations_per_hour: per_hour,
                cleanup_last_run: Instant::now(),
            })),
        }
    }

    /// Check if an operation is allowed for the given key (e.g., package name, user IP)
    pub fn check_rate_limit(&self, key: &str, operation: &str) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("Rate limiter mutex poisoned"))?;

        // Periodic cleanup of old entries
        if inner.cleanup_last_run.elapsed() > Duration::from_secs(300) {
            // 5 minutes
            self.cleanup_old_entries(&mut inner);
        }

        let rate_key = format!("{}:{}", key, operation);
        let now = Instant::now();

        // Get max limits to avoid borrowing issues
        let max_operations_per_hour = inner.max_operations_per_hour;
        let max_operations_per_minute = inner.max_operations_per_minute;

        // Get or create the operation history for this key
        let operations = inner
            .operations
            .entry(rate_key.clone())
            .or_insert_with(Vec::new);

        // Remove operations older than 1 hour
        operations.retain(|&op_time| now.duration_since(op_time) < Duration::from_secs(3600));

        // Check hourly limit
        if operations.len() >= max_operations_per_hour {
            warn!("Hourly rate limit exceeded for key: {}", rate_key);
            return Err(anyhow!("Rate limit exceeded: too many operations per hour"));
        }

        // Check per-minute limit
        let recent_operations = operations
            .iter()
            .filter(|&&op_time| now.duration_since(op_time) < Duration::from_secs(60))
            .count();

        if recent_operations >= max_operations_per_minute {
            warn!("Per-minute rate limit exceeded for key: {}", rate_key);
            return Err(anyhow!(
                "Rate limit exceeded: too many operations per minute"
            ));
        }

        // Record this operation
        operations.push(now);
        info!(
            "Rate limit check passed for {}: {}/{} per minute, {}/{} per hour",
            rate_key,
            recent_operations + 1,
            max_operations_per_minute,
            operations.len(),
            max_operations_per_hour
        );

        Ok(())
    }

    /// Reset rate limits for a specific key
    pub fn reset_limits(&self, key: &str) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("Rate limiter mutex poisoned"))?;

        // Remove all entries that start with the key
        inner.operations.retain(|k, _| !k.starts_with(key));
        info!("Reset rate limits for key: {}", key);

        Ok(())
    }

    /// Get current rate limit status for a key
    pub fn get_status(&self, key: &str, operation: &str) -> Result<RateLimitStatus> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("Rate limiter mutex poisoned"))?;

        let rate_key = format!("{}:{}", key, operation);
        let now = Instant::now();

        let operations = inner.operations.get(&rate_key);

        if let Some(ops) = operations {
            let recent_ops = ops
                .iter()
                .filter(|&&op_time| now.duration_since(op_time) < Duration::from_secs(60))
                .count();

            let hourly_ops = ops.len();

            Ok(RateLimitStatus {
                operations_this_minute: recent_ops,
                operations_this_hour: hourly_ops,
                max_per_minute: inner.max_operations_per_minute,
                max_per_hour: inner.max_operations_per_hour,
                can_proceed: recent_ops < inner.max_operations_per_minute
                    && hourly_ops < inner.max_operations_per_hour,
            })
        } else {
            Ok(RateLimitStatus {
                operations_this_minute: 0,
                operations_this_hour: 0,
                max_per_minute: inner.max_operations_per_minute,
                max_per_hour: inner.max_operations_per_hour,
                can_proceed: true,
            })
        }
    }

    fn cleanup_old_entries(&self, inner: &mut RateLimiterInner) {
        let now = Instant::now();
        let cutoff = Duration::from_secs(3600); // 1 hour

        inner.operations.retain(|_, operations| {
            operations.retain(|&op_time| now.duration_since(op_time) < cutoff);
            !operations.is_empty()
        });

        inner.cleanup_last_run = now;
        info!("Cleaned up old rate limit entries");
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub operations_this_minute: usize,
    pub operations_this_hour: usize,
    pub max_per_minute: usize,
    pub max_per_hour: usize,
    pub can_proceed: bool,
}

impl RateLimitStatus {
    pub fn remaining_minute(&self) -> usize {
        self.max_per_minute
            .saturating_sub(self.operations_this_minute)
    }

    pub fn remaining_hour(&self) -> usize {
        self.max_per_hour.saturating_sub(self.operations_this_hour)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_rate_limiter_basic() {
        let limiter = RateLimiter::with_limits(2, 5);

        // First two operations should succeed
        assert!(limiter.check_rate_limit("test", "install").is_ok());
        assert!(limiter.check_rate_limit("test", "install").is_ok());

        // Third operation within minute should fail
        assert!(limiter.check_rate_limit("test", "install").is_err());
    }

    #[test]
    fn test_rate_limiter_different_keys() {
        let limiter = RateLimiter::with_limits(1, 3);

        // Different keys should have independent limits
        assert!(limiter.check_rate_limit("user1", "install").is_ok());
        assert!(limiter.check_rate_limit("user2", "install").is_ok());

        // Same key should hit limit
        assert!(limiter.check_rate_limit("user1", "install").is_err());
    }

    #[test]
    fn test_rate_limiter_different_operations() {
        let limiter = RateLimiter::with_limits(1, 3);

        // Different operations for same key should have independent limits
        assert!(limiter.check_rate_limit("user1", "install").is_ok());
        assert!(limiter.check_rate_limit("user1", "remove").is_ok());

        // Same operation should hit limit
        assert!(limiter.check_rate_limit("user1", "install").is_err());
    }

    #[test]
    fn test_rate_limiter_status() {
        let limiter = RateLimiter::with_limits(3, 10);

        // Check initial status
        let status = limiter.get_status("test", "install").unwrap();
        assert_eq!(status.operations_this_minute, 0);
        assert_eq!(status.operations_this_hour, 0);
        assert!(status.can_proceed);

        // After one operation
        limiter.check_rate_limit("test", "install").unwrap();
        let status = limiter.get_status("test", "install").unwrap();
        assert_eq!(status.operations_this_minute, 1);
        assert_eq!(status.operations_this_hour, 1);
        assert!(status.can_proceed);
    }

    #[test]
    fn test_rate_limiter_reset() {
        let limiter = RateLimiter::with_limits(1, 3);

        // Hit the limit
        assert!(limiter.check_rate_limit("test", "install").is_ok());
        assert!(limiter.check_rate_limit("test", "install").is_err());

        // Reset and try again
        limiter.reset_limits("test").unwrap();
        assert!(limiter.check_rate_limit("test", "install").is_ok());
    }
}
