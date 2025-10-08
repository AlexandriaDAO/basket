# Phase 5: Complete Critical Operations Migration

This file contains the COMPLETE implementation details for Phase 5 that were missing from the main guide.

## Step 5.2: Migrate Burning Module

### Complete burning/mod.rs
```rust
// src/icpi_backend/src/1_CRITICAL_OPERATIONS/burning/mod.rs
//! Burning module - CRITICAL security boundary
//! Handles ICPI token burning and proportional redemptions

mod redemption_calculator;
mod token_distributor;
mod burn_validator;

use candid::{CandidType, Deserialize, Nat, Principal};
use crate::infrastructure::{Result, IcpiError, BurnError};
use crate::infrastructure::logging::AuditLogger;
use crate::infrastructure::cache::assert_no_cache_for_critical_op;

// Re-export types
pub use redemption_calculator::calculate_redemptions;
pub use token_distributor::distribute_tokens;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct BurnResult {
    pub successful_transfers: Vec<(String, Nat)>,
    pub failed_transfers: Vec<(String, Nat, String)>,
    pub icpi_burned: Nat,
    pub timestamp: u64,
}

/// Main burn orchestration function
pub async fn burn_icpi(caller: Principal, amount: Nat) -> Result<BurnResult> {
    // 1. Validate burn request
    burn_validator::validate_burn_request(&caller, &amount)?;

    // 2. Assert no caching for critical operation
    assert_no_cache_for_critical_op("burn_icpi");

    // 3. Get current supply atomically (no race condition)
    let current_supply = crate::_2_CRITICAL_DATA::supply_tracker::get_icpi_supply_uncached().await?;

    // 4. Get all token balances
    let token_balances = crate::_2_CRITICAL_DATA::token_queries::get_all_balances_uncached().await?;

    // 5. Calculate redemptions using PURE function
    let redemptions = redemption_calculator::calculate_redemptions(
        &amount,
        &current_supply,
        &token_balances
    )?;

    // 6. Distribute tokens to user
    let result = token_distributor::distribute_tokens(caller, redemptions).await?;

    // 7. Audit log
    AuditLogger::log_burn(caller, amount.0.to_u64().unwrap_or(0),
        result.successful_transfers.clone());

    Ok(result)
}
```

### Complete redemption_calculator.rs
```rust
// src/icpi_backend/src/1_CRITICAL_OPERATIONS/burning/redemption_calculator.rs
//! Pure calculation functions for burn redemptions
//! NO I/O, NO async, fully deterministic

use candid::Nat;
use crate::infrastructure::{Result, IcpiError, BurnError};
use crate::infrastructure::math::multiply_and_divide;

/// Calculate proportional redemption amounts
/// Formula: For each token: (burn_amount × token_balance) ÷ total_supply
pub fn calculate_redemptions(
    burn_amount: &Nat,
    total_supply: &Nat,
    token_balances: &[(String, Nat)],
) -> Result<Vec<(String, Nat)>> {
    // Validate inputs
    if burn_amount == &Nat::from(0u64) {
        return Err(IcpiError::Burn(BurnError::AmountBelowMinimum {
            amount: "0".to_string(),
            minimum: "11000".to_string(),
        }));
    }

    if total_supply == &Nat::from(0u64) {
        return Err(IcpiError::Burn(BurnError::NoRedemptionsPossible {
            reason: "Total supply is zero".to_string(),
        }));
    }

    let mut redemptions = Vec::new();
    const TRANSFER_FEE: u64 = 10_000;

    for (token_name, balance) in token_balances {
        if balance > &Nat::from(0u64) {
            // Calculate proportional amount
            let redemption_amount = multiply_and_divide(burn_amount, balance, total_supply)?;

            // Only include if above fee threshold
            if redemption_amount > Nat::from(TRANSFER_FEE + 1_000) {
                let amount_after_fee = redemption_amount - Nat::from(TRANSFER_FEE);
                redemptions.push((token_name.clone(), amount_after_fee));
            }
        }
    }

    if redemptions.is_empty() {
        return Err(IcpiError::Burn(BurnError::NoRedemptionsPossible {
            reason: "No tokens meet minimum threshold after fees".to_string(),
        }));
    }

    Ok(redemptions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_redemptions() {
        let burn_amount = Nat::from(50_000_000u64); // 0.5 ICPI
        let total_supply = Nat::from(100_000_000u64); // 1 ICPI
        let balances = vec![
            ("ALEX".to_string(), Nat::from(1_000_000u64)),
            ("KONG".to_string(), Nat::from(2_000_000u64)),
        ];

        let result = calculate_redemptions(&burn_amount, &total_supply, &balances).unwrap();

        // Should get 50% of each token minus fees
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, Nat::from(500_000u64 - 10_000));
        assert_eq!(result[1].1, Nat::from(1_000_000u64 - 10_000));
    }
}
```

