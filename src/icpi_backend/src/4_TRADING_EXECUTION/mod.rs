//! # Trading Execution Module
//!
//! Execute swaps on Kongswap DEX for portfolio rebalancing.
//!
//! ## Architecture
//! - **approvals/**: ICRC-2 approval flow for secure token spending
//! - **swaps/**: Kongswap swap execution (always via ckUSDT intermediary)
//! - **slippage/**: Slippage protection calculations and validation
//!
//! ## Key Constraints
//! - **ICRC-2 Only**: All swaps use approval flow (`pay_tx_id: None`)
//! - **ckUSDT Intermediary**: Every swap routes through ckUSDT
//! - **Sequential Execution**: No parallel swaps (Kongswap limitation)
//! - **Slippage Protected**: Default 2% max slippage enforced
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use crate::_4_TRADING_EXECUTION;
//! use crate::types::TrackedToken;
//! use candid::Nat;
//!
//! // Buy ALEX with 1 ckUSDT (e6 decimals)
//! let swap_result = _4_TRADING_EXECUTION::swaps::execute_swap(
//!     &TrackedToken::ckUSDT,
//!     Nat::from(1_000_000u64),
//!     &TrackedToken::ALEX,
//!     0.02 // 2% max slippage
//! ).await?;
//!
//! println!("Received {} ALEX", swap_result.receive_amount);
//! ```
//!
//! ## Swap Flow
//! 1. **Validate**: Check amounts, slippage params
//! 2. **Approve**: Backend approves Kongswap via ICRC-2
//! 3. **Query Price**: Get expected output for slippage check
//! 4. **Execute**: Call Kongswap `swap()` with ICRC-2 flow
//! 5. **Validate Result**: Ensure slippage within limits
//! 6. **Log**: Record swap for history and debugging
//!
//! ## Safety Features
//! - Approval expires after 5 minutes
//! - Slippage validation prevents bad trades
//! - Comprehensive logging for audit trail
//! - Input validation before expensive operations

pub mod approvals;
pub mod swaps;
pub mod slippage;
