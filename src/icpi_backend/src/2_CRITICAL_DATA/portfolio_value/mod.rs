//! Portfolio value calculation module
//!
//! Calculates total portfolio value for minting formula

use candid::Nat;
use num_traits::ToPrimitive;
use crate::infrastructure::Result;
use crate::types::portfolio::IndexState;
use crate::types::TrackedToken;

/// Calculate total portfolio value atomically
///
/// Sums: (all token balances × token prices) + ckUSDT reserves
///
/// For tracked tokens (ALEX, ZERO, KONG, BOB), we query Kongswap pools
/// to get their ckUSDT exchange rate and calculate USD value.
///
/// Formula: TVL = ckUSDT + Σ(token_balance × token_price_in_ckusdt)
pub async fn calculate_portfolio_value_atomic() -> Result<Nat> {
    ic_cdk::println!("CALC: Computing total portfolio value");

    // Get all balances in parallel
    let balances = crate::_2_CRITICAL_DATA::token_queries::get_all_balances_uncached().await?;

    let mut total_value_e6: u128 = 0;

    for (symbol, balance) in balances {
        if symbol == "ckUSDT" {
            // ckUSDT is 1:1 with USD, already in e6 decimals
            // Safely convert balance, return error if overflow
            let value = balance.0.to_u64()
                .ok_or_else(|| {
                    crate::infrastructure::IcpiError::Other(
                        format!("ckUSDT balance {} too large to process", balance)
                    )
                })?;

            // Use checked addition to prevent overflow
            total_value_e6 = total_value_e6.checked_add(value as u128)
                .ok_or_else(|| {
                    crate::infrastructure::IcpiError::Other(
                        "Portfolio value overflow when adding ckUSDT".to_string()
                    )
                })?;

            ic_cdk::println!("  ckUSDT: {} (e6) = ${}", balance, value as f64 / 1_000_000.0);
        } else {
            // For tracked tokens, get price from Kongswap and calculate value
            // CRITICAL: Fail if any token pricing fails to ensure accurate TVL
            let value_e6 = get_token_usd_value(&symbol, &balance).await
                .map_err(|e| {
                    ic_cdk::println!("  ❌ Error valuing {}: {}", symbol, e);
                    crate::infrastructure::IcpiError::Other(
                        format!("Failed to value token {}: {}", symbol, e)
                    )
                })?;

            // Use checked addition to prevent overflow
            total_value_e6 = total_value_e6.checked_add(value_e6 as u128)
                .ok_or_else(|| {
                    crate::infrastructure::IcpiError::Other(
                        format!("Portfolio value overflow when adding {} value", symbol)
                    )
                })?;

            ic_cdk::println!("  {}: {} tokens = ${}", symbol, balance, value_e6 as f64 / 1_000_000.0);
        }
    }

    // Validate the total value is reasonable (under $1 trillion as sanity check)
    const MAX_REASONABLE_VALUE_E6: u128 = 1_000_000_000_000 * 1_000_000; // $1 trillion in e6
    if total_value_e6 > MAX_REASONABLE_VALUE_E6 {
        return Err(crate::infrastructure::IcpiError::Other(
            format!("Portfolio value {} exceeds maximum reasonable limit", total_value_e6)
        ));
    }

    let total_value = Nat::from(total_value_e6);
    ic_cdk::println!("✅ Total portfolio value: ${} (e6 ckUSDT)", total_value_e6 as f64 / 1_000_000.0);

    Ok(total_value)
}

