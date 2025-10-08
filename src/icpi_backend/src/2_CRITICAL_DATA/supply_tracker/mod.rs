//! Supply tracking module

use candid::{Nat, Principal};
use num_traits::ToPrimitive;
use crate::infrastructure::{Result, IcpiError};
use crate::infrastructure::constants::ICPI_CANISTER_ID;
use crate::infrastructure::errors::{QueryError, ValidationError};

/// Get ICPI supply without caching
///
/// Queries the ICPI ledger canister for the current total supply using ICRC-1 standard.
/// This function MUST NOT cache results for financial accuracy.
pub async fn get_icpi_supply_uncached() -> Result<Nat> {
    // Assert no caching for critical operation
    ic_cdk::println!("CRITICAL: Querying ICPI supply (uncached)");

    // Parse ICPI canister principal
    let icpi_canister = Principal::from_text(ICPI_CANISTER_ID)
        .map_err(|e| IcpiError::Query(QueryError::CanisterUnreachable {
            canister: ICPI_CANISTER_ID.to_string(),
            reason: format!("Invalid principal: {}", e),
        }))?;

    // ICRC-1 total_supply call - no arguments
    let result: std::result::Result<(Nat,), _> = ic_cdk::call(
        icpi_canister,
        "icrc1_total_supply",
        ()
    ).await;

    match result {
        Ok((supply,)) => {
            // Validate supply is reasonable
            validate_supply(&supply)?;

            ic_cdk::println!("✅ ICPI total supply: {}", supply);
            Ok(supply)
        }
        Err((code, msg)) => {
            ic_cdk::println!("❌ Supply query failed: {:?} - {}", code, msg);
            Err(IcpiError::Query(QueryError::CanisterUnreachable {
                canister: ICPI_CANISTER_ID.to_string(),
                reason: format!("Call failed: {:?} - {}", code, msg),
            }))
        }
    }
}

/// Validate supply is within reasonable bounds
fn validate_supply(supply: &Nat) -> Result<()> {
    // Maximum possible supply: 100 million ICPI with 8 decimals
    const MAX_POSSIBLE_SUPPLY: u128 = 100_000_000 * 100_000_000; // 10^16

    // Convert to u128 for comparison
    let supply_u128 = supply.0.to_u128()
        .ok_or_else(|| IcpiError::Validation(ValidationError::SupplyOutOfBounds {
            supply: supply.to_string(),
            max: MAX_POSSIBLE_SUPPLY.to_string(),
        }))?;

    if supply_u128 > MAX_POSSIBLE_SUPPLY {
        return Err(IcpiError::Validation(ValidationError::SupplyOutOfBounds {
            supply: supply.to_string(),
            max: MAX_POSSIBLE_SUPPLY.to_string(),
        }));
    }

    Ok(())
}

/// Get validated supply (convenience wrapper)
pub async fn get_validated_supply() -> Result<Nat> {
    get_icpi_supply_uncached().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_supply_normal() {
        let supply = Nat::from(100_000_000u64); // 1 ICPI
        assert!(validate_supply(&supply).is_ok());
    }

    #[test]
    fn test_validate_supply_large() {
        let supply = Nat::from(10_000_000_000_000_000u64); // 100M ICPI
        assert!(validate_supply(&supply).is_ok());
    }

    #[test]
    fn test_validate_supply_too_large() {
        let supply = Nat::from(100_000_000_000_000_001u64); // > max
        assert!(validate_supply(&supply).is_err());
    }
}
