//! Main mint orchestration logic

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, MintError};
use crate::infrastructure::constants::ICPI_CANISTER_ID;
use super::mint_state::{MintStatus, PendingMint, MintSnapshot, store_pending_mint, get_pending_mint, update_mint_status};
use super::mint_validator::validate_mint_request;
use super::fee_handler::{collect_mint_fee, collect_deposit};
use super::refund_handler::refund_deposit;

/// Initiate a new mint request
pub async fn initiate_mint(caller: Principal, amount: Nat) -> Result<String> {
    // Validate request
    validate_mint_request(&caller, &amount)?;

    // Generate unique mint ID
    let mint_id = format!("mint_{}_{}", caller.to_text(), ic_cdk::api::time());
    let now = ic_cdk::api::time();

    // Create pending mint
    let pending_mint = PendingMint {
        id: mint_id.clone(),
        user: caller,
        amount: amount.clone(),
        status: MintStatus::Pending,
        created_at: now,
        last_updated: now,
        snapshot: None,
    };

    // Store pending mint
    store_pending_mint(pending_mint)?;

    ic_cdk::println!("Mint initiated: {} for user {} amount {}", mint_id, caller, amount);

    Ok(mint_id)
}

/// Complete a pending mint request
pub async fn complete_mint(caller: Principal, mint_id: String) -> Result<Nat> {
    // Acquire reentrancy guard - prevents concurrent mints by same user
    let _guard = crate::infrastructure::MintGuard::acquire(caller)?;

    // Get pending mint
    let pending_mint = get_pending_mint(&mint_id)?
        .ok_or_else(|| IcpiError::Mint(MintError::InvalidMintId {
            id: mint_id.clone(),
        }))?;

    // Verify ownership
    if pending_mint.user != caller {
        return Err(IcpiError::Mint(MintError::Unauthorized {
            principal: caller.to_text(),
            mint_id: mint_id.clone(),
        }));
    }

    // Check if already completed
    if let MintStatus::Complete(amount) = pending_mint.status {
        return Ok(amount);
    }

    // Step 1: Collect fee
    update_mint_status(&mint_id, MintStatus::CollectingFee)?;

    match collect_mint_fee(caller).await {
        Ok(_) => {
            ic_cdk::println!("Fee collected for mint {}", mint_id);
        }
        Err(e) => {
            update_mint_status(&mint_id, MintStatus::Failed(format!("Fee collection failed: {}", e)))?;
            return Err(e);
        }
    }

    // Step 2: Take snapshot of supply and TVL BEFORE collecting deposit
    update_mint_status(&mint_id, MintStatus::Snapshotting)?;

    let current_supply = match crate::_2_CRITICAL_DATA::supply_tracker::get_icpi_supply_uncached().await {
        Ok(supply) => supply,
        Err(e) => {
            handle_mint_failure(
                &mint_id,
                caller,
                pending_mint.amount.clone(),
                format!("Failed to query ICPI supply: {}", e)
            ).await?;
            return Err(e);
        }
    };

    let current_tvl = match crate::_2_CRITICAL_DATA::portfolio_value::calculate_portfolio_value_atomic().await {
        Ok(tvl) => tvl,
        Err(e) => {
            handle_mint_failure(
                &mint_id,
                caller,
                pending_mint.amount.clone(),
                format!("TVL calculation failed: {}", e)
            ).await?;
            return Err(e);
        }
    };

    // Validate TVL is not zero
    if current_tvl == Nat::from(0u32) {
        update_mint_status(&mint_id, MintStatus::Failed("TVL is zero - canister has no holdings".to_string()))?;
        return Err(IcpiError::Mint(MintError::InsufficientTVL {
            tvl: "0".to_string(),
            required: "non-zero".to_string(),
        }));
    }

    ic_cdk::println!("Pre-deposit TVL: {} ckUSDT (e6), Supply: {} ICPI (e8)", current_tvl, current_supply);

    // Store snapshot
    let snapshot = MintSnapshot {
        supply: current_supply.clone(),
        tvl: current_tvl.clone(),
        timestamp: ic_cdk::api::time(),
    };

    // Update mint with snapshot
    if let Some(mut mint) = get_pending_mint(&mint_id)? {
        mint.snapshot = Some(snapshot);
        store_pending_mint(mint)?;
    }

    // Step 3: NOW collect deposit (after TVL snapshot taken)
    update_mint_status(&mint_id, MintStatus::CollectingDeposit)?;

    match collect_deposit(caller, pending_mint.amount.clone(), format!("ICPI mint {}", mint_id)).await {
        Ok(_) => {
            ic_cdk::println!("Deposit collected for mint {}", mint_id);
        }
        Err(e) => {
            update_mint_status(&mint_id, MintStatus::Failed(format!("Deposit collection failed: {}", e)))?;
            return Err(e);
        }
    }

    // Step 4: Calculate ICPI to mint using pre-deposit TVL
    update_mint_status(&mint_id, MintStatus::Calculating)?;

    let icpi_to_mint = if current_supply == Nat::from(0u32) {
        // Initial mint: 1 ICPI = 1 ckUSDT (adjust for decimals)
        // ckUSDT has 6 decimals, ICPI has 8 decimals
        // Use proper decimal conversion: 1 ckUSDT (e6) = 100 ICPI (e8)
        crate::infrastructure::math::convert_decimals(
            &pending_mint.amount,
            crate::infrastructure::constants::CKUSDT_DECIMALS,
            crate::infrastructure::constants::ICPI_DECIMALS
        )?
    } else {
        // Formula: new_icpi = (deposit * current_supply) / current_tvl
        match crate::infrastructure::math::multiply_and_divide(&pending_mint.amount, &current_supply, &current_tvl) {
            Ok(amount) => amount,
            Err(e) => {
                handle_mint_failure(
                    &mint_id,
                    caller,
                    pending_mint.amount.clone(),
                    format!("Calculation failed: {}", e)
                ).await?;
                return Err(e);
            }
        }
    };

    ic_cdk::println!("Calculated ICPI to mint: {}", icpi_to_mint);

    // Step 5: Mint ICPI tokens on the actual ICPI ledger
    update_mint_status(&mint_id, MintStatus::Minting)?;

    match mint_icpi_on_ledger(caller, icpi_to_mint.clone()).await {
        Ok(block_index) => {
            ic_cdk::println!("Minted {} ICPI to {} (block: {})", icpi_to_mint, caller, block_index);
        }
        Err(e) => {
            handle_mint_failure(
                &mint_id,
                caller,
                pending_mint.amount.clone(),
                format!("Ledger minting failed: {}", e)
            ).await?;
            return Err(e);
        }
    }

    // Step 6: Mark as complete
    update_mint_status(&mint_id, MintStatus::Complete(icpi_to_mint.clone()))?;

    Ok(icpi_to_mint)
}

