//! Display module - Index state formatting for UI

use crate::types::portfolio::IndexState;

/// Get index state for display (with optional caching)
///
/// Returns complete portfolio state including:
/// - Total value in USD
/// - Current token positions
/// - Target allocations
/// - Allocation deviations
/// - ckUSDT reserves
pub async fn get_index_state_cached() -> IndexState {
    // Call the portfolio value module to get real state
    match crate::_2_CRITICAL_DATA::portfolio_value::get_portfolio_state_uncached().await {
        Ok(state) => state,
        Err(e) => {
            ic_cdk::println!("⚠️ Failed to get portfolio state: {}", e);
            // Return empty state on error rather than panicking
            IndexState {
                total_value: 0.0,
                current_positions: Vec::new(),
                target_allocations: Vec::new(),
                deviations: Vec::new(),
                ckusdt_balance: candid::Nat::from(0u64),
                timestamp: ic_cdk::api::time(),
            }
        }
    }
}
