//! Burning module - Handles ICPI token burning and redemptions
//! Critical operation that reduces token supply

pub mod burn_validator;
pub mod redemption_calculator;
pub mod token_distributor;

#[cfg(test)]
mod tests;

use candid::{CandidType, Deserialize, Nat, Principal};
use crate::infrastructure::{Result, IcpiError};

// Burn result structure
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct BurnResult {
    pub successful_transfers: Vec<(String, Nat)>,    // (token_symbol, amount)
    pub failed_transfers: Vec<(String, Nat, String)>, // (token_symbol, amount, error)
    pub icpi_burned: Nat,
    pub timestamp: u64,
}

// Main burn orchestration function
//
// BURN FLOW (ICRC-2 - Requires TWO Approvals):
// 1. User calls icrc2_approve on ckUSDT ledger to approve backend for 0.1 ckUSDT fee
// 2. User calls icrc2_approve on ICPI ledger to approve backend for burn amount
// 3. User calls this burn_icpi function
// 4. Backend validates request and checks user has sufficient ICPI balance
// 5. Backend collects 0.1 ckUSDT fee via ICRC-2 transfer_from (from ckUSDT approval)
// 6. Backend pulls ICPI from user via ICRC-2 transfer_from (atomically burns it)
// 7. Backend calculates proportional redemptions based on current portfolio
// 8. Backend distributes redemption tokens to user
//
// SECURITY: ICRC-2 prevents race conditions because each burn atomically pulls
// from the specific user's approved tokens, not from a shared pool
pub async fn burn_icpi(caller: Principal, amount: Nat) -> Result<BurnResult> {
    // Check not paused (Phase 2: H-1)
    crate::infrastructure::check_not_paused()?;

    // Acquire reentrancy guard - prevents concurrent burns by same user
    let _guard = crate::infrastructure::BurnGuard::acquire(caller)?;

    // Validate request
    burn_validator::validate_burn_request(&caller, &amount)?;

    // CRITICAL: Check fee approval BEFORE other validations (Phase 3: M-2)
    // This prevents user from wasting gas on validations if they can't afford the fee
    // User must have approved backend for 0.1 ckUSDT on ckUSDT ledger
    ic_cdk::println!("Checking ckUSDT fee approval for user {}", caller);
    let ckusdt_canister = Principal::from_text(crate::infrastructure::constants::CKUSDT_CANISTER_ID)
        .map_err(|e| IcpiError::Other(format!("Invalid ckUSDT principal: {}", e)))?;

    use crate::types::icrc::{AllowanceArgs, Allowance};

    let allowance_result: std::result::Result<(Allowance,), _> = ic_cdk::call(
        ckusdt_canister,
        "icrc2_allowance",
        (AllowanceArgs {
            account: crate::types::icrc::Account {
                owner: caller,
                subaccount: None,
            },
            spender: crate::types::icrc::Account {
                owner: ic_cdk::api::id(), // Backend canister
                subaccount: None,
            },
        },)
    ).await;

    match allowance_result {
        Ok((allowance,)) => {
            let required_fee = Nat::from(crate::infrastructure::constants::MINT_FEE_AMOUNT);
            if allowance.allowance < required_fee {
                ic_cdk::println!(
                    "⚠️ Insufficient fee approval: user approved {} e6, required {} e6",
                    allowance.allowance, required_fee
                );
                return Err(IcpiError::Burn(crate::infrastructure::BurnError::InsufficientFeeAllowance {
                    required: required_fee.to_string(),
                    approved: allowance.allowance.to_string(),
                }));
            }
            ic_cdk::println!("✅ Fee approval sufficient: {} e6 approved", allowance.allowance);
        },
        Err((code, msg)) => {
            // Warning only - proceed with burn, fee collection will fail with clear error if needed
            ic_cdk::println!(
                "⚠️ Could not check fee allowance: {:?} - {}. Proceeding...",
                code, msg
            );
        }
    }

    // Get current supply atomically BEFORE collecting fee
    let current_supply = crate::_2_CRITICAL_DATA::supply_tracker::get_icpi_supply_uncached().await?;

    if current_supply == Nat::from(0u32) {
        return Err(IcpiError::Burn(crate::infrastructure::BurnError::NoSupply));
    }

    // Phase 3: M-3 - Enforce maximum burn amount (10% of supply per transaction)
    // Phase 4 Enhancement: Extracted to burn_validator for testability and reusability
    burn_validator::validate_burn_limit(&amount, &current_supply)?;

    // CRITICAL: Check user has sufficient ICPI balance BEFORE collecting fee
    // This prevents user from paying fee if burn will fail anyway
    let icpi_canister = Principal::from_text(crate::infrastructure::constants::ICPI_CANISTER_ID)
        .map_err(|e| IcpiError::Other(format!("Invalid ICPI principal: {}", e)))?;

    let user_balance_result: std::result::Result<(Nat,), _> = ic_cdk::call(
        icpi_canister,
        "icrc1_balance_of",
        (crate::types::icrc::Account {
            owner: caller,
            subaccount: None,
        },)
    ).await;

    let user_icpi_balance = match user_balance_result {
        Ok((balance,)) => balance,
        Err((code, msg)) => {
            return Err(IcpiError::Query(crate::infrastructure::errors::QueryError::CanisterUnreachable {
                canister: "ICPI ledger".to_string(),
                reason: format!("Balance query failed: {:?} - {}", code, msg),
            }));
        }
    };

    if user_icpi_balance < amount {
        return Err(IcpiError::Burn(crate::infrastructure::BurnError::InsufficientBalance {
            required: amount.to_string(),
            available: user_icpi_balance.to_string(),
        }));
    }

    ic_cdk::println!("User {} has {} ICPI, burning {} ICPI", caller, user_icpi_balance, amount);

    // NOW collect fee (after all validations passed)
    // Fee is 0.1 ckUSDT - user must have approved backend for this amount
    // Same fee structure as minting (prevents spam, covers compute costs)
    ic_cdk::println!("Collecting 0.1 ckUSDT burn fee from user {}", caller);
    match crate::_1_CRITICAL_OPERATIONS::minting::fee_handler::collect_mint_fee(caller).await {
        Ok(_) => {
            ic_cdk::println!("Fee collected successfully for burn from user {}", caller);
        }
        Err(e) => {
            ic_cdk::println!("⚠️ Fee collection failed for burn: {}", e);
            ic_cdk::println!("User must approve backend for 0.1 ckUSDT on ckUSDT ledger first");
            return Err(e);
        }
    }

    ic_cdk::println!("Burning {} ICPI from supply of {}", amount, current_supply);

    // CRITICAL: Transfer ICPI from user to backend (which automatically burns it)
    // Uses ICRC-2 transfer_from so user keeps custody until burn confirmed
    // IMPORTANT: User must have called icrc2_approve on ICPI ledger first to approve backend
    // Backend is the burning account - tokens transferred to it are automatically burned
    let icpi_canister = Principal::from_text(crate::infrastructure::constants::ICPI_CANISTER_ID)
        .map_err(|e| IcpiError::Other(format!("Invalid ICPI principal: {}", e)))?;

    use crate::types::icrc::{TransferFromArgs, TransferFromError};

    let transfer_from_args = TransferFromArgs {
        from: crate::types::icrc::Account {
            owner: caller,
            subaccount: None,
        },
        to: crate::types::icrc::Account {
            owner: ic_cdk::id(),
            subaccount: None,
        },
        amount: amount.clone(),
        fee: None,
        memo: Some(b"ICPI burn".to_vec()),
        created_at_time: Some(ic_cdk::api::time()),
    };

    let transfer_result: std::result::Result<(std::result::Result<Nat, TransferFromError>,), _> = ic_cdk::call(
        icpi_canister,
        "icrc2_transfer_from",
        (transfer_from_args,)
    ).await;

    match transfer_result {
        Ok((Ok(block),)) => {
            ic_cdk::println!("✅ ICPI transferred to burning account at block {} via ICRC-2", block);
        }
        Ok((Err(TransferFromError::InsufficientAllowance { allowance }),)) => {
            ic_cdk::println!("⚠️ Insufficient ICPI approval: required {}, approved {}", amount, allowance);
            ic_cdk::println!("User must call icrc2_approve on ICPI ledger to approve backend first");
            return Err(IcpiError::Burn(crate::infrastructure::BurnError::InsufficientApproval {
                required: amount.to_string(),
                approved: allowance.to_string(),
            }));
        }
        Ok((Err(e),)) => {
            return Err(IcpiError::Burn(crate::infrastructure::BurnError::TokenTransferFailed {
                token: "ICPI".to_string(),
                amount: amount.to_string(),
                reason: format!("ICRC-2 error: {:?}", e),
            }));
        }
        Err((code, msg)) => {
            return Err(IcpiError::Burn(crate::infrastructure::BurnError::TokenTransferFailed {
                token: "ICPI".to_string(),
                amount: amount.to_string(),
                reason: format!("Transfer call failed: {:?} - {}", code, msg),
            }));
        }
    }

    // Calculate redemptions
    let redemptions = redemption_calculator::calculate_redemptions(&amount, &current_supply).await?;

    // Distribute tokens to user (passing actual burn amount)
    let result = token_distributor::distribute_tokens(caller, redemptions, amount.clone()).await?;

    Ok(result)
}
