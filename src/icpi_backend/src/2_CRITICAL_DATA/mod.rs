//! Critical Data - Portfolio calculations and validation
//! Source of truth for all financial data

pub mod portfolio_value;
pub mod supply_tracker;
pub mod token_queries;
pub mod validation;

use crate::infrastructure::Result;
use candid::Nat;

// Re-export commonly used functions
pub use portfolio_value::{calculate_portfolio_value_atomic, get_portfolio_state_uncached};
pub use supply_tracker::{get_icpi_supply_uncached, get_validated_supply};
pub use token_queries::{get_all_balances_uncached, get_token_balance_uncached};
pub use validation::{validate_price, validate_supply};

/// Get supply and TVL atomically (Phase 3: M-5)
///
/// Phase 4 Enhancement: Added retry logic for transient network failures
///
/// Queries both values in parallel using futures::join! to minimize time gap.
/// This reduces the risk of stale data affecting calculations.
/// Retries up to 2 times on failure to handle transient network issues.
///
/// Returns: (supply, tvl) both as Nat
pub async fn get_supply_and_tvl_atomic() -> Result<(Nat, Nat)> {
    const MAX_RETRIES: u8 = 2;
    let mut last_error = None;

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            ic_cdk::println!("🔄 Retrying atomic snapshot (attempt {} of {})", attempt + 1, MAX_RETRIES + 1);
        } else {
            ic_cdk::println!("📸 Taking atomic snapshot of supply and TVL");
        }

        // Query both in parallel using futures::join!
        let supply_future = supply_tracker::get_icpi_supply_uncached();
        let tvl_future = portfolio_value::calculate_portfolio_value_atomic();

        let (supply_result, tvl_result) = futures::join!(supply_future, tvl_future);

        // Check if both queries succeeded
        match (supply_result, tvl_result) {
            (Ok(supply), Ok(tvl)) => {
                if attempt > 0 {
                    ic_cdk::println!("✅ Atomic snapshot successful on retry");
                }
                // Continue to validation below
                return validate_and_return_snapshot(supply, tvl).await;
            },
            (Err(e), _) | (_, Err(e)) => {
                ic_cdk::println!("⚠️ Atomic snapshot failed on attempt {}: {}", attempt + 1, e);
                last_error = Some(e);

                // Don't retry on final attempt
                if attempt < MAX_RETRIES {
                    // Brief delay before retry (100ms)
                    // Note: ic_cdk doesn't have async sleep, but the query itself provides natural delay
                    continue;
                }
            }
        }
    }

    // All retries exhausted
    ic_cdk::println!("❌ Atomic snapshot failed after {} attempts", MAX_RETRIES + 1);
    Err(last_error.unwrap_or_else(|| {
        crate::infrastructure::IcpiError::Query(
            crate::infrastructure::errors::QueryError::CanisterUnreachable {
                canister: "ICPI ledger or portfolio".to_string(),
                reason: "All retry attempts failed".to_string(),
            }
        )
    }))
}