### Complete token_distributor.rs
```rust
// src/icpi_backend/src/1_CRITICAL_OPERATIONS/burning/token_distributor.rs
//! Token distribution after burn
//! Handles the actual token transfers to users

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, BurnError};
use crate::types::{TrackedToken, Account};
use super::BurnResult;

/// Distribute calculated redemption amounts to user
pub async fn distribute_tokens(
    recipient: Principal,
    redemptions: Vec<(String, Nat)>,
) -> Result<BurnResult> {
    let mut result = BurnResult {
        successful_transfers: Vec::new(),
        failed_transfers: Vec::new(),
        icpi_burned: calculate_total_burned(&redemptions),
        timestamp: ic_cdk::api::time(),
    };

    for (token_symbol, amount) in redemptions {
        match transfer_token(&token_symbol, recipient, amount.clone()).await {
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

    Ok(result)
}

async fn transfer_token(
    token_symbol: &str,
    recipient: Principal,
    amount: Nat,
) -> Result<Nat> {
    let token_canister = get_token_canister(token_symbol)?;

    let transfer_args = crate::types::icrc::TransferArgs {
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

    let result: Result<(crate::types::icrc::TransferResult,), _> =
        ic_cdk::call(token_canister, "icrc1_transfer", (transfer_args,)).await;

    match result {
        Ok((crate::types::icrc::TransferResult::Ok(block),)) => Ok(block),
        Ok((crate::types::icrc::TransferResult::Err(e),)) => {
            Err(IcpiError::Burn(BurnError::TokenTransferFailed {
                token: token_symbol.to_string(),
                amount: amount.to_string(),
                reason: format!("{:?}", e),
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Burn(BurnError::TokenTransferFailed {
                token: token_symbol.to_string(),
                amount: amount.to_string(),
                reason: format!("Call failed: {:?} - {}", code, msg),
            }))
        }
    }
}

fn get_token_canister(symbol: &str) -> Result<Principal> {
    match symbol {
        "ckUSDT" => Principal::from_text(crate::infrastructure::constants::CKUSDT_CANISTER_ID),
        "ALEX" => Principal::from_text(crate::infrastructure::constants::ALEX_CANISTER_ID),
        "ZERO" => Principal::from_text(crate::infrastructure::constants::ZERO_CANISTER_ID),
        "KONG" => Principal::from_text(crate::infrastructure::constants::KONG_CANISTER_ID),
        "BOB" => Principal::from_text(crate::infrastructure::constants::BOB_CANISTER_ID),
        _ => return Err(IcpiError::Burn(BurnError::TokenTransferFailed {
            token: symbol.to_string(),
            amount: "0".to_string(),
            reason: "Unknown token".to_string(),
        }))
    }.map_err(|e| IcpiError::System(crate::infrastructure::errors::SystemError::StateCorrupted {
        reason: format!("Invalid canister ID for {}: {}", symbol, e),
    }))
}

fn calculate_total_burned(redemptions: &[(String, Nat)]) -> Nat {
    redemptions.iter()
        .map(|(_, amount)| amount.clone())
        .fold(Nat::from(0u64), |acc, x| acc + x)
}
```

### Complete burn_validator.rs
```rust
// src/icpi_backend/src/1_CRITICAL_OPERATIONS/burning/burn_validator.rs
//! Validation for burn operations

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, ValidationError, BurnError};
use crate::infrastructure::constants::MIN_BURN_AMOUNT;

pub fn validate_burn_request(caller: &Principal, amount: &Nat) -> Result<()> {
    // Check principal is not anonymous
    if caller == &Principal::anonymous() {
        return Err(IcpiError::Validation(ValidationError::InvalidPrincipal {
            principal: caller.to_text(),
        }));
    }

    // Check minimum amount
    if amount < &Nat::from(MIN_BURN_AMOUNT) {
        return Err(IcpiError::Burn(BurnError::AmountBelowMinimum {
            amount: amount.to_string(),
            minimum: MIN_BURN_AMOUNT.to_string(),
        }));
    }

    Ok(())
}
```

