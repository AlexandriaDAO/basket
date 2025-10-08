//! Calculate redemption amounts for burn operations
//! Pure functions for proportional distribution

use candid::Nat;
use crate::infrastructure::{Result, IcpiError, CalculationError};
use crate::types::TrackedToken;

/// Calculate redemption amounts for all tokens based on burn amount and current supply
pub async fn calculate_redemptions(
    burn_amount: &Nat,
    current_supply: &Nat,
) -> Result<Vec<(String, Nat)>> {
    let mut redemptions = Vec::new();

    // Get all token balances
    let balances = crate::_2_CRITICAL_DATA::token_queries::get_all_balances_uncached().await?;

    // Calculate proportional redemption for each token
    for (token_symbol, balance) in balances {
        if balance > Nat::from(0u32) {
            // Calculate: (burn_amount * balance) / current_supply
            let redemption_amount = match crate::infrastructure::math::multiply_and_divide(
                burn_amount,
                &balance,
                current_supply
            ) {
                Ok(amount) => amount,
                Err(e) => {
                    ic_cdk::println!("Warning: Calculation failed for {}: {}", token_symbol, e);
                    continue;
                }
            };

            // Check if amount is above dust threshold (transfer fee + buffer)
            const TRANSFER_FEE: u32 = 10_000; // Standard ICRC-1 fee
            const MIN_BUFFER: u32 = 1_000;    // Small buffer above fee

            if redemption_amount > Nat::from(TRANSFER_FEE + MIN_BUFFER) {
                let amount_after_fee = redemption_amount - Nat::from(TRANSFER_FEE);
                redemptions.push((token_symbol, amount_after_fee));
            } else {
                ic_cdk::println!("Skipping {} redemption: {} below dust threshold",
                    token_symbol, redemption_amount);
            }
        }
    }

    if redemptions.is_empty() {
        return Err(IcpiError::Burn(crate::infrastructure::BurnError::NoRedemptionsPossible {
            reason: "No tokens available for redemption or all amounts below dust threshold".to_string(),
        }));
    }

    ic_cdk::println!("Calculated {} token redemptions for burn", redemptions.len());
    Ok(redemptions)
}

/// Calculate proportional share for a single token (pure function)
pub fn calculate_proportional_share(
    burn_amount: &Nat,
    token_balance: &Nat,
    total_supply: &Nat,
) -> Result<Nat> {
    if total_supply == &Nat::from(0u32) {
        return Err(IcpiError::Calculation(CalculationError::DivisionByZero {
            operation: "proportional_share".to_string(),
        }));
    }

    crate::infrastructure::math::multiply_and_divide(burn_amount, token_balance, total_supply)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proportional_share_calculation() {
        let burn_amount = Nat::from(100_000_000u64); // 1 ICPI (e8)
        let token_balance = Nat::from(500_000_000u64); // 500 tokens
        let total_supply = Nat::from(1_000_000_000u64); // 10 ICPI total

        let result = calculate_proportional_share(&burn_amount, &token_balance, &total_supply).unwrap();

        // Should get 10% of token balance (50 tokens)
        assert_eq!(result, Nat::from(50_000_000u64));
    }

    #[test]
    fn test_division_by_zero() {
        let burn_amount = Nat::from(100u64);
        let token_balance = Nat::from(1000u64);
        let total_supply = Nat::from(0u64);

        let result = calculate_proportional_share(&burn_amount, &token_balance, &total_supply);
        assert!(result.is_err());
    }
}