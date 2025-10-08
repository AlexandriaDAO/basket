//! # Kong Liquidity Integration
//!
//! Query Kong Locker and Kongswap for:
//! - Locked liquidity amounts for TVL calculation
//! - Real-time token prices from liquidity pools
//! - Target allocation percentages based on locked TVL
//!
//! ## Architecture
//!
//! ### locker/
//! Queries kong_locker backend for list of all lock canisters.
//! Each user can have one lock canister that holds their LP tokens.
//!
//! ### pools/
//! Queries Kongswap for token prices using swap_amounts endpoint.
//! Returns how much ckUSDT you'd receive for 1 token.
//!
//! ### tvl/
//! Calculates total value locked across all kong_locker positions.
//! Queries each lock canister's balances from Kongswap and sums by token.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use crate::_3_KONG_LIQUIDITY;
//!
//! // Get Kong Locker TVL distribution
//! let tvl = _3_KONG_LIQUIDITY::tvl::calculate_kong_locker_tvl().await?;
//! // Returns: [(ALEX, $22500), (ZERO, $640), (KONG, $48), (BOB, $2)]
//!
//! // Get current token price
//! let alex_price = _3_KONG_LIQUIDITY::pools::get_token_price_in_usdt(&TrackedToken::ALEX).await?;
//! // Returns: 0.0012 (meaning 1 ALEX = 0.0012 ckUSDT)
//! ```

pub mod locker;
pub mod pools;
pub mod tvl;
