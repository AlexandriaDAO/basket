//! Refund handling for failed mints

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, MintError};
use crate::infrastructure::constants::CKUSDT_CANISTER_ID;
use crate::types::{Account, TransferArgs};

pub async fn refund_deposit(user: Principal, amount: Nat) -> Result<Nat> {
    ic_cdk::println!("Refunding {} to {}", amount, user);

    let ckusdt = Principal::from_text(CKUSDT_CANISTER_ID)
        .map_err(|e| IcpiError::Mint(MintError::RefundFailed {
            user: user.to_text(),
            amount: amount.to_string(),
            reason: format!("Invalid ckUSDT principal: {}", e),
        }))?;

    let transfer_args = TransferArgs {
        to: Account {
            owner: user,
            subaccount: None,
        },
        amount: amount.clone(),
        fee: None,
        memo: Some(b"ICPI mint refund".to_vec()),
        from_subaccount: None,
        created_at_time: None,
    };

    let result: std::result::Result<(crate::types::icrc::TransferResult,), _> = ic_cdk::call(
        ckusdt,
        "icrc1_transfer",
        (transfer_args,)
    ).await;

    match result {
        Ok((crate::types::icrc::TransferResult::Ok(block),)) => {
            ic_cdk::println!("Refund successful: block {}", block);
            Ok(block)
        }
        Ok((crate::types::icrc::TransferResult::Err(e),)) => {
            Err(IcpiError::Mint(MintError::RefundFailed {
                user: user.to_text(),
                amount: amount.to_string(),
                reason: format!("{:?}", e),
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Mint(MintError::RefundFailed {
                user: user.to_text(),
                amount: amount.to_string(),
                reason: format!("Call failed: {:?} - {}", code, msg),
            }))
        }
    }
}