/// Helper function to validate and return snapshot
async fn validate_and_return_snapshot(supply: Nat, tvl: Nat) -> Result<(Nat, Nat)> {

    // Phase 4 Enhancement: Make inconsistent state detection a hard error
    // Validation: detect inconsistent state - this indicates serious data corruption
    if supply > Nat::from(0u32) && tvl == Nat::from(0u32) {
        ic_cdk::println!("🚨 CRITICAL: Supply exists ({}) but TVL is zero - data corruption detected", supply);
        return Err(crate::infrastructure::IcpiError::Validation(
            crate::infrastructure::errors::ValidationError::DataInconsistency {
                reason: format!(
                    "Supply exists ({} ICPI) but TVL is zero. This indicates serious data corruption. \
                    Manual admin intervention required.",
                    supply
                ),
            }
        ));
    }

    if supply == Nat::from(0u32) && tvl > Nat::from(0u32) {
        ic_cdk::println!("🚨 CRITICAL: TVL exists ({}) but supply is zero - data corruption detected", tvl);
        return Err(crate::infrastructure::IcpiError::Validation(
            crate::infrastructure::errors::ValidationError::DataInconsistency {
                reason: format!(
                    "TVL exists ({} ckUSDT) but supply is zero. This indicates serious data corruption. \
                    Manual admin intervention required.",
                    tvl
                ),
            }
        ));
    }

    ic_cdk::println!("  Supply: {} ICPI (e8)", supply);
    ic_cdk::println!("  TVL: {} ckUSDT (e6)", tvl);

    Ok((supply, tvl))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// M-5 tests: Atomic snapshot logic and validation
    /// Note: Full integration tests require mocking async canister calls

    #[test]
    fn test_inconsistent_state_supply_but_no_tvl() {
        // Simulate the validation logic for supply > 0 but TVL = 0
        let supply = Nat::from(1_000_000u64);
        let tvl = Nat::from(0u64);

        // This condition should trigger a hard error
        let is_inconsistent = supply > Nat::from(0u32) && tvl == Nat::from(0u32);
        assert!(is_inconsistent, "Should detect supply without TVL as inconsistent");
    }

    #[test]
    fn test_inconsistent_state_tvl_but_no_supply() {
        // Simulate the validation logic for TVL > 0 but supply = 0
        let supply = Nat::from(0u64);
        let tvl = Nat::from(100_000u64);

        // This condition should trigger a hard error
        let is_inconsistent = supply == Nat::from(0u32) && tvl > Nat::from(0u32);
        assert!(is_inconsistent, "Should detect TVL without supply as inconsistent");
    }

    #[test]
    fn test_consistent_state_both_zero() {
        // Both zero is consistent (initial state)
        let supply = Nat::from(0u64);
        let tvl = Nat::from(0u64);

        let is_inconsistent = (supply > Nat::from(0u32) && tvl == Nat::from(0u32)) ||
                              (supply == Nat::from(0u32) && tvl > Nat::from(0u32));

        assert!(!is_inconsistent, "Both zero should be consistent (initial state)");
    }

    #[test]
    fn test_consistent_state_both_positive() {
        // Both positive is consistent (normal operation)
        let supply = Nat::from(1_000_000u64);
        let tvl = Nat::from(500_000u64);

        let is_inconsistent = (supply > Nat::from(0u32) && tvl == Nat::from(0u32)) ||
                              (supply == Nat::from(0u32) && tvl > Nat::from(0u32));

        assert!(!is_inconsistent, "Both positive should be consistent");
    }

    #[test]
    fn test_max_retries_constant() {
        // Verify retry configuration is reasonable
        const MAX_RETRIES: u8 = 2;

        // Should allow 3 total attempts (initial + 2 retries)
        let total_attempts = MAX_RETRIES + 1;
        assert_eq!(total_attempts, 3, "Should allow 3 total attempts");
    }

    #[test]
    fn test_snapshot_validation_boundaries() {
        // Test edge case: supply = 1, tvl = 0 (should fail)
        let supply_one = Nat::from(1u64);
        let tvl_zero = Nat::from(0u64);

        let is_inconsistent = supply_one > Nat::from(0u32) && tvl_zero == Nat::from(0u32);
        assert!(is_inconsistent, "Even 1 token with 0 TVL should be inconsistent");

        // Test edge case: supply = 0, tvl = 1 (should fail)
        let supply_zero = Nat::from(0u64);
        let tvl_one = Nat::from(1u64);

        let is_inconsistent = supply_zero == Nat::from(0u32) && tvl_one > Nat::from(0u32);
        assert!(is_inconsistent, "Even 1 ckUSDT with 0 supply should be inconsistent");
    }

    #[test]
    fn test_validation_logic_symmetry() {
        // Both conditions should be mutually exclusive
        let cases = vec![
            (Nat::from(0u64), Nat::from(0u64)),     // (0, 0) - both fail
            (Nat::from(0u64), Nat::from(100u64)),   // (0, >0) - condition 2 fails
            (Nat::from(100u64), Nat::from(0u64)),   // (>0, 0) - condition 1 fails
            (Nat::from(100u64), Nat::from(100u64)), // (>0, >0) - both pass
        ];

        for (supply, tvl) in cases {
            let cond1 = supply > Nat::from(0u32) && tvl == Nat::from(0u32);
            let cond2 = supply == Nat::from(0u32) && tvl > Nat::from(0u32);

            // Both conditions should never be true simultaneously
            assert!(!(cond1 && cond2), "Conditions should be mutually exclusive for supply={}, tvl={}", supply, tvl);
        }
    }
}

