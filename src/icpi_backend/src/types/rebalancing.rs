use candid::{CandidType, Deserialize};
use serde::Serialize;
use super::tokens::TrackedToken;

// Target allocation from locked liquidity
#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct TargetAllocation {
    pub token: TrackedToken,
    pub target_percentage: f64,
    pub target_usd_value: f64,
}

// Allocation deviation
#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct AllocationDeviation {
    pub token: TrackedToken,
    pub current_pct: f64,
    pub target_pct: f64,
    pub deviation_pct: f64,     // target - current (can be negative)
    pub usd_difference: f64,    // Amount to buy (positive) or sell (negative)
    pub trade_size_usd: f64,    // 10% of difference for hourly rebalance
}

// Rebalancing action
#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub enum RebalanceAction {
    Buy { token: TrackedToken, usdt_amount: f64 },
    Sell { token: TrackedToken, usdt_value: f64 },
    None,
}

// Rebalancing status for monitoring
#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct RebalanceStatus {
    pub last_rebalance: Option<u64>,
    pub next_rebalance: u64,
    pub is_timer_active: bool,
    pub pending_action: Option<RebalanceAction>,
}