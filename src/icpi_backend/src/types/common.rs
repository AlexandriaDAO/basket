use candid::{CandidType, Deserialize};
use serde::Serialize;
use rust_decimal::Decimal;

// Health status
#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct HealthStatus {
    pub version: String,
    pub tracked_tokens: Vec<String>,
    pub last_rebalance: Option<u64>,
    pub cycles_balance: u128,
}

// Error recovery types
#[derive(Debug)]
pub enum PriceResult {
    Available(Decimal),
    Cached(Decimal),
    Unavailable,
}

// Common result types for better error handling
pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug, Clone)]
pub enum ServiceError {
    NotAuthorized(String),
    InvalidInput(String),
    InterCanisterError(String),
    InsufficientFunds(String),
    RebalancingError(String),
    CriticalError(String),
}