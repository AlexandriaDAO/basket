//! # Kongswap Swap Execution Module
//!
//! Executes token swaps on Kongswap DEX for portfolio rebalancing.
//! All swaps use ckUSDT as intermediary and ICRC-2 approval flow.
//!
//! ## Swap Flow
//! 1. Approve Kongswap to spend pay_token
//! 2. Query expected receive amount (for slippage check)
//! 3. Execute swap with `pay_tx_id: None` (ICRC-2)
//! 4. Validate actual slippage vs max_slippage
//! 5. Log results
//!
//! ## Key Constraints
//! - **ICRC-2 Only**: Must use approval flow (`pay_tx_id: None`)
//! - **ckUSDT Intermediary**: All swaps go through ckUSDT
//! - **Sequential**: No parallel swaps (Kongswap limitation)
//! - **Slippage Protected**: Enforces max 2% default slippage

use candid::{Nat, Principal};
use crate::types::{TrackedToken, kongswap::{SwapArgs, SwapReply, SwapAmountsReply, SwapAmountsResult}};
use crate::infrastructure::{Result, IcpiError, errors::TradingError, KONGSWAP_BACKEND_ID};

/// Execute a token swap via Kongswap
///
/// ## Parameters
/// - `pay_token`: Token to send (e.g., ckUSDT to buy ALEX)
/// - `pay_amount`: Amount of pay_token to swap (in token's base units)
/// - `receive_token`: Token to receive (e.g., ALEX when buying)
/// - `max_slippage`: Maximum acceptable slippage as percentage (e.g., 5.0 = 5%)
///
/// ## Returns
/// - `Ok(SwapReply)`: Swap details including actual amounts
/// - `Err`: If approval, swap, or validation fails
///
/// ## Process
/// 1. **Validate inputs**
/// 2. **Approve tokens**: Backend approves Kongswap to spend pay_amount
/// 3. **Query price**: Get expected receive_amount for slippage check
/// 4. **Execute swap**: Call Kongswap `swap()` with ICRC-2 flow
/// 5. **Validate slippage**: Ensure actual matches expected within limits
/// 6. **Log results**: Track swap for rebalance history
///
/// ## Examples
/// ```rust
/// // Buy ALEX with 1 ckUSDT (e6 decimals = 1_000_000) with 2% slippage
/// let swap_result = execute_swap(
///     &TrackedToken::ckUSDT,
///     Nat::from(1_000_000u64),
///     &TrackedToken::ALEX,
///     2.0  // 2% slippage (percentage form, not decimal)
/// ).await?;
///
/// // Sell 10 ALEX for ckUSDT (e8 decimals = 1_000_000_000) with 5% slippage
/// let swap_result = execute_swap(
///     &TrackedToken::ALEX,
///     Nat::from(1_000_000_000u64),
///     &TrackedToken::ckUSDT,
///     5.0  // 5% slippage (percentage form, not decimal)
/// ).await?;
/// ```
pub async fn execute_swap(
    pay_token: &TrackedToken,
    pay_amount: Nat,
    receive_token: &TrackedToken,
    max_slippage: f64,
) -> Result<SwapReply> {
    // === STEP 1: Validate Inputs ===
    validate_swap_params(pay_token, &pay_amount, receive_token, max_slippage)?;

    ic_cdk::println!(
        "🔄 Executing swap: {} {} → {} (max slippage: {:.2}%)",
        pay_amount,
        pay_token.to_symbol(),
        receive_token.to_symbol(),
        max_slippage * 100.0
    );

    // === STEP 2: Approve Tokens ===
    let approval_block = super::approvals::approve_token_for_swap(
        pay_token,
        pay_amount.clone()
    ).await?;

    ic_cdk::println!("✅ Approval complete (block: {})", approval_block);

    // === STEP 3: Query Expected Output ===
    let expected_receive = query_swap_amounts(
        pay_token.to_symbol(),
        pay_amount.clone(),
        receive_token.to_symbol()
    ).await?;

    ic_cdk::println!(
        "📊 Expected to receive: {} {}",
        expected_receive,
        receive_token.to_symbol()
    );

    // === STEP 4: Execute Swap ===
    let kongswap_principal = Principal::from_text(KONGSWAP_BACKEND_ID)
        .map_err(|e| IcpiError::Trading(TradingError::KongswapError {
            operation: "get_principal".to_string(),
            message: format!("Invalid Kongswap principal: {}", e),
        }))?;

    let swap_args = SwapArgs {
        pay_token: pay_token.to_symbol().to_string(),
        pay_amount: pay_amount.clone(),
        pay_tx_id: None, // CRITICAL: None = ICRC-2 flow (approval-based)
        receive_token: receive_token.to_symbol().to_string(),
        receive_amount: None, // Let Kongswap calculate
        receive_address: Some(ic_cdk::id().to_text()), // Send to our backend
        max_slippage: Some(max_slippage),
        referred_by: None,
    };

    ic_cdk::println!("📤 Calling Kongswap swap()...");

    let (swap_result,): (std::result::Result<SwapReply, String>,) = ic_cdk::call(
        kongswap_principal,
        "swap",
        (swap_args,)
    )
    .await
    .map_err(|(code, msg)| {
        ic_cdk::println!("❌ Swap call failed: {:?} - {}", code, msg);
        IcpiError::Trading(TradingError::SwapFailed {
            pay_token: pay_token.to_symbol().to_string(),
            receive_token: receive_token.to_symbol().to_string(),
            amount: pay_amount.clone(),
            reason: format!("Inter-canister call failed: {} - {}", code as u32, msg),
        })
    })?;

    let swap_reply = swap_result.map_err(|e| {
        ic_cdk::println!("❌ Swap rejected by Kongswap: {}", e);
        IcpiError::Trading(TradingError::SwapFailed {
            pay_token: pay_token.to_symbol().to_string(),
            receive_token: receive_token.to_symbol().to_string(),
            amount: pay_amount.clone(),
            reason: e.to_string(),
        })
    })?;

    // === STEP 5: Validate Slippage ===
    super::slippage::validate_swap_result(
        &expected_receive,
        &swap_reply.receive_amount,
        max_slippage
    )?;

    // === STEP 6: Log Success ===
    ic_cdk::println!(
        "✅ Swap complete: {} {} → {} {} (slippage: {:.2}%, price: {})",
        pay_amount,
        pay_token.to_symbol(),
        swap_reply.receive_amount,
        receive_token.to_symbol(),
        swap_reply.slippage * 100.0,
        swap_reply.price
    );

    Ok(swap_reply)
}

