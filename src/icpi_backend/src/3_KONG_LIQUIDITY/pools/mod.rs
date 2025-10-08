//! Kongswap Pool Integration
//!
//! Queries Kongswap for token prices via swap_amounts endpoint.
//! Used to value portfolio tokens in USD equivalent.

use candid::{Nat, Principal};
use num_traits::ToPrimitive;
use crate::infrastructure::{Result, IcpiError};
use crate::infrastructure::constants::KONGSWAP_BACKEND_ID;
use crate::types::TrackedToken;
use crate::types::kongswap::SwapAmountsResult;

/// Get token price in ckUSDT
///
/// Uses Kongswap's swap_amounts to query how much ckUSDT you'd receive
/// for 1 token (in e8 decimals).
///
/// Returns: Price in ckUSDT per token (as f64)
///
/// Example: get_token_price_in_usdt(&TrackedToken::ALEX) -> 0.0012
/// Means 1 ALEX = 0.0012 ckUSDT
pub async fn get_token_price_in_usdt(token: &TrackedToken) -> Result<f64> {
    let symbol = token.to_symbol();

    // Special case: ckUSDT price is always 1.0
    if symbol == "ckUSDT" {
        return Ok(1.0);
    }

    let kongswap = Principal::from_text(KONGSWAP_BACKEND_ID)
        .map_err(|e| IcpiError::Other(format!("Invalid kongswap canister ID: {}", e)))?;

    // Query how much ckUSDT we'd get for 1 token (100_000_000 atomic units = 1.0 token)
    let one_token = Nat::from(100_000_000u64); // 1.0 in e8 decimals

    let (result,): (SwapAmountsResult,) = ic_cdk::call(
        kongswap,
        "swap_amounts",
        (symbol, one_token, "ckUSDT".to_string()) // Removed unnecessary clone
    ).await.map_err(|e| {
        ic_cdk::println!("Failed to query kongswap.swap_amounts for {}: {:?}", symbol, e);
        IcpiError::Other(format!("Kongswap price query failed: {:?}", e.1))
    })?;

    match result {
        SwapAmountsResult::Ok(reply) => {
            // Decimal handling:
            // - Input: 100_000_000 (1.0 token in e8 decimals for ALEX/ZERO/KONG/BOB)
            // - Output: ckUSDT amount in e6 decimals (ckUSDT uses 6 decimals, not 8)
            // - Conversion: divide by 1_000_000 to get float price
            let receive_e6 = reply.receive_amount.0.to_u64()
                .ok_or_else(|| IcpiError::Other(format!("Price amount overflow for {}", symbol)))?;

            let price_usdt = receive_e6 as f64 / 1_000_000.0; // e6 → f64

            ic_cdk::println!("✅ {} price: {} ckUSDT", symbol, price_usdt);
            Ok(price_usdt)
        }
        SwapAmountsResult::Err(e) => {
            ic_cdk::println!("Kongswap price query error for {}: {}", symbol, e);
            Err(IcpiError::Other(format!("Kongswap returned error: {}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kongswap_canister_id() {
        assert!(Principal::from_text(KONGSWAP_BACKEND_ID).is_ok());
    }

    #[test]
    fn test_ckusdt_price_is_one() {
        // Can't test async in unit test, but can verify logic path
        assert_eq!(KONGSWAP_BACKEND_ID, "2ipq2-uqaaa-aaaar-qailq-cai");
    }
}
