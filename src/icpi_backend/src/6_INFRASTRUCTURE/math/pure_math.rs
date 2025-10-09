//! Pure mathematical functions - no I/O, no async
//! All functions here must be deterministic and side-effect free

use candid::Nat;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use crate::infrastructure::errors::{Result, IcpiError, CalculationError, ValidationError, MintError, BurnError};

/// Multiply two Nats and divide by a third with arbitrary precision
/// Formula: (a × b) ÷ c
pub fn multiply_and_divide(a: &Nat, b: &Nat, c: &Nat) -> Result<Nat> {
    // Check for division by zero
    if c == &Nat::from(0u64) {
        return Err(IcpiError::Calculation(CalculationError::DivisionByZero {
            operation: format!("({} × {}) ÷ {}", a, b, c),
        }));
    }

    // Convert to BigUint for arbitrary precision
    let a_big = nat_to_biguint(a);
    let b_big = nat_to_biguint(b);
    let c_big = nat_to_biguint(c);

    // Perform calculation
    let result = (a_big * b_big) / c_big;

    // Convert back to Nat
    biguint_to_nat(result)
}

/// Convert between different decimal places
pub fn convert_decimals(
    amount: &Nat,
    from_decimals: u32,
    to_decimals: u32
) -> Result<Nat> {
    if from_decimals == to_decimals {
        return Ok(amount.clone());
    }

    if from_decimals < to_decimals {
        // Scale up: multiply by 10^(to - from)
        let multiplier = 10u64.pow(to_decimals - from_decimals);
        Ok(amount.clone() * Nat::from(multiplier))
    } else {
        // Scale down: divide by 10^(from - to)
        let divisor = Nat::from(10u64.pow(from_decimals - to_decimals));

        // Check if division would result in zero (precision loss)
        if amount < &divisor {
            return Err(IcpiError::Calculation(CalculationError::PrecisionLoss {
                operation: format!("convert_decimals({}, {}, {})", amount, from_decimals, to_decimals),
                original: amount.to_string(),
                result: "0".to_string(),
            }));
        }

        Ok(amount.clone() / divisor)
    }
}

/// Calculate ICPI tokens to mint based on deposit
///
/// # Formula
/// - Initial mint (supply = 0): amount adjusted for decimals
/// - Subsequent mints: (deposit × supply) ÷ tvl
pub fn calculate_mint_amount(
    deposit_amount: &Nat,
    current_supply: &Nat,
    current_tvl: &Nat,
) -> Result<Nat> {
    // Validate inputs
    if deposit_amount == &Nat::from(0u64) {
        return Err(IcpiError::Validation(ValidationError::InvalidAmount {
            amount: "0".to_string(),
            reason: "Deposit amount cannot be zero".to_string(),
        }));
    }

    // Initial mint case: 1:1 ratio adjusted for decimals
    if current_supply == &Nat::from(0u64) || current_tvl == &Nat::from(0u64) {
        // Convert ckUSDT (e6) to ICPI (e8)
        return convert_decimals(deposit_amount, 6, 8);
    }

    // Subsequent mints: proportional ownership
    // First convert deposit to e8 to match supply decimals
    let deposit_e8 = convert_decimals(deposit_amount, 6, 8)?;
    let tvl_e8 = convert_decimals(current_tvl, 6, 8)?;

    let icpi_amount = multiply_and_divide(&deposit_e8, current_supply, &tvl_e8)?;

    // Sanity check: must produce non-zero result
    if icpi_amount == Nat::from(0u64) {
        return Err(IcpiError::Mint(MintError::ProportionalCalculationError {
            reason: "Calculated mint amount is zero - deposit too small".to_string(),
        }));
    }

    Ok(icpi_amount)
}

