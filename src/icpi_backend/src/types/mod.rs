pub mod tokens;
pub mod icrc;
pub mod kongswap;
pub mod portfolio;
pub mod rebalancing;
pub mod common;

// Re-export commonly used types
pub use tokens::{TrackedToken, TokenMetadata, ICPI_CANISTER_ID, CKUSDT_CANISTER_ID};
pub use icrc::{Account, TransferArgs, TransferResult};
pub use kongswap::{SwapArgs, SwapReply, SwapAmountsReply, SwapAmountsResult, LPBalancesReply, UserBalancesReply, UserBalancesResult, TxId};
pub use portfolio::{CurrentPosition, IndexState};
pub use rebalancing::{TargetAllocation, AllocationDeviation, RebalanceAction};
pub use common::HealthStatus;

// Constants
pub const KONG_LOCKER_ID: &str = "eazgb-giaaa-aaaap-qqc2q-cai";
pub const KONGSWAP_BACKEND_ID: &str = "2ipq2-uqaaa-aaaar-qailq-cai";

// Utility functions
use candid::{Nat, Principal};
use rust_decimal::Decimal;

pub fn validate_principal(principal_str: &str) -> Result<Principal, String> {
    Principal::from_text(principal_str)
        .map_err(|e| format!("Invalid principal: {}", e))
}

pub fn decimal_to_f64(decimal: &Decimal) -> f64 {
    decimal.to_string().parse::<f64>().unwrap_or(0.0)
}

pub fn f64_to_decimal(value: f64) -> Decimal {
    Decimal::from_f64_retain(value).unwrap_or(Decimal::ZERO)
}