/// Query expected swap output for slippage calculation
///
/// Calls Kongswap's `swap_amounts` to get current pool price
/// without executing the trade.
///
/// ## Parameters
/// - `pay_symbol`: Token symbol to send (e.g., "ckUSDT")
/// - `pay_amount`: Amount to send
/// - `receive_symbol`: Token symbol to receive (e.g., "ALEX")
///
/// ## Returns
/// - `Ok(Nat)`: Expected receive amount
/// - `Err`: If query fails or pool doesn't exist
async fn query_swap_amounts(
    pay_symbol: &str,
    pay_amount: Nat,
    receive_symbol: &str,
) -> Result<Nat> {
    let kongswap_principal = Principal::from_text(KONGSWAP_BACKEND_ID)
        .map_err(|e| IcpiError::Trading(TradingError::KongswapError {
            operation: "get_principal".to_string(),
            message: format!("Invalid Kongswap principal: {}", e),
        }))?;

    let (result,): (SwapAmountsResult,) = ic_cdk::call(
        kongswap_principal,
        "swap_amounts",
        (pay_symbol, pay_amount.clone(), receive_symbol)
    )
    .await
    .map_err(|(code, msg)| {
        IcpiError::Trading(TradingError::KongswapError {
            operation: "swap_amounts".to_string(),
            message: format!("Call failed: {} - {}", code as u32, msg),
        })
    })?;

    match result {
        SwapAmountsResult::Ok(reply) => Ok(reply.receive_amount),
        SwapAmountsResult::Err(e) => {
            Err(IcpiError::Trading(TradingError::KongswapError {
                operation: "swap_amounts".to_string(),
                message: format!("Query failed: {}", e),
            }))
        }
    }
}

/// Validate swap parameters before execution
///
/// Checks:
/// - Pay amount > 0
/// - Max slippage reasonable (0-10%)
/// - Tokens are different
/// - Both tokens are tracked
fn validate_swap_params(
    pay_token: &TrackedToken,
    pay_amount: &Nat,
    receive_token: &TrackedToken,
    max_slippage: f64,
) -> Result<()> {
    // Check pay amount > 0
    if pay_amount == &Nat::from(0u64) {
        return Err(IcpiError::Trading(TradingError::InvalidSwapAmount {
            reason: "Pay amount must be greater than zero".to_string(),
        }));
    }

    // Check max slippage is reasonable (expects percentage form: 5.0 = 5%)
    if max_slippage < 0.0 || max_slippage > 10.0 {
        return Err(IcpiError::Trading(TradingError::InvalidSwapAmount {
            reason: format!(
                "Max slippage must be between 0% and 10%, got {:.2}%",
                max_slippage  // Already in percentage form
            ),
        }));
    }

    // Check tokens are different
    if pay_token.to_symbol() == receive_token.to_symbol() {
        return Err(IcpiError::Trading(TradingError::InvalidSwapAmount {
            reason: "Pay and receive tokens must be different".to_string(),
        }));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_swap_params_valid() {
        let result = validate_swap_params(
            &TrackedToken::ckUSDT,
            &Nat::from(1_000_000u64),
            &TrackedToken::ALEX,
            2.0  // 2% in percentage form
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_swap_params_zero_amount() {
        let result = validate_swap_params(
            &TrackedToken::ckUSDT,
            &Nat::from(0u64),
            &TrackedToken::ALEX,
            2.0  // 2% in percentage form
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_swap_params_invalid_slippage() {
        let result = validate_swap_params(
            &TrackedToken::ckUSDT,
            &Nat::from(1_000_000u64),
            &TrackedToken::ALEX,
            15.0  // 15% is too high
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_swap_params_same_token() {
        let result = validate_swap_params(
            &TrackedToken::ALEX,
            &Nat::from(1_000_000u64),
            &TrackedToken::ALEX,
            2.0  // 2% in percentage form
        );
        assert!(result.is_err());
    }
}
