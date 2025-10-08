//! TVL Calculation from Kong Locker
//!
//! Calculates total value locked across all kong_locker positions for tracked tokens.
//! Used to determine target portfolio allocations.

use candid::Principal;
use crate::infrastructure::{Result, IcpiError, KONGSWAP_BACKEND_ID};
use crate::types::TrackedToken;
use crate::types::kongswap::{UserBalancesResult, UserBalancesReply};

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

    // Get all lock canisters - allow this to fail hard as it's a critical dependency
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

    // Initialize TVL accumulator for each tracked token
    let tracked_tokens = TrackedToken::all();
    let mut tvl_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for token in tracked_tokens {
        tvl_map.insert(token.to_symbol().to_string(), 0.0);
    }

    let kongswap = Principal::from_text(KONGSWAP_BACKEND_ID)
        .map_err(|e| IcpiError::Other(format!("Invalid kongswap canister ID: {}", e)))?;

    // Query balances for each lock canister in parallel
    // CRITICAL: We use Result<Option<...>> to allow partial failures
    // If one canister query fails, we return Ok(None) and continue with others
    let balance_futures: Vec<_> = lock_canisters.iter().map(|(_, lock_principal)| {
        let lock_id = lock_principal.to_text();
        async move {
            match ic_cdk::call::<_, (UserBalancesResult,)>(
                kongswap,
                "user_balances",
                (lock_id.clone(),)
            ).await {
                Ok((result,)) => Ok::<_, IcpiError>(Some((lock_id, result))),
                Err(e) => {
                    // Log error but don't fail entire TVL - return None for this canister
                    ic_cdk::println!("  ⚠️  Failed to query balances for {}: {:?}", lock_id, e.1);
                    Ok(None) // Partial failure - skip this canister
                }
            }
        }
    }).collect();

    let balance_results = futures::future::join_all(balance_futures).await;

    // Process results - partial failures are Ok(None)
    let mut successful_queries = 0;
    let mut failed_queries = 0;
    for result in balance_results {
        match result {
            Ok(Some((lock_id, UserBalancesResult::Ok(balances)))) => {
                successful_queries += 1;

                // Process each LP balance entry
                for balance_entry in balances {
                    let UserBalancesReply::LP(lp) = balance_entry;  // UserBalancesReply only has LP variant

                    // CRITICAL: LP positions have two sides (e.g., ALEX/ckUSDT)
                    // usd_balance = total USD value of both sides
                    // usd_amount_0 = USD value of symbol_0 side only
                    // usd_amount_1 = USD value of symbol_1 side only
                    // We must use usd_amount_X to avoid double-counting!

                    // Check symbol_0 and symbol_1 for tracked tokens
                    let mut tracked_found = false;
                    for token in tracked_tokens {
                        let tracked_symbol = token.to_symbol();
                        if lp.symbol_0 == tracked_symbol {
                            // Add only this token's side of the LP
                            *tvl_map.get_mut(tracked_symbol).unwrap() += lp.usd_amount_0;

                            ic_cdk::println!(
                                "  {} (side 0) in {}: ${:.2}",
                                tracked_symbol,
                                &lock_id[..8],
                                lp.usd_amount_0
                            );
                            tracked_found = true;
                        }
                        if lp.symbol_1 == tracked_symbol {
                            // Add only this token's side of the LP
                            *tvl_map.get_mut(tracked_symbol).unwrap() += lp.usd_amount_1;

                            ic_cdk::println!(
                                "  {} (side 1) in {}: ${:.2}",
                                tracked_symbol,
                                &lock_id[..8],
                                lp.usd_amount_1
                            );
                            tracked_found = true;
                        }
                    }

                    // Defensive check: If both sides are tracked tokens (e.g., ALEX/ZERO pool),
                    // we correctly count both sides. This is intentional and expected.
                    if !tracked_found {
                        // This LP position doesn't contain any tracked tokens - skip it
                        ic_cdk::println!(
                            "  Skipping {}/{} pool in {} (no tracked tokens)",
                            lp.symbol_0,
                            lp.symbol_1,
                            &lock_id[..8]
                        );
                    }
                }
            }
            Ok(Some((lock_id, UserBalancesResult::Err(e)))) => {
                ic_cdk::println!("  ⚠️  Kongswap error for {}: {}", &lock_id[..8], e);
                failed_queries += 1;
            }
            Ok(None) => {
                // Query failed (network error, timeout, etc.) - already logged
                failed_queries += 1;
            }
            Err(e) => {
                // This should never happen with our new error handling, but handle defensively
                ic_cdk::println!("  ⚠️  Unexpected error in TVL calculation: {:?}", e);
                failed_queries += 1;
            }
        }
    }

    let total_canisters = lock_canisters.len();
    ic_cdk::println!(
        "✅ Queried {}/{} lock canisters successfully ({} failed)",
        successful_queries,
        total_canisters,
        failed_queries
    );

    // Validate that we have enough successful queries for reliable TVL
    // If more than 50% of queries fail, the TVL data is unreliable
    if total_canisters > 0 && successful_queries == 0 {
        return Err(IcpiError::Other(
            "TVL calculation failed: all lock canister queries failed".to_string()
        ));
    }

    let success_rate = successful_queries as f64 / total_canisters as f64;
    if success_rate < 0.5 {
        return Err(IcpiError::Other(format!(
            "TVL calculation unreliable: only {}/{} queries succeeded ({:.0}% success rate, need >50%)",
            successful_queries,
            total_canisters,
            success_rate * 100.0
        )));
    }

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
        assert!(Principal::from_text(KONGSWAP_BACKEND_ID).is_ok());
    }
}
