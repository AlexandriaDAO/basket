//! TVL Calculation from Kong Locker
//!
//! Calculates total value locked across all kong_locker positions for tracked tokens.
//! Used to determine target portfolio allocations.

use candid::Principal;
use crate::infrastructure::{Result, IcpiError};
use crate::types::TrackedToken;
use crate::types::kongswap::{UserBalancesResult, UserBalancesReply};

/// Kongswap backend canister ID (for querying user balances)
const KONGSWAP_CANISTER: &str = "2ipq2-uqaaa-aaaar-qailq-cai";

/// Calculate TVL from Kong Locker positions
///
/// Returns: Vec<(TrackedToken, usd_value)>
///
/// Process:
/// 1. Get all lock canisters from kong_locker
/// 2. For each lock canister, query Kongswap for user_balances
/// 3. Extract LP balances for tracked tokens (ALEX, ZERO, KONG, BOB)
/// 4. Sum USD values across all users
///
/// Example output: [(ALEX, 22500.0), (ZERO, 640.0), (KONG, 48.0), (BOB, 2.0)]
pub async fn calculate_kong_locker_tvl() -> Result<Vec<(TrackedToken, f64)>> {
    ic_cdk::println!("📊 Calculating Kong Locker TVL...");

    // Get all lock canisters
    let lock_canisters = super::locker::get_all_lock_canisters().await?;
    ic_cdk::println!("  Found {} lock canisters", lock_canisters.len());

    if lock_canisters.is_empty() {
        ic_cdk::println!("⚠️  No lock canisters found, returning zero TVL");
        return Ok(vec![
            (TrackedToken::ALEX, 0.0),
            (TrackedToken::ZERO, 0.0),
            (TrackedToken::KONG, 0.0),
            (TrackedToken::BOB, 0.0),
        ]);
    }

    // Initialize TVL accumulator for each token
    let mut tvl_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    tvl_map.insert("ALEX".to_string(), 0.0);
    tvl_map.insert("ZERO".to_string(), 0.0);
    tvl_map.insert("KONG".to_string(), 0.0);
    tvl_map.insert("BOB".to_string(), 0.0);

    let kongswap = Principal::from_text(KONGSWAP_CANISTER)
        .map_err(|e| IcpiError::Other(format!("Invalid kongswap canister ID: {}", e)))?;

    // Query balances for each lock canister in parallel
    let balance_futures: Vec<_> = lock_canisters.iter().map(|(_, lock_principal)| {
        let lock_id = lock_principal.to_text();
        async move {
            let (result,): (UserBalancesResult,) = ic_cdk::call(
                kongswap,
                "user_balances",
                (lock_id.clone(),)
            ).await.map_err(|e| {
                // Log error but don't fail entire TVL calculation for one user
                ic_cdk::println!("  ⚠️  Failed to query balances for {}: {:?}", lock_id, e.1);
                IcpiError::Other(format!("Balance query failed: {:?}", e.1))
            })?;

            Ok::<_, IcpiError>((lock_id, result))
        }
    }).collect();

    let balance_results = futures::future::join_all(balance_futures).await;

    // Process results
    let mut successful_queries = 0;
    for result in balance_results {
        match result {
            Ok((lock_id, UserBalancesResult::Ok(balances))) => {
                successful_queries += 1;

                // Process each LP balance entry
                for balance_entry in balances {
                    if let UserBalancesReply::LP(lp) = balance_entry {
                        // Check if this is a tracked token by looking at symbol_0 or symbol_1
                        for tracked_symbol in ["ALEX", "ZERO", "KONG", "BOB"] {
                            if lp.symbol_0 == tracked_symbol || lp.symbol_1 == tracked_symbol {
                                // Add USD value to accumulator
                                *tvl_map.get_mut(tracked_symbol).unwrap() += lp.usd_balance;

                                ic_cdk::println!(
                                    "  {} in {}: ${:.2}",
                                    tracked_symbol,
                                    &lock_id[..8],
                                    lp.usd_balance
                                );
                                break;
                            }
                        }
                    }
                }
            }
            Ok((lock_id, UserBalancesResult::Err(e))) => {
                ic_cdk::println!("  ⚠️  Kongswap error for {}: {}", &lock_id[..8], e);
            }
            Err(e) => {
                // Already logged in the query
                continue;
            }
        }
    }

    ic_cdk::println!("✅ Queried {}/{} lock canisters successfully", successful_queries, lock_canisters.len());

    // Convert to output format
    let tvl_vec = vec![
        (TrackedToken::ALEX, *tvl_map.get("ALEX").unwrap()),
        (TrackedToken::ZERO, *tvl_map.get("ZERO").unwrap()),
        (TrackedToken::KONG, *tvl_map.get("KONG").unwrap()),
        (TrackedToken::BOB, *tvl_map.get("BOB").unwrap()),
    ];

    // Log totals
    let total_tvl: f64 = tvl_vec.iter().map(|(_, v)| v).sum();
    ic_cdk::println!("📊 Total Kong Locker TVL: ${:.2}", total_tvl);
    for (token, value) in &tvl_vec {
        ic_cdk::println!("  {}: ${:.2}", token.to_symbol(), value);
    }

    Ok(tvl_vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kongswap_canister_id() {
        assert!(Principal::from_text(KONGSWAP_CANISTER).is_ok());
    }
}
