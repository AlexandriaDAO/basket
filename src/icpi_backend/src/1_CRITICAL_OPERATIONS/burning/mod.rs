//! Burning module - Handles ICPI token burning and redemptions
//! Critical operation that reduces token supply

pub mod burn_validator;
pub mod redemption_calculator;
pub mod token_distributor;

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
pub async fn burn_icpi(caller: Principal, amount: Nat) -> Result<BurnResult> {
    // Acquire reentrancy guard - prevents concurrent burns by same user
    let _guard = crate::infrastructure::BurnGuard::acquire(caller)?;

    // Validate request
    burn_validator::validate_burn_request(&caller, &amount)?;

    // Get current supply atomically BEFORE collecting fee
    let current_supply = crate::_2_CRITICAL_DATA::supply_tracker::get_icpi_supply_uncached().await?;

    if current_supply == Nat::from(0u32) {
        return Err(IcpiError::Burn(crate::infrastructure::BurnError::NoSupply));
    }

    // CRITICAL: Check backend has received sufficient ICPI from user
    // User must transfer ICPI to backend BEFORE calling this function
    // Backend is the burning account - any ICPI transferred to it is automatically burned
    let icpi_canister = Principal::from_text(crate::infrastructure::constants::ICPI_CANISTER_ID)
        .map_err(|e| IcpiError::Other(format!("Invalid ICPI principal: {}", e)))?;

    let backend_balance_result: std::result::Result<(Nat,), _> = ic_cdk::call(
        icpi_canister,
        "icrc1_balance_of",
        (crate::types::icrc::Account {
            owner: ic_cdk::id(),
            subaccount: None,
        },)
    ).await;

    let backend_icpi_balance = match backend_balance_result {
        Ok((balance,)) => balance,
        Err((code, msg)) => {
            return Err(IcpiError::Query(crate::infrastructure::errors::QueryError::CanisterUnreachable {
                canister: "ICPI ledger".to_string(),
                reason: format!("Balance query failed: {:?} - {}", code, msg),
            }));
        }
    };

    if backend_icpi_balance < amount {
        return Err(IcpiError::Burn(crate::infrastructure::BurnError::InsufficientBalance {
            required: amount.to_string(),
            available: backend_icpi_balance.to_string(),
        }));
    }

    ic_cdk::println!("Backend has {} ICPI, burning {} ICPI for user {}", backend_icpi_balance, amount, caller);

    // NOW collect fee (after all validations passed)
    let _ckusdt = Principal::from_text(crate::infrastructure::constants::CKUSDT_CANISTER_ID)
        .map_err(|e| IcpiError::Other(format!("Invalid ckUSDT principal: {}", e)))?;
    crate::_1_CRITICAL_OPERATIONS::minting::fee_handler::collect_mint_fee(caller).await?;

    ic_cdk::println!("Fee collected for burn from user {}", caller);

    ic_cdk::println!("Burning {} ICPI from supply of {}", amount, current_supply);

    // ICPI has already been transferred to backend (verified above)
    // Backend is the burning account - tokens transferred to it are automatically burned
    // No need for ICRC-2 transfer_from since ICPI ledger doesn't support ICRC-2
    ic_cdk::println!("ICPI already at burning account (backend), proceeding with redemption");

    // Calculate redemptions
    let redemptions = redemption_calculator::calculate_redemptions(&amount, &current_supply).await?;

    // Distribute tokens to user (passing actual burn amount)
    let result = token_distributor::distribute_tokens(caller, redemptions, amount.clone()).await?;

    Ok(result)
}
