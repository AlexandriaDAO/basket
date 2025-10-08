//! Token balance queries module
//!
//! Queries token balances without caching for financial accuracy

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError};
use crate::infrastructure::errors::{QueryError};
use crate::types::{TrackedToken, Account};

/// Get single token balance without caching
///
/// Queries the specified token canister for the backend's balance
pub async fn get_token_balance_uncached(token: &TrackedToken) -> Result<Nat> {
    ic_cdk::println!("QUERY: Getting balance for token {}", token.to_symbol());

    // Get token canister ID
    let token_canister = token.get_canister_id()?;

    // Backend's account (no subaccount)
    let backend_principal = ic_cdk::id();
    let account = Account {
        owner: backend_principal,
        subaccount: None,
    };

    // ICRC-1 balance_of call
    let result: std::result::Result<(Nat,), _> = ic_cdk::call(
        token_canister,
        "icrc1_balance_of",
        (account,)
    ).await;

    match result {
        Ok((balance,)) => {
            ic_cdk::println!("✅ {} balance: {}", token.to_symbol(), balance);
            Ok(balance)
        }
        Err((code, msg)) => {
            ic_cdk::println!(
                "❌ Balance query failed for {}: {:?} - {}",
                token.to_symbol(), code, msg
            );
            Err(IcpiError::Query(QueryError::CanisterUnreachable {
                canister: token_canister.to_text(),
                reason: format!("{:?}: {}", code, msg),
            }))
        }
    }
}

/// Get all token balances without caching
///
/// Queries all tracked tokens + ckUSDT in parallel for efficiency
pub async fn get_all_balances_uncached() -> Result<Vec<(String, Nat)>> {
    ic_cdk::println!("QUERY: Getting all token balances in parallel");

    // Get all tracked tokens
    let tokens = TrackedToken::all();

    // Create futures for parallel execution
    let balance_futures: Vec<_> = tokens.iter()
        .map(|token| async move {
            let balance = get_token_balance_uncached(token).await?;
            Ok::<(String, Nat), IcpiError>((token.to_symbol().to_string(), balance))
        })
        .collect();

    // Execute all queries in parallel
    let results = futures::future::join_all(balance_futures).await;

    // Collect successful results
    let mut balances = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok((symbol, balance)) => {
                balances.push((symbol, balance));
            }
            Err(e) => {
                errors.push(e.to_string());
            }
        }
    }

    // If ANY query failed, log but continue with what we have
    if !errors.is_empty() {
        ic_cdk::println!("⚠️ {} token balance queries failed", errors.len());
        for error in &errors {
            ic_cdk::println!("  - {}", error);
        }
    }

    // Add ckUSDT balance
    let ckusdt_balance = get_ckusdt_balance().await?;
    balances.push(("ckUSDT".to_string(), ckusdt_balance));

    ic_cdk::println!("✅ Retrieved {} token balances", balances.len());
    Ok(balances)
}

/// Get ckUSDT balance specifically
pub async fn get_ckusdt_balance() -> Result<Nat> {
    use crate::infrastructure::constants::CKUSDT_CANISTER_ID;

    let ckusdt = Principal::from_text(CKUSDT_CANISTER_ID)
        .map_err(|e| IcpiError::Query(QueryError::CanisterUnreachable {
            canister: CKUSDT_CANISTER_ID.to_string(),
            reason: format!("Invalid principal: {}", e),
        }))?;

    let backend_principal = ic_cdk::id();
    let account = Account {
        owner: backend_principal,
        subaccount: None,
    };

    let result: std::result::Result<(Nat,), _> = ic_cdk::call(
        ckusdt,
        "icrc1_balance_of",
        (account,)
    ).await;

    match result {
        Ok((balance,)) => {
            ic_cdk::println!("✅ ckUSDT balance: {}", balance);
            Ok(balance)
        }
        Err((code, msg)) => {
            Err(IcpiError::Query(QueryError::CanisterUnreachable {
                canister: CKUSDT_CANISTER_ID.to_string(),
                reason: format!("{:?}: {}", code, msg),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require mock canisters
    // For now, they demonstrate the expected interface

    #[test]
    fn test_all_tokens_queried() {
        // Verify all tracked tokens are included
        let tokens = TrackedToken::all();
        assert!(tokens.len() >= 4); // ALEX, ZERO, KONG, BOB minimum
    }
}