/// Calculate redemption amounts for burning ICPI
pub fn calculate_redemptions(
    burn_amount: &Nat,
    total_supply: &Nat,
    token_balances: &[(String, Nat)],
) -> Result<Vec<(String, Nat)>> {
    // Validate inputs
    if burn_amount == &Nat::from(0u64) {
        return Err(IcpiError::Validation(ValidationError::InvalidAmount {
            amount: "0".to_string(),
            reason: "Burn amount cannot be zero".to_string(),
        }));
    }

    if total_supply == &Nat::from(0u64) {
        return Err(IcpiError::Burn(BurnError::NoRedemptionsPossible {
            reason: "Total supply is zero".to_string(),
        }));
    }

    if burn_amount > total_supply {
        return Err(IcpiError::Validation(ValidationError::InvalidAmount {
            amount: burn_amount.to_string(),
            reason: format!("Burn amount exceeds total supply {}", total_supply),
        }));
    }

    let mut redemptions = Vec::new();

    for (token_name, balance) in token_balances {
        if balance == &Nat::from(0u64) {
            continue;
        }

        let redemption_amount = multiply_and_divide(burn_amount, balance, total_supply)?;

        if redemption_amount > Nat::from(0u64) {
            redemptions.push((token_name.clone(), redemption_amount));
        }
    }

    if redemptions.is_empty() {
        return Err(IcpiError::Burn(BurnError::NoRedemptionsPossible {
            reason: "No tokens available for redemption".to_string(),
        }));
    }

    Ok(redemptions)
}

/// Calculate rebalancing trade size
pub fn calculate_trade_size(
    deviation_usd: f64,
    trade_intensity: f64,
    min_trade_size: f64,
) -> Result<Nat> {
    if deviation_usd <= 0.0 {
        return Ok(Nat::from(0u64));
    }

    let trade_size = deviation_usd * trade_intensity;

    if trade_size < min_trade_size {
        return Ok(Nat::from(0u64));
    }

    // Convert to e6 decimals (ckUSDT)
    let trade_size_e6 = (trade_size * 1_000_000.0) as u64;

    Ok(Nat::from(trade_size_e6))
}

// ===== Helper Functions =====

fn nat_to_biguint(nat: &Nat) -> BigUint {
    BigUint::from_bytes_be(&nat.0.to_bytes_be())
}

