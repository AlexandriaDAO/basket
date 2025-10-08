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
    // Phase 3: M-5 - Uses atomic parallel query to minimize time gap
    update_mint_status(&mint_id, MintStatus::Snapshotting)?;

    let (current_supply, current_tvl) = match crate::_2_CRITICAL_DATA::get_supply_and_tvl_atomic().await {
        Ok((supply, tvl)) => (supply, tvl),
        Err(e) => {
            handle_mint_failure(
                &mint_id,
                caller,
                pending_mint.amount.clone(),
                format!("Atomic snapshot failed: {}", e)
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

    // CRITICAL TIMING: Snapshot MUST be taken BEFORE collecting user's deposit (Phase 3: M-1)
    //
    // Why this order matters:
    // 1. User's deposit should NOT be included in TVL used for their mint calculation
    // 2. Formula: new_icpi = (deposit * supply) / tvl
    //    - If we snapshot AFTER deposit, tvl includes user's own deposit
    //    - This would cause under-minting (user gets fewer tokens than proportional share)
    //
    // Example scenario (WHY snapshot-before-deposit is correct):
    // - Current TVL: $100, Current Supply: 100 ICPI
    // - User deposits: $10 (should get 10% ownership)
    // - CORRECT (snapshot before): (10 * 100) / 100 = 10 ICPI → user owns 10/110 = 9.09%
    // - WRONG (snapshot after): (10 * 100) / 110 = 9.09 ICPI → user owns 9.09/109.09 = 8.33%
    //
    // Security implications:
    // - Concurrent mints don't interfere (each uses pre-their-deposit TVL)
    // - Maintains proportional ownership guarantee
    // - Prevents gaming via deposit timing
    //
    // Staleness concern (addressed):
    // - Time between snapshot and deposit is typically <2 seconds
    // - Rebalancing has separate guard preventing concurrent execution
    // - Even if stale, proportionality is maintained (deposit/tvl ratio correct)
    let snapshot = MintSnapshot {
        supply: current_supply.clone(),
        tvl: current_tvl.clone(),
        timestamp: ic_cdk::api::time(),
    };

    // Update mint with snapshot
    if let Some(mut mint) = get_pending_mint(&mint_id)? {
        mint.snapshot = Some(snapshot.clone());
        store_pending_mint(mint)?;
    }

    // Check for stale snapshot (warning only, does not block)
    const MAX_SNAPSHOT_AGE_NANOS: u64 = 30_000_000_000; // 30 seconds
    let snapshot_age = ic_cdk::api::time() - snapshot.timestamp;
    if snapshot_age > MAX_SNAPSHOT_AGE_NANOS {
        ic_cdk::println!(
            "⚠️ WARNING: Using snapshot {} seconds old (max recommended: 30s)",
            snapshot_age / 1_000_000_000
        );
    }

    // Step 3: NOW collect deposit (after TVL snapshot taken)
    update_mint_status(&mint_id, MintStatus::CollectingDeposit)?;

    match collect_deposit(caller, pending_mint.amount.clone(), "ICPI mint".to_string()).await {
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

    // SECURITY FIX (Phase 1, H-3): Use pure_math calculate_mint_amount() with proper decimal handling
    // This replaces the inline multiply_and_divide which had decimal discrepancies
    let icpi_to_mint = match crate::infrastructure::math::calculate_mint_amount(
        &pending_mint.amount,  // ckUSDT in e6 decimals
        &current_supply,       // ICPI in e8 decimals
        &current_tvl,          // ckUSDT in e6 decimals
    ) {
        Ok(amount) => {
            ic_cdk::println!(
                "  Mint calculation: deposit={} e6, supply={} e8, tvl={} e6 → icpi={} e8",
                pending_mint.amount, current_supply, current_tvl, amount
            );
            amount
        },
        Err(e) => {
            handle_mint_failure(
                &mint_id,
                caller,
                pending_mint.amount.clone(),
                format!("Mint calculation failed: {}", e)
            ).await?;
            return Err(e);
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