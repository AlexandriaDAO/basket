//! Display module - Index state formatting for UI

use crate::types::portfolio::IndexState;
use crate::infrastructure::Result;

/// Get index state for display (with optional caching)
///
/// Returns complete portfolio state including:
/// - Total value in USD
/// - Current token positions
/// - Target allocations
/// - Allocation deviations
/// - ckUSDT reserves
///
/// IMPORTANT: Propagates errors instead of silently returning empty state
/// This ensures callers are aware of failures in portfolio calculation
pub async fn get_index_state_cached() -> Result<IndexState> {
    // Call the portfolio value module to get real state
    // Propagate errors up so they're visible to API consumers
    crate::_2_CRITICAL_DATA::portfolio_value::get_portfolio_state_uncached().await
}
