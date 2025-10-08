//! Data validation module

use candid::Nat;
use crate::infrastructure::{Result, IcpiError, ValidationError};

/// Validate external supply data
pub fn validate_supply(new_supply: &Nat, cached_supply: Option<&Nat>) -> Result<()> {
    const MAX_POSSIBLE_SUPPLY: u128 = 1_000_000_000_000_000_000; // 10 billion ICPI

    if new_supply > &Nat::from(MAX_POSSIBLE_SUPPLY) {
        return Err(IcpiError::Validation(ValidationError::SupplyOutOfBounds {
            supply: new_supply.to_string(),
            max: MAX_POSSIBLE_SUPPLY.to_string(),
        }));
    }

    Ok(())
}

/// Validate token price
pub fn validate_price(
    token: &str,
    price: f64,
    cached_price: Option<f64>,
) -> Result<()> {
    const MIN_REASONABLE_PRICE: f64 = 0.0001;
    const MAX_REASONABLE_PRICE: f64 = 1_000_000.0;

    if price < MIN_REASONABLE_PRICE || price > MAX_REASONABLE_PRICE {
        return Err(IcpiError::Validation(ValidationError::PriceOutOfBounds {
            price: price.to_string(),
            min: MIN_REASONABLE_PRICE.to_string(),
            max: MAX_REASONABLE_PRICE.to_string(),
        }));
    }

    Ok(())
}
