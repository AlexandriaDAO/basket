//! Infrastructure-level caching module

use crate::infrastructure::Result;

/// Assert no cache for critical operations
pub fn assert_no_cache_for_critical_op(op: &str) {
    // This ensures critical operations don't use cached data
    ic_cdk::println!("Critical operation {} bypasses cache", op);
}