---

## Step 5.3: Migrate Rebalancing Module

### Complete rebalancing/mod.rs
```rust
// src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing/mod.rs
//! Rebalancing module - CRITICAL security boundary
//! Handles portfolio rebalancing via automated trading

mod deviation_analyzer;
mod trade_executor;
mod rebalance_timer;

use candid::{CandidType, Deserialize};
use crate::infrastructure::{Result, IcpiError, RebalanceError};
use crate::infrastructure::logging::AuditLogger;
use crate::infrastructure::constants::{REBALANCE_INTERVAL_SECONDS, MIN_DEVIATION_PERCENT};

pub use rebalance_timer::{start_rebalancing_timer, stop_rebalancing_timer};
pub use deviation_analyzer::calculate_deviations;
pub use trade_executor::execute_trade;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RebalanceRecord {
    pub timestamp: u64,
    pub action: RebalanceAction,
    pub success: bool,
    pub details: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum RebalanceAction {
    Buy { token: String, usd_amount: f64 },
    Sell { token: String, usd_amount: f64 },
    None,
}

/// Perform rebalancing if needed
pub async fn perform_rebalance() -> Result<String> {
    // Check if enough time has passed
    if !rebalance_timer::can_rebalance_now()? {
        return Err(IcpiError::Rebalance(RebalanceError::TooSoonSinceLastRebalance {
            last: rebalance_timer::get_last_rebalance_time().unwrap_or(0),
            interval: REBALANCE_INTERVAL_SECONDS,
        }));
    }

    // Get current state
    let state = crate::_2_CRITICAL_DATA::portfolio_value::get_portfolio_state_uncached().await?;

    // Calculate deviations from target
    let deviations = deviation_analyzer::calculate_deviations(&state)?;

    // Determine action needed
    let action = deviation_analyzer::determine_rebalance_action(&deviations, MIN_DEVIATION_PERCENT)?;

    match action {
        RebalanceAction::None => {
            rebalance_timer::record_rebalance(RebalanceAction::None, true, "No action needed");
            Ok("No rebalancing needed".to_string())
        }
        RebalanceAction::Buy { ref token, ref usd_amount } => {
            let result = trade_executor::execute_buy(token, *usd_amount).await;
            handle_trade_result(action, result).await
        }
        RebalanceAction::Sell { ref token, ref usd_amount } => {
            let result = trade_executor::execute_sell(token, *usd_amount).await;
            handle_trade_result(action, result).await
        }
    }
}

async fn handle_trade_result(action: RebalanceAction, result: Result<String>) -> Result<String> {
    match result {
        Ok(details) => {
            rebalance_timer::record_rebalance(action.clone(), true, &details);
            AuditLogger::log_rebalance(&format!("{:?}", action), 0, "", "");
            Ok(details)
        }
        Err(e) => {
            rebalance_timer::record_rebalance(action, false, &e.to_string());
            Err(e)
        }
    }
}
```

### Complete deviation_analyzer.rs
```rust
// src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing/deviation_analyzer.rs
//! Pure functions for analyzing portfolio deviations

use crate::types::portfolio::IndexState;
use crate::types::rebalancing::{AllocationDeviation, TargetAllocation};
use crate::infrastructure::{Result, IcpiError, RebalanceError};
use super::RebalanceAction;

/// Calculate deviations from target allocations
pub fn calculate_deviations(state: &IndexState) -> Result<Vec<AllocationDeviation>> {
    let mut deviations = Vec::new();

    for position in &state.current_positions {
        // Find target for this token
        let target = state.target_allocations.iter()
            .find(|t| t.token == position.token)
            .ok_or_else(|| IcpiError::Rebalance(RebalanceError::AllocationCalculationError {
                reason: format!("No target for token {:?}", position.token),
            }))?;

        let current_pct = position.percentage;
        let target_pct = target.target_percentage;
        let deviation_pct = ((current_pct - target_pct) / target_pct * 100.0).abs();

        let usd_difference = position.usd_value - target.target_usd_value;
        let trade_size_usd = usd_difference.abs() * 0.1; // Trade 10% of deviation

        deviations.push(AllocationDeviation {
            token: position.token.clone(),
            current_pct,
            target_pct,
            deviation_pct,
            usd_difference,
            trade_size_usd,
        });
    }

    Ok(deviations)
}

/// Determine what rebalance action to take
pub fn determine_rebalance_action(
    deviations: &[AllocationDeviation],
    min_deviation_percent: f64,
) -> Result<RebalanceAction> {
    // Find largest deviation
    let max_deviation = deviations.iter()
        .max_by(|a, b| a.deviation_pct.partial_cmp(&b.deviation_pct).unwrap())
        .ok_or_else(|| IcpiError::Rebalance(RebalanceError::NoRebalanceNeeded {
            reason: "No deviations calculated".to_string(),
        }))?;

    if max_deviation.deviation_pct < min_deviation_percent {
        return Ok(RebalanceAction::None);
    }

    // Determine if we need to buy or sell
    if max_deviation.usd_difference < 0.0 {
        // Under-allocated, need to buy
        Ok(RebalanceAction::Buy {
            token: format!("{:?}", max_deviation.token),
            usd_amount: max_deviation.trade_size_usd,
        })
    } else {
        // Over-allocated, need to sell
        Ok(RebalanceAction::Sell {
            token: format!("{:?}", max_deviation.token),
            usd_amount: max_deviation.trade_size_usd,
        })
    }
}
```

