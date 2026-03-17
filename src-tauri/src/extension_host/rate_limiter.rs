/**
 * Rate Limiter
 *
 * Prevents DOS attacks by limiting the rate of requests from extensions.
 * Uses token bucket algorithm via the governor crate.
 */

use governor::{Quota, RateLimiter as GovernorRateLimiter, state::InMemoryState, clock::DefaultClock};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::num::NonZeroU32;

pub struct RateLimiter {
    /// Per-extension rate limiters
    limiters: Arc<Mutex<HashMap<String, Arc<GovernorRateLimiter<governor::state::direct::NotKeyed, InMemoryState, DefaultClock>>>>>,

    /// Default quota: 100 requests per second per extension
    default_quota: Quota,
}

impl RateLimiter {
    pub fn new() -> Self {
        // Default: 100 requests per second
        let quota = Quota::per_second(NonZeroU32::new(100).unwrap());

        Self {
            limiters: Arc::new(Mutex::new(HashMap::new())),
            default_quota: quota,
        }
    }

    pub fn with_quota(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(
            NonZeroU32::new(requests_per_second).unwrap_or(NonZeroU32::new(100).unwrap())
        );

        Self {
            limiters: Arc::new(Mutex::new(HashMap::new())),
            default_quota: quota,
        }
    }

    /// Check if a request should be allowed
    pub fn check_rate_limit(&self, extension_id: &str) -> Result<(), String> {
        let limiter = self.get_or_create_limiter(extension_id);

        match limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Rate limit exceeded for extension '{}'. Please slow down.",
                extension_id
            )),
        }
    }

    /// Get or create a rate limiter for an extension
    fn get_or_create_limiter(&self, extension_id: &str) -> Arc<GovernorRateLimiter<governor::state::direct::NotKeyed, InMemoryState, DefaultClock>> {
        let mut limiters = self.limiters.lock().unwrap();

        limiters.entry(extension_id.to_string())
            .or_insert_with(|| {
                Arc::new(GovernorRateLimiter::direct(self.default_quota))
            })
            .clone()
    }

    /// Remove rate limiter for an extension (called when extension unloads)
    pub fn remove_limiter(&self, extension_id: &str) {
        let mut limiters = self.limiters.lock().unwrap();
        limiters.remove(extension_id);
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_rate_limiting() {
        let limiter = RateLimiter::with_quota(10); // 10 requests per second
        let ext_id = "test.extension";

        // First 10 requests should succeed
        for _ in 0..10 {
            assert!(limiter.check_rate_limit(ext_id).is_ok());
        }

        // 11th request should fail
        assert!(limiter.check_rate_limit(ext_id).is_err());

        // Wait for quota to refill
        thread::sleep(Duration::from_millis(1100));

        // Should work again
        assert!(limiter.check_rate_limit(ext_id).is_ok());
    }

    #[test]
    fn test_per_extension_limits() {
        let limiter = RateLimiter::with_quota(5);

        // Exhaust quota for extension1
        for _ in 0..5 {
            limiter.check_rate_limit("ext1").unwrap();
        }
        assert!(limiter.check_rate_limit("ext1").is_err());

        // Extension2 should still have quota
        assert!(limiter.check_rate_limit("ext2").is_ok());
    }
}
