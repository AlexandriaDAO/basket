//! Health monitoring module

use crate::types::common::HealthStatus;
use crate::types::TrackedToken;

/// Get system health status
pub fn get_health_status() -> HealthStatus {
    HealthStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        tracked_tokens: get_tracked_tokens(),
        last_rebalance: Some(0), // TODO: Get from rebalancer state
        cycles_balance: ic_cdk::api::canister_balance128(),
    }
}

/// Get list of tracked tokens
pub fn get_tracked_tokens() -> Vec<String> {
    TrackedToken::all()
        .into_iter()
        .map(|t| t.to_symbol().to_string())
        .collect()
}
