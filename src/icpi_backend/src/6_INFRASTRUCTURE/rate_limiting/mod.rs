//! Rate limiting module

use crate::infrastructure::Result;
use std::cell::RefCell;
use std::collections::HashMap;

const MAX_ENTRIES: usize = 10000;
const CLEANUP_INTERVAL: u64 = 3600_000_000_000; // 1 hour in nanoseconds
const CLEANUP_THRESHOLD: usize = 1000; // Cleanup when we have more than 1000 entries

thread_local! {
    static RATE_LIMITS: RefCell<HashMap<String, u64>> = RefCell::new(HashMap::new());
    static LAST_CLEANUP: RefCell<u64> = RefCell::new(0);
}

/// Check rate limit for an operation
pub fn check_rate_limit(key: &str, limit_nanos: u64) -> Result<()> {
    let now = ic_cdk::api::time();

    RATE_LIMITS.with(|limits| {
        let mut limits = limits.borrow_mut();

        // Perform cleanup when:
        // 1. We have too many entries (> MAX_ENTRIES) - emergency cleanup
        // 2. We have moderate entries (> CLEANUP_THRESHOLD) AND it's been > 1 hour since last cleanup
        let should_cleanup = if limits.len() > MAX_ENTRIES {
            // Emergency cleanup when exceeding max
            true
        } else if limits.len() > CLEANUP_THRESHOLD {
            // Check if periodic cleanup is due
            LAST_CLEANUP.with(|last| {
                let last_cleanup_time = *last.borrow();
                now - last_cleanup_time > CLEANUP_INTERVAL
            })
        } else {
            false
        };

        if should_cleanup {
            let cutoff = now - CLEANUP_INTERVAL;
            let before_count = limits.len();
            limits.retain(|_, &mut time| time > cutoff);
            let after_count = limits.len();

            // Update last cleanup time
            LAST_CLEANUP.with(|last| {
                *last.borrow_mut() = now;
            });

            ic_cdk::println!(
                "Rate limit cleanup: removed {} old entries ({} -> {})",
                before_count - after_count,
                before_count,
                after_count
            );
        }

        if let Some(last_time) = limits.get(key) {
            if now - last_time < limit_nanos {
                return Err(crate::infrastructure::IcpiError::Other(
                    format!("Rate limit exceeded. Please wait {} seconds",
                        (limit_nanos - (now - last_time)) / 1_000_000_000)
                ));
            }
        }

        limits.insert(key.to_string(), now);
        Ok(())
    })
}

/// Manually trigger rate limit cleanup
/// This can be called periodically by a timer to ensure cleanup happens
/// even if there's low activity
pub fn periodic_cleanup() {
    let now = ic_cdk::api::time();

    RATE_LIMITS.with(|limits| {
        let mut limits = limits.borrow_mut();

        if limits.is_empty() {
            return;
        }

        let cutoff = now - CLEANUP_INTERVAL;
        let before_count = limits.len();
        limits.retain(|_, &mut time| time > cutoff);
        let after_count = limits.len();

        if before_count > after_count {
            ic_cdk::println!(
                "Periodic rate limit cleanup: removed {} old entries ({} -> {})",
                before_count - after_count,
                before_count,
                after_count
            );
        }
    });

    // Update last cleanup time
    LAST_CLEANUP.with(|last| {
        *last.borrow_mut() = now;
    });
}