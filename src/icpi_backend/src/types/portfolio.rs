use candid::{CandidType, Deserialize, Nat};
use serde::Serialize;
use super::tokens::TrackedToken;

// Current holdings with precision handling
#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct CurrentPosition {
    pub token: TrackedToken,
    pub balance: Nat,           // Raw token balance with proper decimals
    pub usd_value: f64,         // USD value (using f64 for Candid compatibility)
    pub percentage: f64,        // Percentage of portfolio
}

// Combined state for rebalancing decisions
#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct IndexState {
    pub total_value: f64,
    pub current_positions: Vec<CurrentPosition>,
    pub target_allocations: Vec<super::rebalancing::TargetAllocation>,
    pub deviations: Vec<super::rebalancing::AllocationDeviation>,
    pub timestamp: u64,
    pub ckusdt_balance: Nat,  // Track available ckUSDT for rebalancing
}

// Cached data structures
#[derive(CandidType, Deserialize, Default)]
pub struct CachedLockCanisters {
    pub canisters: Vec<String>,
    pub timestamp: u64,
}

// TVL calculation types
#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct TokenTvl {
    pub token: TrackedToken,
    pub locked_value_usd: f64,
    pub percentage: f64,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct TvlSummary {
    pub total_tvl_usd: f64,
    pub token_tvls: Vec<TokenTvl>,
    pub timestamp: u64,
}