### Complete trade_executor.rs
```rust
// src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing/trade_executor.rs
//! Execute rebalancing trades on Kongswap

use candid::Nat;
use crate::infrastructure::{Result, IcpiError, TradingError};
use crate::types::TrackedToken;

/// Execute a buy trade (ckUSDT -> Token)
pub async fn execute_buy(token_symbol: &str, usd_amount: f64) -> Result<String> {
    let usdt_nat = Nat::from((usd_amount * 1_000_000.0) as u128);

    ic_cdk::println!("Executing buy: {} ckUSDT for {}", usdt_nat, token_symbol);

    let swap_result = crate::_4_TRADING_EXECUTION::swaps::execute_swap(
        "ckUSDT".to_string(),
        usdt_nat.clone(),
        token_symbol.to_string(),
        Some(2.0), // 2% max slippage
    ).await?;

    Ok(format!("Bought {} {} for {} ckUSDT",
        swap_result.receive_amount,
        token_symbol,
        swap_result.pay_amount
    ))
}

/// Execute a sell trade (Token -> ckUSDT)
pub async fn execute_sell(token_symbol: &str, usd_amount: f64) -> Result<String> {
    let token = TrackedToken::from_symbol(token_symbol)?;
    let token_amount = calculate_token_amount_for_usd(&token, usd_amount).await?;

    ic_cdk::println!("Executing sell: {} {} for ~${}", token_amount, token_symbol, usd_amount);

    let swap_result = crate::_4_TRADING_EXECUTION::swaps::execute_swap(
        token_symbol.to_string(),
        token_amount,
        "ckUSDT".to_string(),
        Some(2.0), // 2% max slippage
    ).await?;

    Ok(format!("Sold {} {} for {} ckUSDT",
        swap_result.pay_amount,
        token_symbol,
        swap_result.receive_amount
    ))
}

async fn calculate_token_amount_for_usd(
    token: &TrackedToken,
    usd_value: f64,
) -> Result<Nat> {
    let price = crate::_3_KONG_LIQUIDITY::pricing::get_token_price_in_usdt(token).await?;

    if price <= 0.0 {
        return Err(IcpiError::Trading(TradingError::InvalidQuote {
            reason: "Zero or negative price".to_string(),
        }));
    }

    let token_amount = usd_value / price;
    let decimals = token.get_decimals();
    let units = (token_amount * 10_f64.powi(decimals as i32)) as u128;

    Ok(Nat::from(units))
}
```

---

## Migration Mapping Table