fn biguint_to_nat(big: BigUint) -> Result<Nat> {
    // BigUint already implements ToBigUint, so conversion should always succeed
    // But we handle the theoretical error case properly for production safety
    match num_bigint::ToBigUint::to_biguint(&big) {
        Some(biguint) => Ok(Nat::from(biguint)),
        None => Err(IcpiError::Calculation(CalculationError::Overflow {
            operation: format!("BigUint to Nat conversion failed for value: {}", big),
        }))
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiply_and_divide() {
        let a = Nat::from(100u64);
        let b = Nat::from(200u64);
        let c = Nat::from(50u64);

        let result = multiply_and_divide(&a, &b, &c).unwrap();
        assert_eq!(result, Nat::from(400u64));
    }

    #[test]
    fn test_convert_decimals_up() {
        let amount = Nat::from(1_000_000u64);
        let result = convert_decimals(&amount, 6, 8).unwrap();
        assert_eq!(result, Nat::from(100_000_000u64));
    }

    #[test]
    fn test_calculate_mint_initial() {
        let deposit = Nat::from(1_000_000u64);
        let supply = Nat::from(0u64);
        let tvl = Nat::from(0u64);

        let result = calculate_mint_amount(&deposit, &supply, &tvl).unwrap();
        assert_eq!(result, Nat::from(100_000_000u64));
    }

    #[test]
    fn test_calculate_redemptions() {
        let burn_amount = Nat::from(50_000_000u64);
        let total_supply = Nat::from(100_000_000u64);
        let balances = vec![
            ("ALEX".to_string(), Nat::from(1000u64)),
            ("KONG".to_string(), Nat::from(2000u64)),
        ];

        let result = calculate_redemptions(&burn_amount, &total_supply, &balances).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, Nat::from(500u64));
        assert_eq!(result[1].1, Nat::from(1000u64));
    }

    // === Phase 4: Comprehensive Math Tests ===

    #[test]
    fn test_division_by_zero() {
        let a = Nat::from(100u64);
        let b = Nat::from(200u64);
        let c = Nat::from(0u64);

        let result = multiply_and_divide(&a, &b, &c);
        assert!(result.is_err());
        assert!(matches!(result, Err(IcpiError::Calculation(CalculationError::DivisionByZero { .. }))));
    }

    #[test]
    fn test_large_value_multiplication() {
        // Test with large values that would overflow u64 but work with BigUint
        let a = Nat::from(u64::MAX);
        let b = Nat::from(2u64);
        let c = Nat::from(1u64);

        let result = multiply_and_divide(&a, &b, &c).unwrap();
        // Result should be 2 * u64::MAX which is larger than u64::MAX
        assert!(result > Nat::from(u64::MAX));
    }

    #[test]
    fn test_decimal_conversion_same_decimals() {
        let amount = Nat::from(1_000_000u64);
        let result = convert_decimals(&amount, 6, 6).unwrap();
        assert_eq!(result, amount); // Should return unchanged
    }

    #[test]
    fn test_decimal_conversion_precision_loss() {
        // Convert 1 (e6) to e8 and back - should preserve value
        let original = Nat::from(1_000_000u64); // 1.0 in e6
        let to_e8 = convert_decimals(&original, 6, 8).unwrap();
        let back_to_e6 = convert_decimals(&to_e8, 8, 6).unwrap();
        assert_eq!(back_to_e6, original);
    }

    #[test]
    fn test_decimal_conversion_down_below_minimum() {
        // Trying to convert 1 (raw value) from e6 to e8 by going down would cause precision loss
        let tiny_amount = Nat::from(1u64);
        let result = convert_decimals(&tiny_amount, 8, 6); // Scale down would lose precision

        // Should error due to precision loss
        assert!(result.is_err());
        assert!(matches!(result, Err(IcpiError::Calculation(CalculationError::PrecisionLoss { .. }))));
    }

    #[test]
    fn test_mint_amount_zero_deposit() {
        let result = calculate_mint_amount(
            &Nat::from(0u64),
            &Nat::from(1_000_000u64),
            &Nat::from(1_000_000u64)
        );

        assert!(result.is_err());
        assert!(matches!(result, Err(IcpiError::Validation(ValidationError::InvalidAmount { .. }))));
    }

    #[test]
    fn test_mint_amount_proportional_ownership() {
        // Supply: 100 ICPI (10,000,000,000 e8)
        // TVL: 100 ckUSDT (100,000,000 e6)
        // Deposit: 10 ckUSDT (10,000,000 e6) = 10%
        // Expected: 10 ICPI (1,000,000,000 e8)

        let deposit = Nat::from(10_000_000u64);
        let supply = Nat::from(10_000_000_000u64);
        let tvl = Nat::from(100_000_000u64);

        let result = calculate_mint_amount(&deposit, &supply, &tvl).unwrap();
        assert_eq!(result, Nat::from(1_000_000_000u64));
    }

    #[test]
    fn test_mint_amount_very_small_deposit() {
        // Test with a very small deposit that would round to zero
        // Supply: 1M ICPI, TVL: 1M ckUSDT
        // Deposit: 1 e6 (tiny)
        let deposit = Nat::from(1u64); // 0.000001 ckUSDT
        let supply = Nat::from(100_000_000_000_000u64); // 1M ICPI
        let tvl = Nat::from(1_000_000_000_000u64); // 1M ckUSDT

        let result = calculate_mint_amount(&deposit, &supply, &tvl);
        // Should fail because result would be zero
        assert!(result.is_err());
    }

    #[test]
    fn test_redemptions_burn_entire_supply() {
        // Burning entire supply should redeem all balances
        let burn_amount = Nat::from(100_000_000u64);
        let total_supply = Nat::from(100_000_000u64);
        let balances = vec![
            ("ALEX".to_string(), Nat::from(1000u64)),
            ("KONG".to_string(), Nat::from(2000u64)),
        ];

        let result = calculate_redemptions(&burn_amount, &total_supply, &balances).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, Nat::from(1000u64)); // All of ALEX
        assert_eq!(result[1].1, Nat::from(2000u64)); // All of KONG
    }

    #[test]
    fn test_redemptions_burn_more_than_supply() {
        // Burning more than supply should error
        let burn_amount = Nat::from(101_000_000u64);
        let total_supply = Nat::from(100_000_000u64);
        let balances = vec![
            ("ALEX".to_string(), Nat::from(1000u64)),
        ];

        let result = calculate_redemptions(&burn_amount, &total_supply, &balances);
        assert!(result.is_err());
        assert!(matches!(result, Err(IcpiError::Validation(ValidationError::InvalidAmount { .. }))));
    }

    #[test]
    fn test_redemptions_zero_burn() {
        let burn_amount = Nat::from(0u64);
        let total_supply = Nat::from(100_000_000u64);
        let balances = vec![
            ("ALEX".to_string(), Nat::from(1000u64)),
        ];

        let result = calculate_redemptions(&burn_amount, &total_supply, &balances);
        assert!(result.is_err());
        assert!(matches!(result, Err(IcpiError::Validation(ValidationError::InvalidAmount { .. }))));
    }

    #[test]
    fn test_redemptions_zero_supply() {
        let burn_amount = Nat::from(1_000_000u64);
        let total_supply = Nat::from(0u64);
        let balances = vec![
            ("ALEX".to_string(), Nat::from(1000u64)),
        ];

        let result = calculate_redemptions(&burn_amount, &total_supply, &balances);
        assert!(result.is_err());
        assert!(matches!(result, Err(IcpiError::Burn(BurnError::NoRedemptionsPossible { .. }))));
    }

    #[test]
    fn test_redemptions_no_balances() {
        let burn_amount = Nat::from(1_000_000u64);
        let total_supply = Nat::from(100_000_000u64);
        let balances: Vec<(String, Nat)> = vec![];

        let result = calculate_redemptions(&burn_amount, &total_supply, &balances);
        assert!(result.is_err());
        assert!(matches!(result, Err(IcpiError::Burn(BurnError::NoRedemptionsPossible { .. }))));
    }

    #[test]
    fn test_redemptions_skip_zero_balances() {
        let burn_amount = Nat::from(50_000_000u64);
        let total_supply = Nat::from(100_000_000u64);
        let balances = vec![
            ("ALEX".to_string(), Nat::from(1000u64)),
            ("KONG".to_string(), Nat::from(0u64)), // Zero balance
            ("ZERO".to_string(), Nat::from(2000u64)),
        ];

        let result = calculate_redemptions(&burn_amount, &total_supply, &balances).unwrap();
        // Should only include ALEX and ZERO
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "ALEX");
        assert_eq!(result[1].0, "ZERO");
    }

    #[test]
    fn test_trade_size_zero_deviation() {
        let result = calculate_trade_size(0.0, 0.1, 1.0).unwrap();
        assert_eq!(result, Nat::from(0u64));
    }

    #[test]
    fn test_trade_size_below_minimum() {
        // Deviation $5, intensity 10%, min $1 → trade $0.50 → returns 0
        let result = calculate_trade_size(5.0, 0.1, 1.0).unwrap();
        assert_eq!(result, Nat::from(0u64));
    }

    #[test]
    fn test_trade_size_above_minimum() {
        // Deviation $100, intensity 10%, min $1 → trade $10 → returns 10_000_000 e6
        let result = calculate_trade_size(100.0, 0.1, 1.0).unwrap();
        assert_eq!(result, Nat::from(10_000_000u64)); // $10 in e6
    }

    #[test]
    fn test_multiply_and_divide_maintains_precision() {
        // Test that multiply_and_divide doesn't lose precision unnecessarily
        // (3 * 7) / 2 = 21 / 2 = 10 (integer division, expected)
        let result = multiply_and_divide(&Nat::from(3u64), &Nat::from(7u64), &Nat::from(2u64)).unwrap();
        assert_eq!(result, Nat::from(10u64));
    }

    #[test]
    fn test_nat_biguint_roundtrip() {
        // Test conversion helpers
        let original = Nat::from(123456789u64);
        let big = nat_to_biguint(&original);
        let back = biguint_to_nat(big).unwrap();
        assert_eq!(back, original);
    }
}
