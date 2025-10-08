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