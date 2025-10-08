//! Fee handling for mint operations

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, MintError};
use crate::infrastructure::constants::{MINT_FEE_E6, CKUSDT_CANISTER_ID};
use crate::types::{Account, TransferArgs};

/// Collect minting fee from user
pub async fn collect_mint_fee(user: Principal) -> Result<Nat> {
    let fee_amount = Nat::from(MINT_FEE_E6);

    ic_cdk::println!("Collecting mint fee of {} from {}", fee_amount, user);

    let ckusdt = Principal::from_text(CKUSDT_CANISTER_ID)
        .map_err(|e| IcpiError::Mint(MintError::FeeCollectionFailed {
            user: user.to_text(),
            reason: format!("Invalid ckUSDT principal: {}", e),
        }))?;

    // ICRC-2 transfer_from requires approval first
    // User must have called icrc2_approve before this
    use crate::types::icrc::{TransferFromArgs, TransferFromError};

    let args = TransferFromArgs {
        from: Account { owner: user, subaccount: None },
        to: Account { owner: ic_cdk::id(), subaccount: None },
        amount: fee_amount.clone(),
        fee: None,
        memo: Some(b"ICPI mint fee".to_vec()),
        created_at_time: Some(ic_cdk::api::time()),
    };

    let result: std::result::Result<(std::result::Result<Nat, TransferFromError>,), _> = ic_cdk::call(
        ckusdt,
        "icrc2_transfer_from",
        (args,)
    ).await;

    match result {
        Ok((Ok(block_index),)) => {
            ic_cdk::println!("✅ Fee collected: block {}", block_index);
            Ok(fee_amount)
        }
        Ok((Err(e),)) => {
            Err(IcpiError::Mint(MintError::FeeCollectionFailed {
                user: user.to_text(),
                reason: format!("ICRC-2 error: {:?}", e),
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Mint(MintError::FeeCollectionFailed {
                user: user.to_text(),
                reason: format!("Call failed: {:?} - {}", code, msg),
            }))
        }
    }
}

/// Collect deposit from user for minting
pub async fn collect_deposit(
    user: Principal,
    amount: Nat,
    memo: String,
) -> Result<Nat> {
    ic_cdk::println!("Collecting deposit of {} from {} (memo: {})", amount, user, memo);

    let ckusdt = Principal::from_text(CKUSDT_CANISTER_ID)
        .map_err(|e| IcpiError::Mint(MintError::DepositCollectionFailed {
            user: user.to_text(),
            amount: amount.to_string(),
            reason: format!("Invalid ckUSDT principal: {}", e),
        }))?;

    use crate::types::icrc::{TransferFromArgs, TransferFromError};

    let args = TransferFromArgs {
        from: Account { owner: user, subaccount: None },
        to: Account { owner: ic_cdk::id(), subaccount: None },
        amount: amount.clone(),
        fee: None,
        memo: Some(memo.into_bytes()),
        created_at_time: Some(ic_cdk::api::time()),
    };

    let result: std::result::Result<(std::result::Result<Nat, TransferFromError>,), _> = ic_cdk::call(
        ckusdt,
        "icrc2_transfer_from",
        (args,)
    ).await;

    match result {
        Ok((Ok(block_index),)) => {
            ic_cdk::println!("✅ Deposit collected: block {}", block_index);
            Ok(amount)
        }
        Ok((Err(e),)) => {
            Err(IcpiError::Mint(MintError::DepositCollectionFailed {
                user: user.to_text(),
                amount: amount.to_string(),
                reason: format!("ICRC-2 error: {:?}", e),
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Mint(MintError::DepositCollectionFailed {
                user: user.to_text(),
                amount: amount.to_string(),
                reason: format!("Call failed: {:?} - {}", code, msg),
            }))
        }
    }
}