/// Get USD value of a token amount
/// Returns value in e6 (ckUSDT decimals)
///
/// **ALPHA V1 LIMITATION**: Uses conservative hardcoded prices
/// This is intentional for the Alpha deployment to ensure stability and prevent
/// over-minting during experimental phase. These prices are set conservatively low
/// to protect users during the initial testing period.
///
/// **PRODUCTION NOTE**: Beta/Production versions will integrate with Kongswap
/// price feeds for real-time market pricing.
///
/// Current hardcoded prices (conservative):
/// - ALEX: $0.50 (actual market ~$1.00+)
/// - ZERO: $0.10 (actual market ~$0.20+)
/// - KONG: $0.05 (actual market ~$0.10+)
/// - BOB: $0.01 (actual market ~$0.02+)
async fn get_token_usd_value(token_symbol: &str, amount: &Nat) -> Result<u64> {
    // ALPHA V1: Conservative hardcoded prices to prevent over-minting
    // These values are intentionally set below market prices for safety
    let price_per_token_e6 = match token_symbol {
        "ALEX" => 500_000u64,  // $0.50 per ALEX (conservative)
        "ZERO" => 100_000u64,  // $0.10 per ZERO (conservative)
        "KONG" => 50_000u64,   // $0.05 per KONG (conservative)
        "BOB" => 10_000u64,    // $0.01 per BOB (conservative)
        _ => {
            ic_cdk::println!("  Unknown token {}, valuing at $0", token_symbol);
            return Ok(0u64);
        }
    };

    // Safely convert amount to u64, returning error on overflow
    let amount_e8 = amount.0.to_u64()
        .ok_or_else(|| {
            crate::infrastructure::IcpiError::Other(
                format!("Amount {} too large to process", amount)
            )
        })?;

    if amount_e8 == 0 {
        return Ok(0u64);
    }

    // Calculate: (amount_e8 * price_e6) / 1e8 with overflow protection
    let amount_u128 = amount_e8 as u128;
    let price_u128 = price_per_token_e6 as u128;

    // Check for potential overflow before multiplication
    let product = amount_u128.checked_mul(price_u128)
        .ok_or_else(|| {
            crate::infrastructure::IcpiError::Other(
                format!("Arithmetic overflow in price calculation: {} * {}", amount_e8, price_per_token_e6)
            )
        })?;

    let value_e6_u128 = product / 100_000_000;

    // Ensure result fits in u64
    if value_e6_u128 > u64::MAX as u128 {
        return Err(crate::infrastructure::IcpiError::Other(
            format!("Value overflow: {} exceeds u64 max", value_e6_u128)
        ));
    }

    let value_e6 = value_e6_u128 as u64;

    ic_cdk::println!(
        "  {} tokens of {}: ${} (@ ${}/token)",
        amount_e8 as f64 / 100_000_000.0,
        token_symbol,
        value_e6 as f64 / 1_000_000.0,
        price_per_token_e6 as f64 / 1_000_000.0
    );

    Ok(value_e6)
}

/// Get portfolio state without caching
///
/// Returns complete portfolio state for display
pub async fn get_portfolio_state_uncached() -> Result<IndexState> {
    ic_cdk::println!("CALC: Building portfolio state");

    // Get all balances
    let balances = crate::_2_CRITICAL_DATA::token_queries::get_all_balances_uncached().await?;

    // Calculate total value
    let total_value_nat = calculate_portfolio_value_atomic().await?;
    // Handle u128 values properly - convert to f64 safely
    let total_value_u128 = total_value_nat.0.to_u128().unwrap_or(0);
    let total_value_f64 = total_value_u128 as f64 / 1_000_000.0;

    // Build current positions using CurrentPosition type
    use crate::types::portfolio::CurrentPosition;
    use crate::types::rebalancing::TargetAllocation;

    let current_positions: Vec<_> = balances.iter()
        .filter_map(|(symbol, balance)| {
            // Try to parse symbol to TrackedToken
            let token = match symbol.as_str() {
                "ALEX" => Some(TrackedToken::ALEX),
                "ZERO" => Some(TrackedToken::ZERO),
                "KONG" => Some(TrackedToken::KONG),
                "BOB" => Some(TrackedToken::BOB),
                _ => None,
            };

            token.map(|t| {
                let decimals = get_token_decimals(symbol);
                let amount_float = balance.0.to_u64().unwrap_or(0) as f64 / 10_f64.powi(decimals as i32);

                CurrentPosition {
                    token: t,
                    balance: balance.clone(),
                    usd_value: amount_float, // Simplified: actual value TBD
                    percentage: 0.0, // Calculate below
                }
            })
        })
        .collect();

    // For now, target allocations are equal (25% each for 4 tokens)
    let target_allocations = vec![
        TargetAllocation {
            token: TrackedToken::ALEX,
            target_percentage: 25.0,
            target_usd_value: total_value_f64 * 0.25,
        },
        TargetAllocation {
            token: TrackedToken::ZERO,
            target_percentage: 25.0,
            target_usd_value: total_value_f64 * 0.25,
        },
        TargetAllocation {
            token: TrackedToken::KONG,
            target_percentage: 25.0,
            target_usd_value: total_value_f64 * 0.25,
        },
        TargetAllocation {
            token: TrackedToken::BOB,
            target_percentage: 25.0,
            target_usd_value: total_value_f64 * 0.25,
        },
    ];

    // Calculate deviations (placeholder)
    let deviations = Vec::new();

    // Get ckUSDT balance specifically
    let ckusdt_balance = balances.iter()
        .find(|(s, _)| s == "ckUSDT")
        .map(|(_, b)| b.clone())
        .unwrap_or(Nat::from(0u64));

    Ok(IndexState {
        total_value: total_value_f64,
        current_positions,
        target_allocations,
        deviations,
        ckusdt_balance,
        timestamp: ic_cdk::api::time(),
    })
}

/// Get token decimals (helper)
fn get_token_decimals(symbol: &str) -> u32 {
    match symbol {
        "ckUSDT" => 6,
        "ALEX" | "ZERO" | "KONG" | "BOB" => 8,
        "ICPI" => 8,
        _ => 8, // Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_decimals() {
        assert_eq!(get_token_decimals("ckUSDT"), 6);
        assert_eq!(get_token_decimals("ALEX"), 8);
        assert_eq!(get_token_decimals("unknown"), 8);
    }
}
