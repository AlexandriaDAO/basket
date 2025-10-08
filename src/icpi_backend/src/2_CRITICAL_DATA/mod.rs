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
/// Queries both values in parallel using futures::join! to minimize time gap.
/// This reduces the risk of stale data affecting calculations.
///
/// Returns: (supply, tvl) both as Nat
pub async fn get_supply_and_tvl_atomic() -> Result<(Nat, Nat)> {
    ic_cdk::println!("📸 Taking atomic snapshot of supply and TVL");

    // Query both in parallel using futures::join!
    let supply_future = supply_tracker::get_icpi_supply_uncached();
    let tvl_future = portfolio_value::calculate_portfolio_value_atomic();

    let (supply_result, tvl_result) = futures::join!(supply_future, tvl_future);

    let supply = supply_result?;
    let tvl = tvl_result?;

    // Validation: detect inconsistent state
    if supply > Nat::from(0u32) && tvl == Nat::from(0u32) {
        ic_cdk::println!("⚠️ WARNING: Supply exists but TVL is zero - possible data issue");
    }

    if supply == Nat::from(0u32) && tvl > Nat::from(0u32) {
        ic_cdk::println!("⚠️ WARNING: TVL exists but supply is zero - possible data issue");
    }

    ic_cdk::println!("  Supply: {} ICPI (e8)", supply);
    ic_cdk::println!("  TVL: {} ckUSDT (e6)", tvl);

    Ok((supply, tvl))
}

