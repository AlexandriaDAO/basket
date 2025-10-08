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

