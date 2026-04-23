/**
 * Rate Limiter
 *
 * Prevents DOS attacks by limiting the rate of requests from extensions.
 * Uses token bucket algorithm via the governor crate.
 */
use governor::{
    clock::DefaultClock, state::InMemoryState, Quota, RateLimiter as GovernorRateLimiter,
};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

type GovernorLimiter =
    Arc<GovernorRateLimiter<governor::state::direct::NotKeyed, InMemoryState, DefaultClock>>;

pub struct RateLimiter {
    /// Per-extension rate limiters
    limiters: Arc<Mutex<HashMap<String, GovernorLimiter>>>,

    /// Default quota: 100 requests per second per extension
    default_quota: Quota,
}

impl RateLimiter {
    pub fn new() -> Self {
        // Default: 100 requests per second
        let quota = Quota::per_second(NonZeroU32::new(100).expect("100 is nonzero"));

        Self {
            limiters: Arc::new(Mutex::new(HashMap::new())),
            default_quota: quota,
        }
    }

    pub fn with_quota(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(
            NonZeroU32::new(requests_per_second)
                .unwrap_or(NonZeroU32::new(100).expect("100 is nonzero")),
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
    fn get_or_create_limiter(&self, extension_id: &str) -> GovernorLimiter {
        let mut limiters = self.limiters.lock().unwrap_or_else(|e| {
            tracing::warn!("Rate limiter lock poisoned, recovering: {}", e);
            e.into_inner()
        });

        limiters
            .entry(extension_id.to_string())
            .or_insert_with(|| Arc::new(GovernorRateLimiter::direct(self.default_quota)))
            .clone()
    }

    /// Remove rate limiter for an extension (called when extension unloads)
    pub fn remove_limiter(&self, extension_id: &str) {
        let mut limiters = self.limiters.lock().unwrap_or_else(|e| {
            tracing::warn!("Rate limiter lock poisoned, recovering: {}", e);
            e.into_inner()
        });
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

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::with_quota(20);

        // All 20 requests within the rate limit should be allowed
        for i in 0..20 {
            let result = limiter.check_rate_limit("within-limit-ext");
            assert!(result.is_ok(), "Request {} should have been allowed", i);
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::with_quota(5);

        // Exhaust quota
        for _ in 0..5 {
            limiter.check_rate_limit("over-limit-ext").unwrap();
        }

        // Next request should be blocked
        let result = limiter.check_rate_limit("over-limit-ext");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Rate limit exceeded"));
    }

    #[test]
    fn test_rate_limiter_default() {
        let limiter = RateLimiter::default();
        // Default allows 100 requests per second, so the first few should pass
        for _ in 0..10 {
            assert!(limiter.check_rate_limit("default-ext").is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_new() {
        let limiter = RateLimiter::new();
        // Same as default: 100 rps
        assert!(limiter.check_rate_limit("new-ext").is_ok());
    }

    #[test]
    fn test_rate_limiter_window_refill() {
        // After the quota window passes, the limiter should allow requests again
        let limiter = RateLimiter::with_quota(3);
        for _ in 0..3 {
            limiter.check_rate_limit("window-ext").unwrap();
        }
        assert!(limiter.check_rate_limit("window-ext").is_err());

        // Wait for the bucket to refill (1 second + small buffer)
        thread::sleep(Duration::from_millis(1100));
        assert!(limiter.check_rate_limit("window-ext").is_ok());
    }

    #[test]
    fn test_rate_limiter_per_extension_isolation() {
        let limiter = RateLimiter::with_quota(2);

        // Exhaust quota for ext-a
        limiter.check_rate_limit("ext-a").unwrap();
        limiter.check_rate_limit("ext-a").unwrap();
        assert!(limiter.check_rate_limit("ext-a").is_err());

        // ext-b should still have its own independent quota
        assert!(limiter.check_rate_limit("ext-b").is_ok());
        assert!(limiter.check_rate_limit("ext-b").is_ok());
        assert!(limiter.check_rate_limit("ext-b").is_err());

        // ext-c is also independent
        assert!(limiter.check_rate_limit("ext-c").is_ok());
    }

    #[test]
    fn test_rate_limiter_remove_limiter() {
        let limiter = RateLimiter::with_quota(2);

        // Use up quota for ext-rm
        limiter.check_rate_limit("ext-rm").unwrap();
        limiter.check_rate_limit("ext-rm").unwrap();
        assert!(limiter.check_rate_limit("ext-rm").is_err());

        // Remove the limiter; next check creates a fresh one
        limiter.remove_limiter("ext-rm");
        assert!(limiter.check_rate_limit("ext-rm").is_ok());
    }

    #[test]
    fn test_rate_limiter_with_quota_zero_uses_default() {
        // with_quota(0) should fall back to 100 since NonZeroU32::new(0) is None
        let limiter = RateLimiter::with_quota(0);
        // Should still work (default of 100)
        assert!(limiter.check_rate_limit("zero-ext").is_ok());
    }

    #[test]
    fn test_rate_limiter_error_contains_extension_id() {
        let limiter = RateLimiter::with_quota(1);
        limiter.check_rate_limit("error-ext").unwrap();
        let err = limiter.check_rate_limit("error-ext").unwrap_err();
        assert!(
            err.contains("error-ext"),
            "Error message should contain extension id"
        );
    }
}