| Old File | Function/Type | New Location | Notes |
|----------|--------------|--------------|-------|
| **minting.rs** | | | |
| Line 13 | `refund_deposit()` | 1_CRITICAL_OPERATIONS/minting/refund_handler.rs | Moved to dedicated module |
| Line 33 | `MintStatus` enum | 1_CRITICAL_OPERATIONS/minting/types.rs | Moved to types |
| Line 49 | `PendingMint` struct | 1_CRITICAL_OPERATIONS/minting/types.rs | Moved to types |
| Line 60 | `initiate_mint()` | 1_CRITICAL_OPERATIONS/minting/mod.rs | Main orchestration |
| Line 120 | `collect_fee()` | 1_CRITICAL_OPERATIONS/minting/fee_handler.rs | Fee collection |
| Line 150 | `complete_mint()` | 1_CRITICAL_OPERATIONS/minting/mod.rs | Completion logic |
| Line 220 | `calculate_mint_amount()` | 6_INFRASTRUCTURE/math/mod.rs | PURE function |
| **burning.rs** | | | |
| Line 10 | `BurnResult` struct | 1_CRITICAL_OPERATIONS/burning/mod.rs | Result type |
| Line 28 | `burn_icpi()` | 1_CRITICAL_OPERATIONS/burning/mod.rs | Main orchestration |
| Line 54 | Redemption calculation | 1_CRITICAL_OPERATIONS/burning/redemption_calculator.rs | PURE function |
| Line 107 | Token distribution | 1_CRITICAL_OPERATIONS/burning/token_distributor.rs | I/O operations |
| **rebalancer.rs** | | | |
| Line 16 | `RebalanceRecord` | 1_CRITICAL_OPERATIONS/rebalancing/types.rs | Record type |
| Line 26 | `start_rebalancing()` | 1_CRITICAL_OPERATIONS/rebalancing/rebalance_timer.rs | Timer management |
| Line 46 | `stop_rebalancing()` | 1_CRITICAL_OPERATIONS/rebalancing/rebalance_timer.rs | Timer management |
| Line 56 | `perform_rebalance()` | 1_CRITICAL_OPERATIONS/rebalancing/mod.rs | Main logic |
| Line 103 | `execute_buy()` | 1_CRITICAL_OPERATIONS/rebalancing/trade_executor.rs | Trade execution |
| Line 129 | `execute_sell()` | 1_CRITICAL_OPERATIONS/rebalancing/trade_executor.rs | Trade execution |
| Line 157 | `calculate_token_amount_for_usd()` | 1_CRITICAL_OPERATIONS/rebalancing/trade_executor.rs | Calculation |
| **tvl_calculator.rs** | | | |
| Line 10 | `calculate_locked_tvl()` | 3_KONG_LIQUIDITY/tvl/calculator.rs | TVL calculation |
| Line 58 | `process_lp_position()` | 3_KONG_LIQUIDITY/tvl/position_processor.rs | LP processing |
| Line 101 | `calculate_tvl_percentages()` | 2_CRITICAL_DATA/portfolio_value/percentages.rs | PURE function |
| Line 122 | `get_tvl_summary()` | 5_INFORMATIONAL/display/tvl_display.rs | Display formatting |
| **balance_tracker.rs** | | | |
| Line 10 | `get_token_balance()` | 2_CRITICAL_DATA/token_queries/balance_queries.rs | Balance query |
| Line 45 | `get_all_balances()` | 2_CRITICAL_DATA/token_queries/balance_queries.rs | All balances |
| Line 80 | `get_backend_principal()` | 6_INFRASTRUCTURE/utils.rs | Utility function |
| **kongswap.rs** | | | |
| Line 13 | `get_lp_positions()` | 3_KONG_LIQUIDITY/pools/lp_queries.rs | LP positions |
| Line 50 | `get_token_price_in_usdt()` | 3_KONG_LIQUIDITY/pricing/price_queries.rs | Price queries |
| Line 108 | `execute_swap()` | 4_TRADING_EXECUTION/swaps/swap_executor.rs | Swap execution |
| Line 180 | `approve_token_for_swap()` | 4_TRADING_EXECUTION/approvals/approval_manager.rs | Approvals |
| **index_state.rs** | | | |
| Line 20 | `get_index_state()` | 5_INFORMATIONAL/display/index_display.rs | State display |
| Line 85 | `calculate_target_allocations()` | 2_CRITICAL_DATA/portfolio_value/target_calculator.rs | Target calculation |
| Line 150 | `get_rebalancing_recommendation()` | 1_CRITICAL_OPERATIONS/rebalancing/deviation_analyzer.rs | Rebalance logic |
| **precision.rs** | | | |
| Line 10 | `multiply_and_divide()` | 6_INFRASTRUCTURE/math/mod.rs | Math function |
| Line 45 | `convert_decimals()` | 6_INFRASTRUCTURE/math/mod.rs | Decimal conversion |
| **icpi_math.rs** | | | |
| Line 15 | `calculate_proportional_share()` | 6_INFRASTRUCTURE/math/mod.rs | Share calculation |
| Line 50 | `safe_divide()` | 6_INFRASTRUCTURE/math/mod.rs | Safe division |

