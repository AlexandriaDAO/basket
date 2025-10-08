//! Infrastructure - Shared utilities and types
//! Foundation layer for all other modules

pub mod constants;
pub mod errors;
pub mod math;
pub mod types;
pub mod logging;
pub mod cache;
pub mod rate_limiting;
pub mod reentrancy;
pub mod stable_storage;

// Re-export commonly used items
pub use constants::*;
pub use errors::{IcpiError, Result, MintError, BurnError, RebalanceError, ValidationError, CalculationError, TradingError, KongswapError, SystemError};
pub use math::{multiply_and_divide, convert_decimals, calculate_mint_amount};
pub use reentrancy::{MintGuard, BurnGuard};
