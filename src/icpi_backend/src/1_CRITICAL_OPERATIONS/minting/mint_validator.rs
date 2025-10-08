//! Validation for mint operations

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, ValidationError, MintError};
use crate::infrastructure::constants::{MIN_MINT_AMOUNT, MAX_MINT_AMOUNT};

pub fn validate_mint_request(caller: &Principal, amount: &Nat) -> Result<()> {
    // Check principal is not anonymous
    if caller == &Principal::anonymous() {
        return Err(IcpiError::Validation(ValidationError::InvalidPrincipal {
            principal: caller.to_text(),
        }));
    }

    // Check amount bounds
    if amount < &Nat::from(MIN_MINT_AMOUNT) {
        return Err(IcpiError::Mint(MintError::AmountBelowMinimum {
            amount: amount.to_string(),
            minimum: MIN_MINT_AMOUNT.to_string(),
        }));
    }

    if amount > &Nat::from(MAX_MINT_AMOUNT) {
        return Err(IcpiError::Mint(MintError::AmountAboveMaximum {
            amount: amount.to_string(),
            maximum: MAX_MINT_AMOUNT.to_string(),
        }));
    }

    // Rate limiting check
    crate::infrastructure::rate_limiting::check_rate_limit(
        &format!("mint_{}", caller),
        1_000_000_000 // 1 second
    )?;

    Ok(())
}