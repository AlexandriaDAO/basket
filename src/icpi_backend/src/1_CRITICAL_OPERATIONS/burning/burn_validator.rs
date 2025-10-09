//! Validation for burn operations

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, ValidationError, BurnError};
use crate::infrastructure::constants::MIN_BURN_AMOUNT;

pub fn validate_burn_request(caller: &Principal, amount: &Nat) -> Result<()> {
    // Check principal
    if caller == &Principal::anonymous() {
        return Err(IcpiError::Validation(ValidationError::InvalidPrincipal {
            principal: caller.to_text(),
        }));
    }

    // Check minimum amount
    if amount < &Nat::from(MIN_BURN_AMOUNT) {
        return Err(IcpiError::Burn(BurnError::AmountBelowMinimum {
            amount: amount.to_string(),
            minimum: MIN_BURN_AMOUNT.to_string(),
        }));
    }

    // Rate limiting
    crate::infrastructure::rate_limiting::check_rate_limit(
        &format!("burn_{}", caller),
        1_000_000_000 // 1 second
    )?;

    Ok(())
}

/// Validates burn amount does not exceed maximum (10% of supply)
///
/// This function is extracted to be testable and reusable.
/// Uses pure integer arithmetic to avoid floating point precision loss.
///
/// # Arguments
/// * `amount` - The amount of ICPI tokens to burn
/// * `supply` - The current total ICPI supply
///
/// # Returns
/// * `Ok(())` if amount is within the 10% limit
/// * `Err(IcpiError::Burn(AmountExceedsMaximum))` if amount exceeds limit
pub fn validate_burn_limit(amount: &Nat, supply: &Nat) -> Result<()> {
    const MAX_BURN_PERCENTAGE_NUMERATOR: u128 = 10; // 10%
    const PERCENTAGE_DENOMINATOR: u128 = 100;

    // Convert to u128 for safe calculation
    use num_traits::ToPrimitive;
    let supply_u128 = supply.0.to_u128()
        .ok_or_else(|| IcpiError::Other("Supply too large to process".to_string()))?;
    let amount_u128 = amount.0.to_u128()
        .ok_or_else(|| IcpiError::Other("Amount too large to process".to_string()))?;

    // Integer arithmetic: Check if (amount * 100 > supply * 10)
    // This is equivalent to (amount / supply > 0.10) but avoids floating point
    // Using checked_mul to prevent overflow
    let amount_scaled = amount_u128.checked_mul(PERCENTAGE_DENOMINATOR)
        .ok_or_else(|| IcpiError::Other("Burn amount too large for calculation".to_string()))?;
    let supply_scaled = supply_u128.checked_mul(MAX_BURN_PERCENTAGE_NUMERATOR)
        .ok_or_else(|| IcpiError::Other("Supply too large for calculation".to_string()))?;

    if amount_scaled > supply_scaled {
        // Calculate maximum allowed burn using integer arithmetic
        // max_burn = (supply * 10) / 100
        let maximum_burn = supply_u128
            .checked_mul(MAX_BURN_PERCENTAGE_NUMERATOR)
            .and_then(|v| v.checked_div(PERCENTAGE_DENOMINATOR))
            .ok_or_else(|| IcpiError::Other("Maximum burn calculation overflow".to_string()))?;

        return Err(IcpiError::Burn(BurnError::AmountExceedsMaximum {
            amount: amount.to_string(),
            maximum: maximum_burn.to_string(),
            percentage_limit: format!("{}%", MAX_BURN_PERCENTAGE_NUMERATOR),
        }));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anonymous_principal_rejected() {
        let result = validate_burn_request(&Principal::anonymous(), &Nat::from(MIN_BURN_AMOUNT));
        assert!(matches!(result, Err(IcpiError::Validation(_))));
    }

    #[test]
    fn test_min_burn_amount() {
        // Below minimum should fail
        let principal = Principal::from_text("2vxsx-fae").unwrap();
        let result = validate_burn_request(&principal, &Nat::from(1u32));
        assert!(matches!(result, Err(IcpiError::Burn(BurnError::AmountBelowMinimum { .. }))));

        // Exactly at minimum should pass validation (rate limit may fail in repeat calls)
        let result = validate_burn_request(&principal, &Nat::from(MIN_BURN_AMOUNT));
        // Note: May fail due to rate limiting in test environment, but should pass validation check
        match result {
            Ok(_) => {}, // Passed validation
            Err(IcpiError::Other(msg)) if msg.contains("Rate limit") => {}, // Failed rate limit, but validation passed
            Err(e) => panic!("Expected validation to pass or rate limit error, got: {:?}", e),
        }
    }

    #[test]
    fn test_valid_burn_request_structure() {
        let principal = Principal::from_text("aaaaa-aa").unwrap();
        let amount = Nat::from(1_000_000u64); // Well above minimum

        // First call should pass validation (may hit rate limit on repeat)
        let result = validate_burn_request(&principal, &amount);
        match result {
            Ok(_) => {},
            Err(IcpiError::Other(msg)) if msg.contains("Rate limit") => {},
            Err(e) => panic!("Unexpected validation error: {:?}", e),
        }
    }
}