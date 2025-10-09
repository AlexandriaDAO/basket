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
/// Uses Nat (BigUint) arithmetic directly - no artificial u128 ceiling.
///
/// # Algorithm
/// Check if: amount * 100 > supply * 10
/// This is equivalent to: amount / supply > 0.10
/// But uses integer arithmetic to avoid floating point precision loss.
///
/// # Arguments
/// * `amount` - The amount of ICPI tokens to burn
/// * `supply` - The current total ICPI supply
///
/// # Returns
/// * `Ok(())` if amount is within the 10% limit
/// * `Err(IcpiError::Burn(AmountExceedsMaximum))` if amount exceeds limit
pub fn validate_burn_limit(amount: &Nat, supply: &Nat) -> Result<()> {
    // Use Nat arithmetic directly (supports BigUint, no u128 ceiling)
    // Check if: amount * 100 > supply * 10
    let amount_scaled = amount.clone() * Nat::from(100u64);
    let supply_scaled = supply.clone() * Nat::from(10u64);

    if amount_scaled > supply_scaled {
        // Calculate maximum allowed burn: (supply * 10) / 100
        let maximum_burn = supply.clone() * Nat::from(10u64) / Nat::from(100u64);

        return Err(IcpiError::Burn(BurnError::AmountExceedsMaximum {
            amount: amount.to_string(),
            maximum: maximum_burn.to_string(),
            percentage_limit: "10%".to_string(),
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