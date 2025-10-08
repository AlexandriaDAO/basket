//! Token distribution after burn
//! Handles the actual token transfers to users

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, BurnError};
use crate::types::{TrackedToken, Account, TransferArgs, TransferResult};
use super::BurnResult;

/// Distribute calculated redemption amounts to user
/// Now with parallel execution for efficiency
pub async fn distribute_tokens(
    recipient: Principal,
    redemptions: Vec<(String, Nat)>,
    icpi_burn_amount: Nat,
) -> Result<BurnResult> {
    let mut result = BurnResult {
        successful_transfers: Vec::new(),
        failed_transfers: Vec::new(),
        icpi_burned: icpi_burn_amount,
        timestamp: ic_cdk::api::time(),
    };

    // Execute all transfers in parallel
    let transfer_futures: Vec<_> = redemptions.iter()
        .map(|(token_symbol, amount)| {
            let symbol = token_symbol.clone();
            let amt = amount.clone();
            async move {
                (symbol.clone(), amt.clone(), transfer_token(&symbol, recipient, amt).await)
            }
        })
        .collect();

    let transfer_results = futures::future::join_all(transfer_futures).await;

    // Process results
    for (token_symbol, amount, transfer_result) in transfer_results {
        match transfer_result {
            Ok(block_index) => {
                ic_cdk::println!("✓ Transferred {} {} to {} (block: {})",
                    amount, token_symbol, recipient, block_index);
                result.successful_transfers.push((token_symbol, amount));
            }
            Err(e) => {
                ic_cdk::println!("✗ Failed to transfer {} {}: {}",
                    amount, token_symbol, e);
                result.failed_transfers.push((token_symbol, amount, e.to_string()));
            }
        }
    }

    // Check if all transfers failed
    if result.successful_transfers.is_empty() && !result.failed_transfers.is_empty() {
        return Err(IcpiError::Burn(BurnError::NoRedemptionsPossible {
            reason: "All token transfers failed".to_string(),
        }));
    }

    Ok(result)
}

async fn transfer_token(
    token_symbol: &str,
    recipient: Principal,
    amount: Nat,
) -> Result<Nat> {
    let token_canister = get_token_canister(token_symbol)?;

    let amount_str = amount.to_string();

    let transfer_args = TransferArgs {
        to: Account {
            owner: recipient,
            subaccount: None,
        },
        amount,
        fee: None,
        memo: Some(b"ICPI burn redemption".to_vec()),
        from_subaccount: None,
        created_at_time: None,
    };

    let result: std::result::Result<(TransferResult,), _> = ic_cdk::call(
        token_canister,
        "icrc1_transfer",
        (transfer_args,)
    ).await;

    match result {
        Ok((TransferResult::Ok(block),)) => Ok(block),
        Ok((TransferResult::Err(e),)) => {
            Err(IcpiError::Burn(BurnError::TokenTransferFailed {
                token: token_symbol.to_string(),
                amount: amount_str.clone(),
                reason: format!("{:?}", e),
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Burn(BurnError::TokenTransferFailed {
                token: token_symbol.to_string(),
                amount: amount_str.clone(),
                reason: format!("Call failed: {:?} - {}", code, msg),
            }))
        }
    }
}

fn get_token_canister(symbol: &str) -> Result<Principal> {
    use crate::infrastructure::constants::*;

    let id = match symbol {
        "ckUSDT" => CKUSDT_CANISTER_ID,
        "ALEX" => ALEX_CANISTER_ID,
        "ZERO" => ZERO_CANISTER_ID,
        "KONG" => KONG_CANISTER_ID,
        "BOB" => BOB_CANISTER_ID,
        _ => return Err(IcpiError::Burn(BurnError::TokenTransferFailed {
            token: symbol.to_string(),
            amount: "0".to_string(),
            reason: "Unknown token".to_string(),
        }))
    };

    Principal::from_text(id)
        .map_err(|e| IcpiError::System(crate::infrastructure::errors::SystemError::StateCorrupted {
            reason: format!("Invalid canister ID for {}: {}", symbol, e),
        }))
}