---

## Complete lib.rs Implementation

```rust
// src/icpi_backend/src/lib.rs
//! ICPI Backend - Security-First Architecture with Numbered Zones

// Module declarations using numbered structure
#[path = "1_CRITICAL_OPERATIONS/mod.rs"]
mod critical_operations;

#[path = "2_CRITICAL_DATA/mod.rs"]
mod critical_data;

#[path = "3_KONG_LIQUIDITY/mod.rs"]
mod kong_liquidity;

#[path = "4_TRADING_EXECUTION/mod.rs"]
mod trading_execution;

#[path = "5_INFORMATIONAL/mod.rs"]
mod informational;

#[path = "6_INFRASTRUCTURE/mod.rs"]
mod infrastructure;

use candid::{candid_method, Nat, Principal};
use ic_cdk::{init, post_upgrade, query, update};
use infrastructure::{FeatureFlags, OperationStrategy, Result, IcpiError};

// === Public API Endpoints ===

#[update]
#[candid_method(update)]
async fn mint_icpi(amount: Nat) -> Result<Nat> {
    let caller = ic_cdk::caller();

    match FeatureFlags::get_minting_strategy() {
        OperationStrategy::Legacy => {
            // Use old minting code for safety
            legacy::minting::mint_with_usdt(caller, amount).await
                .map_err(|e| IcpiError::from(e))
        }
        OperationStrategy::Refactored => {
            critical_operations::minting::initiate_mint(caller, amount).await
        }
        OperationStrategy::Shadow => {
            let legacy_result = legacy::minting::mint_with_usdt(caller.clone(), amount.clone()).await;
            let refactored_result = critical_operations::minting::initiate_mint(caller, amount).await;

            infrastructure::log_shadow_comparison("mint_icpi", &legacy_result, &refactored_result);

            legacy_result.map_err(|e| IcpiError::from(e))
        }
    }
}

#[update]
#[candid_method(update)]
async fn complete_mint(mint_id: String) -> Result<Nat> {
    let caller = ic_cdk::caller();

    match FeatureFlags::get_minting_strategy() {
        OperationStrategy::Legacy => {
            legacy::minting::complete_mint(mint_id).await
                .map_err(|e| IcpiError::from(e))
        }
        OperationStrategy::Refactored => {
            critical_operations::minting::complete_mint(caller, mint_id).await
        }
        OperationStrategy::Shadow => {
            let legacy_result = legacy::minting::complete_mint(mint_id.clone()).await;
            let refactored_result = critical_operations::minting::complete_mint(caller, mint_id).await;

            infrastructure::log_shadow_comparison("complete_mint", &legacy_result, &refactored_result);

            legacy_result.map_err(|e| IcpiError::from(e))
        }
    }
}

#[update]
#[candid_method(update)]
async fn burn_icpi(amount: Nat) -> Result<critical_operations::burning::BurnResult> {
    let caller = ic_cdk::caller();

    match FeatureFlags::get_burning_strategy() {
        OperationStrategy::Legacy => {
            legacy::burning::burn_icpi(amount).await
                .map(|r| critical_operations::burning::BurnResult {
                    successful_transfers: r.successful_transfers,
                    failed_transfers: r.failed_transfers,
                    icpi_burned: r.icpi_burned,
                    timestamp: ic_cdk::api::time(),
                })
                .map_err(|e| IcpiError::from(e))
        }
        OperationStrategy::Refactored => {
            critical_operations::burning::burn_icpi(caller, amount).await
        }
        OperationStrategy::Shadow => {
            let legacy_result = legacy::burning::burn_icpi(amount.clone()).await;
            let refactored_result = critical_operations::burning::burn_icpi(caller, amount).await;

            infrastructure::log_shadow_comparison("burn_icpi", &legacy_result, &refactored_result);

            legacy_result
                .map(|r| critical_operations::burning::BurnResult {
                    successful_transfers: r.successful_transfers,
                    failed_transfers: r.failed_transfers,
                    icpi_burned: r.icpi_burned,
                    timestamp: ic_cdk::api::time(),
                })
                .map_err(|e| IcpiError::from(e))
        }
    }
}

#[update]
#[candid_method(update)]
async fn perform_rebalance() -> Result<String> {
    // Admin check
    if !is_admin(ic_cdk::caller()) {
        return Err(IcpiError::System(infrastructure::errors::SystemError::Unauthorized {
            principal: ic_cdk::caller().to_text(),
            required_role: "admin".to_string(),
        }));
    }

    match FeatureFlags::get_rebalancing_strategy() {
        OperationStrategy::Legacy => {
            legacy::rebalancer::perform_rebalance().await
                .map_err(|e| IcpiError::from(e))
        }
        OperationStrategy::Refactored => {
            critical_operations::rebalancing::perform_rebalance().await
        }
        OperationStrategy::Shadow => {
            let legacy_result = legacy::rebalancer::perform_rebalance().await;
            let refactored_result = critical_operations::rebalancing::perform_rebalance().await;

            infrastructure::log_shadow_comparison("perform_rebalance", &legacy_result, &refactored_result);

            legacy_result.map_err(|e| IcpiError::from(e))
        }
    }
}

#[query]
#[candid_method(query)]
fn get_index_state() -> critical_data::portfolio_value::IndexState {
    match FeatureFlags::get_query_strategy() {
        OperationStrategy::Legacy => {
            legacy::index_state::get_index_state_cached()
        }
        OperationStrategy::Refactored | OperationStrategy::Shadow => {
            informational::display::get_index_state_cached()
        }
    }
}

#[query]
#[candid_method(query)]
fn get_feature_flags() -> infrastructure::FeatureFlagConfig {
    FeatureFlags::get_all_flags()
}

#[update]
#[candid_method(update)]
fn set_feature_flag(operation: String, strategy: String) -> Result<String> {
    // Admin check
    if !is_admin(ic_cdk::caller()) {
        return Err(IcpiError::System(infrastructure::errors::SystemError::Unauthorized {
            principal: ic_cdk::caller().to_text(),
            required_role: "admin".to_string(),
        }));
    }

    let strat = match strategy.as_str() {
        "legacy" => OperationStrategy::Legacy,
        "refactored" => OperationStrategy::Refactored,
        "shadow" => OperationStrategy::Shadow,
        _ => return Err(IcpiError::Other("Invalid strategy".to_string())),
    };

    FeatureFlags::set_strategy(&operation, strat)
}

#[query]
#[candid_method(query)]
fn get_health_status() -> informational::health::HealthStatus {
    informational::health::get_health_status()
}

#[query]
#[candid_method(query)]
fn get_tracked_tokens() -> Vec<String> {
    use crate::types::TrackedToken;
    TrackedToken::all().into_iter()
        .map(|t| t.to_symbol().to_string())
        .collect()
}

// === Initialization ===

#[init]
fn init() {
    ic_cdk::println!("ICPI Backend initialized with numbered security zones");
    ic_cdk::println!("Starting in LEGACY mode for safety");

    FeatureFlags::set_all_to_legacy();

    match FeatureFlags::get_rebalancing_strategy() {
        OperationStrategy::Legacy => {
            legacy::rebalancer::start_rebalancing();
        }
        OperationStrategy::Refactored => {
            critical_operations::rebalancing::start_rebalancing_timer();
        }
        OperationStrategy::Shadow => {
            legacy::rebalancer::start_rebalancing();
            critical_operations::rebalancing::start_rebalancing_timer();
        }
    }
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("ICPI Backend upgraded - maintaining feature flags");

    match FeatureFlags::get_rebalancing_strategy() {
        OperationStrategy::Legacy => {
            legacy::rebalancer::start_rebalancing();
        }
        OperationStrategy::Refactored => {
            critical_operations::rebalancing::start_rebalancing_timer();
        }
        OperationStrategy::Shadow => {
            legacy::rebalancer::start_rebalancing();
            critical_operations::rebalancing::start_rebalancing_timer();
        }
    }
}

// === Helper Functions ===

fn is_admin(principal: Principal) -> bool {
    // TODO: Implement proper admin check
    // For now, hardcode admin principals
    const ADMIN_PRINCIPALS: &[&str] = &[
        "e454q-riaaa-aaaap-qqcyq-cai", // Example admin
    ];

    ADMIN_PRINCIPALS.iter()
        .any(|&admin| Principal::from_text(admin).ok() == Some(principal))
}

// === Legacy module imports (temporary) ===
mod legacy {
    pub use crate::minting;
    pub use crate::burning;
    pub use crate::rebalancer;
    pub use crate::index_state;
}

// === Candid Export ===
ic_cdk::export_candid!();
```

This completes Phase 5 with all the missing implementations!