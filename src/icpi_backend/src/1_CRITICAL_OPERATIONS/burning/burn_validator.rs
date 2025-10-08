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