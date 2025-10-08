//! # Slippage Protection Module
//!
//! Calculates slippage limits and validates swap results
//! to ensure trades execute within acceptable price ranges.
//!
//! ## Key Functions
//! - `calculate_min_receive`: Get minimum acceptable output amount
//! - `validate_swap_result`: Verify actual slippage is within limits
//!
//! ## Safety Checks
//! - Positive slippage (getting more than expected) is allowed
//! - Zero expected amounts are rejected
//! - Actual slippage must not exceed max_slippage parameter

use candid::Nat;
use crate::infrastructure::{Result, IcpiError, errors::TradingError};
use num_traits::ToPrimitive;

/// Calculate minimum acceptable receive amount based on slippage tolerance
///
/// ## Example
/// - Expected: 100 tokens
/// - Max slippage: 0.02 (2%)
/// - Result: 98 tokens (will accept down to 98)
///
/// ## Edge Cases
/// - Returns 0 if expected amount is 0 (will be caught by validation)
/// - Handles Nat conversion safely
pub fn calculate_min_receive(
    expected_amount: &Nat,
    max_slippage: f64,
) -> Nat {
    let expected_f64 = expected_amount.0.to_u64().unwrap_or(0) as f64;
    let min_f64 = expected_f64 * (1.0 - max_slippage);
    Nat::from(min_f64 as u64)
}

/// Validate that swap result meets slippage requirements
///
/// ## Parameters
/// - `expected`: Amount we expected to receive based on pre-swap query
/// - `actual`: Amount actually received from the swap
/// - `max_slippage`: Maximum allowed slippage (e.g., 0.02 = 2%)
///
/// ## Returns
/// - `Ok(())` if slippage is acceptable
/// - `Err(TradingError::SlippageExceeded)` if slippage too high
///
/// ## Examples
/// ```
/// // Good: 2% slippage with 2% max
/// validate_swap_result(&Nat::from(100), &Nat::from(98), 0.02)?; // OK
///
/// // Good: Positive slippage (got more than expected)
/// validate_swap_result(&Nat::from(100), &Nat::from(102), 0.02)?; // OK
///
/// // Bad: 5% slippage with 2% max
/// validate_swap_result(&Nat::from(100), &Nat::from(95), 0.02)?; // Error
/// ```
pub fn validate_swap_result(
    expected: &Nat,
    actual: &Nat,
    max_slippage: f64,
) -> Result<()> {
    let expected_f64 = expected.0.to_u64().unwrap_or(0) as f64;
    let actual_f64 = actual.0.to_u64().unwrap_or(0) as f64;

    // Zero expected amount is invalid
    if expected_f64 == 0.0 {
        return Err(IcpiError::Trading(TradingError::InvalidSwapAmount {
            reason: "Expected amount cannot be zero".to_string(),
        }));
    }

    // Calculate actual slippage
    let actual_slippage = (expected_f64 - actual_f64) / expected_f64;

    // Positive slippage (got more than expected) is always good
    if actual_slippage < 0.0 {
        ic_cdk::println!("✅ Positive slippage: expected {}, got {} ({}% better)",
            expected_f64, actual_f64, (-actual_slippage * 100.0));
        return Ok(());
    }

    // Check if slippage exceeds maximum
    if actual_slippage > max_slippage {
        return Err(IcpiError::Trading(TradingError::SlippageExceeded {
            expected: expected.clone(),
            actual: actual.clone(),
            max_allowed: max_slippage,
            actual_slippage,
        }));
    }

    ic_cdk::println!("✅ Slippage acceptable: {:.2}% (max: {:.2}%)",
        actual_slippage * 100.0, max_slippage * 100.0);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_min_receive() {
        // 2% slippage on 100 tokens = 98 minimum
        let expected = Nat::from(100u64);
        let min = calculate_min_receive(&expected, 0.02);
        assert_eq!(min, Nat::from(98u64));

        // 5% slippage on 1000 tokens = 950 minimum
        let expected = Nat::from(1000u64);
        let min = calculate_min_receive(&expected, 0.05);
        assert_eq!(min, Nat::from(950u64));
    }

    #[test]
    fn test_validate_swap_result_within_limit() {
        // 2% slippage with 2% max should pass
        let expected = Nat::from(100u64);
        let actual = Nat::from(98u64);
        assert!(validate_swap_result(&expected, &actual, 0.02).is_ok());
    }

    #[test]
    fn test_validate_swap_result_positive_slippage() {
        // Got more than expected should always pass
        let expected = Nat::from(100u64);
        let actual = Nat::from(105u64);
        assert!(validate_swap_result(&expected, &actual, 0.02).is_ok());
    }

    #[test]
    fn test_validate_swap_result_exceeds_limit() {
        // 5% slippage with 2% max should fail
        let expected = Nat::from(100u64);
        let actual = Nat::from(95u64);
        assert!(validate_swap_result(&expected, &actual, 0.02).is_err());
    }

    #[test]
    fn test_validate_swap_result_zero_expected() {
        // Zero expected amount should fail
        let expected = Nat::from(0u64);
        let actual = Nat::from(100u64);
        assert!(validate_swap_result(&expected, &actual, 0.02).is_err());
    }
}
