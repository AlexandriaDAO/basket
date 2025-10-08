//! # ICRC-2 Approval Module
//!
//! Handles token approvals so Kongswap can execute swaps on our behalf.
//! Uses ICRC-2 `icrc2_approve` standard for secure token allowances.
//!
//! ## Approval Flow
//! 1. Backend approves Kongswap to spend tokens
//! 2. Approval valid for 15 minutes
//! 3. Kongswap executes swap using `transfer_from`
//! 4. Unused approvals expire automatically
//!
//! ## Safety
//! - 15-minute expiry balances security and network congestion handling
//! - Each approval is single-use per swap
//! - Amount exactly matches swap requirement

use candid::{Nat, Principal};
use crate::types::{TrackedToken, icrc::{Account, ApproveArgs, ApproveResult}};
use crate::infrastructure::{Result, IcpiError, errors::TradingError, KONGSWAP_BACKEND_ID};

/// Token approval expiry time in nanoseconds (15 minutes)
/// Increased from 5 minutes to handle potential network congestion
const APPROVAL_EXPIRY_NANOS: u64 = 900_000_000_000;

/// Approve Kongswap to spend our tokens for a swap
///
/// ## Parameters
/// - `token`: Which token to approve (ALEX, ZERO, etc.)
/// - `amount`: Exact amount Kongswap can spend
///
/// ## Returns
/// - `Ok(Nat)`: Approval block index on success
/// - `Err`: If approval fails (insufficient balance, network error, etc.)
///
/// ## Process
/// 1. Get token canister Principal
/// 2. Get Kongswap backend Principal
/// 3. Call `icrc2_approve` with 5-minute expiry
/// 4. Return block index for tracking
///
/// ## Example
/// ```rust
/// // Approve Kongswap to spend 1 ALEX (e8 decimals)
/// let approval_block = approve_token_for_swap(
///     &TrackedToken::ALEX,
///     Nat::from(100_000_000u64)
/// ).await?;
/// ```
pub async fn approve_token_for_swap(
    token: &TrackedToken,
    amount: Nat,
) -> Result<Nat> {
    // Get token canister ID
    let token_canister = token.get_canister_id()
        .map_err(|e| IcpiError::Trading(TradingError::InvalidTokenCanister {
            token: token.to_symbol().to_string(),
            canister_id: e.clone(),
            reason: format!("Failed to get canister ID: {}", e),
        }))?;

    // Get Kongswap backend principal
    let kongswap_principal = Principal::from_text(KONGSWAP_BACKEND_ID)
        .map_err(|e| IcpiError::Trading(TradingError::KongswapError {
            operation: "get_principal".to_string(),
            message: format!("Invalid Kongswap principal: {}", e),
        }))?;

    ic_cdk::println!(
        "📝 Approving {} {} for Kongswap (canister: {})",
        amount,
        token.to_symbol(),
        KONGSWAP_BACKEND_ID
    );

    // Prepare approval args
    let approve_args = ApproveArgs {
        from_subaccount: None,
        spender: Account {
            owner: kongswap_principal,
            subaccount: None,
        },
        amount: amount.clone(),
        expected_allowance: None,
        expires_at: Some(ic_cdk::api::time() + APPROVAL_EXPIRY_NANOS),
        fee: None, // Use default
        memo: Some(b"ICPI rebalancing".to_vec()),
        created_at_time: Some(ic_cdk::api::time()),
    };

    // Call icrc2_approve
    let (result,): (ApproveResult,) = ic_cdk::call(
        token_canister,
        "icrc2_approve",
        (approve_args,)
    )
    .await
    .map_err(|(code, msg)| {
        ic_cdk::println!("❌ Approval call failed: {:?} - {}", code, msg);
        IcpiError::Trading(TradingError::ApprovalFailed {
            token: token.to_symbol().to_string(),
            amount: amount.to_string(),
            reason: format!("Inter-canister call failed: {} - {}", code as u32, msg),
        })
    })?;

    // Handle approval result
    match result {
        ApproveResult::Ok(block_index) => {
            ic_cdk::println!(
                "✅ Approval successful: {} {} (block: {})",
                amount,
                token.to_symbol(),
                block_index
            );
            Ok(block_index)
        }
        ApproveResult::Err(err) => {
            ic_cdk::println!("❌ Approval rejected: {:?}", err);
            Err(IcpiError::Trading(TradingError::ApprovalFailed {
                token: token.to_symbol().to_string(),
                amount: amount.to_string(),
                reason: format!("{:?}", err),
            }))
        }
    }
}

/// Check current allowance for Kongswap (for debugging)
///
/// Not used in production flow, but useful for diagnostics
pub async fn check_kongswap_allowance(
    token: &TrackedToken,
) -> Result<Nat> {
    let token_canister = token.get_canister_id()
        .map_err(|e| IcpiError::Trading(TradingError::InvalidTokenCanister {
            token: token.to_symbol().to_string(),
            canister_id: e.clone(),
            reason: format!("Failed to get canister ID: {}", e),
        }))?;

    let kongswap_principal = Principal::from_text(KONGSWAP_BACKEND_ID)
        .map_err(|e| IcpiError::Trading(TradingError::KongswapError {
            operation: "get_principal".to_string(),
            message: format!("Invalid Kongswap principal: {}", e),
        }))?;

    let backend_account = Account {
        owner: ic_cdk::id(),
        subaccount: None,
    };

    let spender_account = Account {
        owner: kongswap_principal,
        subaccount: None,
    };

    let (allowance,): (Nat,) = ic_cdk::call(
        token_canister,
        "icrc2_allowance",
        (backend_account, spender_account)
    )
    .await
    .map_err(|(code, msg)| {
        IcpiError::Trading(TradingError::KongswapError {
            operation: "check_allowance".to_string(),
            message: format!("Call failed: {} - {}", code as u32, msg),
        })
    })?;

    Ok(allowance)
}