/// Handle mint failure and attempt refund
async fn handle_mint_failure(
    mint_id: &str,
    user: Principal,
    amount: Nat,
    reason: String,
) -> Result<()> {
    update_mint_status(mint_id, MintStatus::Refunding)?;

    match refund_deposit(user, amount.clone()).await {
        Ok(_) => {
            ic_cdk::println!("Successfully refunded {} to {}", amount, user);
            update_mint_status(mint_id, MintStatus::FailedRefunded(
                format!("{}, deposit refunded", reason)
            ))?;
        }
        Err(refund_err) => {
            ic_cdk::println!("ERROR: Failed to refund deposit: {}", refund_err);
            update_mint_status(mint_id, MintStatus::FailedNoRefund(
                format!("{}. Refund failed: {}. Amount: {}. Contact support.", reason, refund_err, amount)
            ))?;
        }
    }

    Ok(())
}

/// Mint ICPI tokens on the ledger
pub async fn mint_icpi_on_ledger(recipient: Principal, amount: Nat) -> Result<Nat> {
    let icpi_ledger = Principal::from_text(ICPI_CANISTER_ID)
        .map_err(|e| IcpiError::Mint(MintError::LedgerInteractionFailed {
            operation: "parse_principal".to_string(),
            details: format!("Invalid ICPI principal: {}", e),
        }))?;

    // Call the ledger to mint tokens using icrc1_transfer
    // Backend is the minting account, so transfers create new tokens
    use crate::types::icrc::TransferArgs;

    let transfer_args = TransferArgs {
        from_subaccount: None,
        to: crate::types::Account {
            owner: recipient,
            subaccount: None,
        },
        amount: amount.clone(),
        fee: None, // No fee for minting
        memo: Some(b"ICPI minting".to_vec()),
        created_at_time: Some(ic_cdk::api::time()),
    };

    let result: std::result::Result<(crate::types::icrc::TransferResult,), _> = ic_cdk::call(
        icpi_ledger,
        "icrc1_transfer",
        (transfer_args,)
    ).await;

    match result {
        Ok((crate::types::icrc::TransferResult::Ok(block),)) => {
            Ok(block)
        }
        Ok((crate::types::icrc::TransferResult::Err(e),)) => {
            Err(IcpiError::Mint(MintError::LedgerInteractionFailed {
                operation: "mint".to_string(),
                details: format!("Mint error: {:?}", e),
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Mint(MintError::LedgerInteractionFailed {
                operation: "mint".to_string(),
                details: format!("Call failed: {:?} - {}", code, msg),
            }))
        }
    }
}