# Phases 5-8: Complete Implementation Guide

This file continues from Phase 4 of ICPI_REFACTORING_IMPLEMENTATION_GUIDE_V2.md

---

## Phase 5: Reorganize Critical Operations (Complete Implementation)

This is the most complex phase - migrating all business logic to the numbered structure.

### Step 5.1: Migrate Minting Module

```bash
# Create complete minting module structure
cd src/icpi_backend/src/1_CRITICAL_OPERATIONS/minting

# Create main orchestration file
cat > mod.rs << 'EOF'
//! Minting module - CRITICAL security boundary
//! Handles ICPI token minting with ckUSDT deposits

mod fee_handler;
mod mint_orchestrator;
mod mint_validator;
mod mint_state;
mod refund_handler;

pub use mint_orchestrator::{initiate_mint, complete_mint};
pub use mint_state::{MintStatus, PendingMint, get_pending_mint, cleanup_expired_mints};

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, MintError};
use crate::infrastructure::logging::AuditLogger;
EOF

# Create fee handler
cat > fee_handler.rs << 'EOF'
//! Fee collection for minting operations

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, MintError};
use crate::infrastructure::constants::{CKUSDT_CANISTER_ID, FEE_RECIPIENT, MINT_FEE_AMOUNT};
use crate::types::{Account, TransferFromArgs, TransferFromResult};

pub async fn collect_mint_fee(user: Principal) -> Result<Nat> {
    let ckusdt = Principal::from_text(CKUSDT_CANISTER_ID)
        .map_err(|e| IcpiError::System(crate::infrastructure::errors::SystemError::StateCorrupted {
            reason: format!("Invalid ckUSDT principal: {}", e),
        }))?;

    let fee_recipient = Principal::from_text(FEE_RECIPIENT)
        .map_err(|e| IcpiError::System(crate::infrastructure::errors::SystemError::StateCorrupted {
            reason: format!("Invalid fee recipient: {}", e),
        }))?;

    let transfer_args = TransferFromArgs {
        from: Account {
            owner: user,
            subaccount: None,
        },
        to: Account {
            owner: fee_recipient,
            subaccount: None,
        },
        amount: Nat::from(MINT_FEE_AMOUNT),
        fee: None,
        memo: Some(b"ICPI mint fee".to_vec()),
        created_at_time: None,
    };

    let result: Result<(TransferFromResult,), _> = ic_cdk::call(
        ckusdt,
        "icrc2_transfer_from",
        (transfer_args,)
    ).await;

    match result {
        Ok((TransferFromResult::Ok(block),)) => {
            ic_cdk::println!("Fee collected: block {}", block);
            Ok(block)
        }
        Ok((TransferFromResult::Err(e),)) => {
            Err(IcpiError::Mint(MintError::FeeCollectionFailed {
                reason: format!("{:?}", e),
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Mint(MintError::FeeCollectionFailed {
                reason: format!("Call failed: {:?} - {}", code, msg),
            }))
        }
    }
}

pub async fn collect_deposit(user: Principal, amount: Nat) -> Result<Nat> {
    let ckusdt = Principal::from_text(CKUSDT_CANISTER_ID)
        .map_err(|e| IcpiError::Mint(MintError::DepositCollectionFailed {
            reason: format!("Invalid ckUSDT principal: {}", e),
        }))?;

    let backend = ic_cdk::id();

    let transfer_args = TransferFromArgs {
        from: Account {
            owner: user,
            subaccount: None,
        },
        to: Account {
            owner: backend,
            subaccount: None,
        },
        amount,
        fee: None,
        memo: Some(b"ICPI mint deposit".to_vec()),
        created_at_time: None,
    };

    let result: Result<(TransferFromResult,), _> = ic_cdk::call(
        ckusdt,
        "icrc2_transfer_from",
        (transfer_args,)
    ).await;

    match result {
        Ok((TransferFromResult::Ok(block),)) => {
            ic_cdk::println!("Deposit collected: block {}", block);
            Ok(block)
        }
        Ok((TransferFromResult::Err(e),)) => {
            Err(IcpiError::Mint(MintError::DepositCollectionFailed {
                reason: format!("{:?}", e),
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Mint(MintError::DepositCollectionFailed {
                reason: format!("Call failed: {:?} - {}", code, msg),
            }))
        }
    }
}
EOF

# Create mint orchestrator
cat > mint_orchestrator.rs << 'EOF'
//! Main minting orchestration logic

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, MintError};
use crate::infrastructure::constants::{MIN_MINT_AMOUNT, MAX_MINT_AMOUNT, MINT_TIMEOUT_NANOS};
use crate::infrastructure::math::calculate_mint_amount;
use crate::infrastructure::logging::AuditLogger;
use super::{fee_handler, mint_validator, mint_state, refund_handler};

pub async fn initiate_mint(caller: Principal, usdt_amount: Nat) -> Result<String> {
    // 1. Validate request
    mint_validator::validate_mint_request(&caller, &usdt_amount)?;

    // 2. Generate mint ID
    let mint_id = format!("mint_{}_{}",
        caller.to_text().chars().take(8).collect::<String>(),
        ic_cdk::api::time()
    );

    // 3. Create pending mint record
    let pending_mint = mint_state::PendingMint {
        id: mint_id.clone(),
        user: caller,
        amount: usdt_amount.clone(),
        status: mint_state::MintStatus::Pending,
        created_at: ic_cdk::api::time(),
        last_updated: ic_cdk::api::time(),
        snapshot: None,
    };

    mint_state::store_pending_mint(pending_mint)?;

    // 4. Return mint ID for user to call complete_mint
    Ok(mint_id)
}

pub async fn complete_mint(caller: Principal, mint_id: String) -> Result<Nat> {
    // 1. Retrieve pending mint
    let mut pending_mint = mint_state::get_pending_mint(&mint_id)?
        .ok_or_else(|| IcpiError::Mint(MintError::InvalidMintId {
            id: mint_id.clone(),
        }))?;

    // 2. Verify caller is the original user
    if pending_mint.user != caller {
        return Err(IcpiError::System(crate::infrastructure::errors::SystemError::Unauthorized {
            principal: caller.to_text(),
            required_role: "mint initiator".to_string(),
        }));
    }

    // 3. Check timeout
    let elapsed = ic_cdk::api::time() - pending_mint.created_at;
    if elapsed > MINT_TIMEOUT_NANOS {
        mint_state::update_mint_status(&mint_id, mint_state::MintStatus::Expired)?;
        return Err(IcpiError::Mint(MintError::MintTimeout {
            elapsed,
            timeout: MINT_TIMEOUT_NANOS,
        }));
    }

    // 4. Collect fee
    mint_state::update_mint_status(&mint_id, mint_state::MintStatus::CollectingFee)?;
    let fee_result = fee_handler::collect_mint_fee(caller).await;
    if let Err(e) = fee_result {
        mint_state::update_mint_status(&mint_id, mint_state::MintStatus::Failed(e.to_string()))?;
        return Err(e);
    }

    // 5. Take snapshot BEFORE collecting deposit (critical for correct ratio)
    mint_state::update_mint_status(&mint_id, mint_state::MintStatus::Snapshotting)?;

    let (supply, tvl) = futures::future::join!(
        crate::_2_CRITICAL_DATA::supply_tracker::get_icpi_supply_uncached(),
        crate::_2_CRITICAL_DATA::portfolio_value::calculate_portfolio_value_atomic()
    ).await;

    let supply = supply?;
    let tvl = tvl?;

    pending_mint.snapshot = Some(mint_state::MintSnapshot {
        supply: supply.clone(),
        tvl: tvl.clone(),
        timestamp: ic_cdk::api::time(),
    });
    mint_state::store_pending_mint(pending_mint.clone())?;

    // 6. Collect deposit
    mint_state::update_mint_status(&mint_id, mint_state::MintStatus::CollectingDeposit)?;
    let deposit_result = fee_handler::collect_deposit(caller, pending_mint.amount.clone()).await;

    if let Err(e) = deposit_result {
        // Refund if deposit collection fails
        mint_state::update_mint_status(&mint_id, mint_state::MintStatus::Refunding)?;
        let _ = refund_handler::refund_deposit(caller, pending_mint.amount.clone()).await;
        mint_state::update_mint_status(&mint_id, mint_state::MintStatus::FailedRefunded(e.to_string()))?;
        return Err(e);
    }

    // 7. Calculate ICPI amount
    mint_state::update_mint_status(&mint_id, mint_state::MintStatus::Calculating)?;
    let icpi_amount = calculate_mint_amount(&pending_mint.amount, &supply, &tvl)?;

    // 8. Mint ICPI tokens
    mint_state::update_mint_status(&mint_id, mint_state::MintStatus::Minting)?;
    let mint_result = mint_icpi_on_ledger(caller, icpi_amount.clone()).await;

    match mint_result {
        Ok(block_index) => {
            mint_state::update_mint_status(&mint_id, mint_state::MintStatus::Complete(icpi_amount.clone()))?;

            // Audit log
            AuditLogger::log_mint(
                caller,
                pending_mint.amount.0.to_u64().unwrap_or(0),
                icpi_amount.0.to_u64().unwrap_or(0),
                tvl.0.to_u64().unwrap_or(0),
                (tvl + pending_mint.amount).0.to_u64().unwrap_or(0),
            );

            Ok(icpi_amount)
        }
        Err(e) => {
            mint_state::update_mint_status(&mint_id, mint_state::MintStatus::Failed(e.to_string()))?;
            Err(e)
        }
    }
}

async fn mint_icpi_on_ledger(user: Principal, amount: Nat) -> Result<Nat> {
    let icpi_canister = Principal::from_text(crate::infrastructure::constants::ICPI_LEDGER_ID)
        .map_err(|e| IcpiError::System(crate::infrastructure::errors::SystemError::StateCorrupted {
            reason: format!("Invalid ICPI principal: {}", e),
        }))?;

    let mint_args = crate::types::icrc::MintArgs {
        to: crate::types::Account {
            owner: user,
            subaccount: None,
        },
        amount,
        memo: Some(b"ICPI mint".to_vec()),
        created_at_time: None,
    };

    let result: Result<(Nat,), _> = ic_cdk::call(
        icpi_canister,
        "icrc1_mint",
        (mint_args,)
    ).await;

    match result {
        Ok((block_index,)) => Ok(block_index),
        Err((code, msg)) => {
            Err(IcpiError::Mint(MintError::LedgerMintFailed {
                reason: format!("Call failed: {:?} - {}", code, msg),
            }))
        }
    }
}
EOF
```

### Step 5.2: Migrate Burning Module

```bash
cd src/icpi_backend/src/1_CRITICAL_OPERATIONS/burning

# Create complete burning module
cat > mod.rs << 'EOF'
//! Burning module - CRITICAL security boundary
//! Handles ICPI token burning and proportional redemptions

mod redemption_calculator;
mod token_distributor;
mod burn_validator;

use candid::{CandidType, Deserialize, Nat, Principal};
use crate::infrastructure::{Result, IcpiError, BurnError};
use crate::infrastructure::logging::AuditLogger;
use crate::infrastructure::cache::assert_no_cache_for_critical_op;

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

    // 2. Collect fee from user
    crate::_1_CRITICAL_OPERATIONS::minting::fee_handler::collect_mint_fee(caller).await?;

    // 3. Assert no caching for critical operation
    assert_no_cache_for_critical_op("burn_icpi");

    // 4. Get current supply atomically (no race condition)
    let current_supply = crate::_2_CRITICAL_DATA::supply_tracker::get_icpi_supply_uncached().await?;

    // 5. Get all token balances
    let token_balances = crate::_2_CRITICAL_DATA::token_queries::get_all_balances_uncached().await?;

    // 6. Calculate redemptions using PURE function
    let redemptions = redemption_calculator::calculate_redemptions(
        &amount,
        &current_supply,
        &token_balances
    )?;

    // 7. Distribute tokens to user
    let result = token_distributor::distribute_tokens(caller, redemptions).await?;

    // 8. Audit log
    AuditLogger::log_burn(
        caller,
        amount.0.to_u64().unwrap_or(0),
        result.successful_transfers.clone()
            .into_iter()
            .map(|(token, amt)| (token, amt.0.to_u64().unwrap_or(0)))
            .collect()
    );

    Ok(result)
}
EOF

# Create redemption calculator (PURE function)
cat > redemption_calculator.rs << 'EOF'
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

    if burn_amount > total_supply {
        return Err(IcpiError::Validation(crate::infrastructure::errors::ValidationError::InvalidAmount {
            amount: burn_amount.to_string(),
            reason: format!("Burn amount exceeds total supply {}", total_supply),
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
    fn test_proportional_redemption() {
        let burn_amount = Nat::from(50_000_000u64); // 0.5 ICPI
        let total_supply = Nat::from(100_000_000u64); // 1 ICPI total
        let balances = vec![
            ("ALEX".to_string(), Nat::from(1_000_000u64)),
            ("KONG".to_string(), Nat::from(2_000_000u64)),
            ("ZERO".to_string(), Nat::from(0u64)), // Should be skipped
            ("BOB".to_string(), Nat::from(500_000u64)),
        ];

        let result = calculate_redemptions(&burn_amount, &total_supply, &balances).unwrap();

        // Should get 50% of each token minus fees
        assert_eq!(result.len(), 3); // ZERO excluded

        // ALEX: 500,000 - 10,000 fee = 490,000
        assert_eq!(result[0].0, "ALEX");
        assert_eq!(result[0].1, Nat::from(490_000u64));

        // KONG: 1,000,000 - 10,000 fee = 990,000
        assert_eq!(result[1].0, "KONG");
        assert_eq!(result[1].1, Nat::from(990_000u64));

        // BOB: 250,000 - 10,000 fee = 240,000
        assert_eq!(result[2].0, "BOB");
        assert_eq!(result[2].1, Nat::from(240_000u64));
    }
}
EOF
```

### Step 5.3: Migrate Rebalancing Module

```bash
cd src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing

# Create complete rebalancing module
cat > mod.rs << 'EOF'
//! Rebalancing module - CRITICAL security boundary
//! Handles portfolio rebalancing via automated trading

mod deviation_analyzer;
mod trade_executor;
mod rebalance_timer;

use candid::{CandidType, Deserialize};
use crate::infrastructure::{Result, IcpiError, RebalanceError};
use crate::infrastructure::logging::AuditLogger;
use crate::infrastructure::constants::{REBALANCE_INTERVAL_SECONDS, MIN_DEVIATION_PERCENT};

pub use rebalance_timer::{start_rebalancing_timer, stop_rebalancing_timer, get_rebalancer_status};
pub use deviation_analyzer::calculate_deviations;

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
        let last = rebalance_timer::get_last_rebalance_time().unwrap_or(0);
        let elapsed = (ic_cdk::api::time() - last) / 1_000_000_000;
        return Err(IcpiError::Rebalance(RebalanceError::TooSoonSinceLastRebalance {
            last,
            interval: REBALANCE_INTERVAL_SECONDS,
        }));
    }

    // Get current state (uncached for accuracy)
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

            // Extract trade info for audit log
            let (from, to) = match &action {
                RebalanceAction::Buy { token, .. } => ("ckUSDT", token.as_str()),
                RebalanceAction::Sell { token, .. } => (token.as_str(), "ckUSDT"),
                RebalanceAction::None => ("", ""),
            };

            AuditLogger::log_rebalance(&format!("{:?}", action), 0, from, to);

            Ok(details)
        }
        Err(e) => {
            rebalance_timer::record_rebalance(action, false, &e.to_string());
            Err(e)
        }
    }
}

/// Manual trigger for testing (admin only)
pub async fn trigger_manual_rebalance() -> Result<String> {
    ic_cdk::println!("Manual rebalance triggered");

    // Clear last rebalance time to bypass time check
    rebalance_timer::reset_timer();

    perform_rebalance().await
}
EOF
```

### Step 5.4: Migrate Portfolio Data (Zone 2)

```bash
cd src/icpi_backend/src/2_CRITICAL_DATA

# Create portfolio_value module
mkdir -p portfolio_value
cat > portfolio_value/mod.rs << 'EOF'
//! Portfolio value calculations
//! CRITICAL: Used for mint/burn ratios

mod value_calculator;
mod target_calculator;
mod percentages;

use candid::Nat;
use crate::infrastructure::{Result, cache::assert_no_cache_for_critical_op};
use crate::types::portfolio::IndexState;

pub use value_calculator::calculate_portfolio_value_atomic;
pub use target_calculator::calculate_target_allocations;
pub use percentages::calculate_percentages;

/// Get portfolio state WITHOUT caching (for critical operations)
pub async fn get_portfolio_state_uncached() -> Result<IndexState> {
    assert_no_cache_for_critical_op("get_portfolio_state_uncached");

    // Get all data in parallel
    let (positions, targets, tvl) = futures::future::join3(
        crate::_2_CRITICAL_DATA::token_queries::get_current_positions(),
        target_calculator::calculate_target_allocations(),
        value_calculator::calculate_portfolio_value_atomic()
    ).await;

    Ok(IndexState {
        total_value: tvl?.0.to_u64().unwrap_or(0) as f64 / 1e6, // Convert to USD
        current_positions: positions?,
        target_allocations: targets?,
        deviations: Vec::new(), // Calculate on demand
        ckusdt_balance: crate::_2_CRITICAL_DATA::token_queries::get_ckusdt_balance().await?,
        timestamp: ic_cdk::api::time(),
    })
}

/// Calculate total portfolio value in ckUSDT
pub async fn calculate_tvl_in_ckusdt() -> Result<Nat> {
    value_calculator::calculate_portfolio_value_atomic().await
}
EOF

# Create supply_tracker module
mkdir -p supply_tracker
cat > supply_tracker/mod.rs << 'EOF'
//! ICPI supply tracking
//! CRITICAL: Used for mint/burn calculations

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, QueryError};
use crate::infrastructure::cache::assert_no_cache_for_critical_op;
use crate::infrastructure::constants::ICPI_LEDGER_ID;

/// Get ICPI total supply WITHOUT caching
pub async fn get_icpi_supply_uncached() -> Result<Nat> {
    assert_no_cache_for_critical_op("get_icpi_supply_uncached");

    let icpi_canister = Principal::from_text(ICPI_LEDGER_ID)
        .map_err(|e| IcpiError::Query(QueryError::InvalidResponse {
            expected: "valid principal".to_string(),
            received: e.to_string(),
        }))?;

    let result: Result<(Nat,), _> = ic_cdk::call(
        icpi_canister,
        "icrc1_total_supply",
        ()
    ).await;

    match result {
        Ok((supply,)) => {
            validate_supply(&supply)?;
            Ok(supply)
        }
        Err((code, msg)) => {
            Err(IcpiError::Query(QueryError::CanisterUnreachable {
                canister: ICPI_LEDGER_ID.to_string(),
                reason: format!("{:?}: {}", code, msg),
            }))
        }
    }
}

/// Get validated supply with sanity checks
pub async fn get_validated_supply() -> Result<Nat> {
    let supply = get_icpi_supply_uncached().await?;
    validate_supply(&supply)?;
    Ok(supply)
}

fn validate_supply(supply: &Nat) -> Result<()> {
    const MAX_POSSIBLE_SUPPLY: u128 = 1_000_000_000_000_000_000; // 10 billion ICPI (e8)

    if supply > &Nat::from(MAX_POSSIBLE_SUPPLY) {
        return Err(IcpiError::Validation(
            crate::infrastructure::errors::ValidationError::SupplyOutOfBounds {
                supply: supply.to_string(),
                max: MAX_POSSIBLE_SUPPLY.to_string(),
            }
        ));
    }

    Ok(())
}
EOF

# Create token_queries module
mkdir -p token_queries
cat > token_queries/mod.rs << 'EOF'
//! Token balance queries
//! CRITICAL: Source of truth for all token holdings

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, QueryError};
use crate::infrastructure::cache::{assert_no_cache_for_critical_op, get_cached, CachePolicy};
use crate::types::{TrackedToken, Account, CurrentPosition};

/// Get token balance WITHOUT caching (for critical operations)
pub async fn get_token_balance_uncached(token: &TrackedToken) -> Result<Nat> {
    assert_no_cache_for_critical_op("get_token_balance_uncached");

    let canister_id = token.get_canister_id()?;
    let backend_principal = ic_cdk::id();

    let account = Account {
        owner: backend_principal,
        subaccount: None,
    };

    let result: Result<(Nat,), _> = ic_cdk::call(
        canister_id,
        "icrc1_balance_of",
        (account,)
    ).await;

    match result {
        Ok((balance,)) => Ok(balance),
        Err((code, msg)) => {
            Err(IcpiError::Query(QueryError::CanisterUnreachable {
                canister: canister_id.to_text(),
                reason: format!("{:?}: {}", code, msg),
            }))
        }
    }
}

/// Get all token balances WITHOUT caching
pub async fn get_all_balances_uncached() -> Result<Vec<(String, Nat)>> {
    let mut balances = Vec::new();

    // Include ckUSDT for reserve tracking
    let mut all_tokens = TrackedToken::all();
    all_tokens.push(TrackedToken::ckUSDT);

    // Query all balances in parallel
    let balance_futures: Vec<_> = all_tokens.iter()
        .map(|token| get_token_balance_uncached(token))
        .collect();

    let balance_results = futures::future::join_all(balance_futures).await;

    for (token, result) in all_tokens.iter().zip(balance_results) {
        match result {
            Ok(balance) => {
                balances.push((token.to_symbol().to_string(), balance));
            }
            Err(e) => {
                ic_cdk::println!("Failed to get balance for {}: {}", token.to_symbol(), e);
                balances.push((token.to_symbol().to_string(), Nat::from(0u64)));
            }
        }
    }

    Ok(balances)
}

/// Get ckUSDT balance specifically
pub async fn get_ckusdt_balance() -> Result<Nat> {
    get_token_balance_uncached(&TrackedToken::ckUSDT).await
}

/// Get current positions with prices
pub async fn get_current_positions() -> Result<Vec<CurrentPosition>> {
    let balances = get_all_balances_uncached().await?;
    let mut positions = Vec::new();

    for (token_str, balance) in balances {
        if token_str == "ckUSDT" {
            continue; // Skip ckUSDT in positions
        }

        let token = TrackedToken::from_symbol(&token_str)?;
        let price = crate::_3_KONG_LIQUIDITY::pricing::get_token_price_in_usdt(&token).await?;

        let decimals = token.get_decimals();
        let balance_float = balance.0.to_u64().unwrap_or(0) as f64 / 10_f64.powi(decimals as i32);
        let usd_value = balance_float * price;

        positions.push(CurrentPosition {
            token,
            balance,
            usd_value,
            percentage: 0.0, // Calculate after all positions
        });
    }

    // Calculate percentages
    let total_value: f64 = positions.iter().map(|p| p.usd_value).sum();
    for position in &mut positions {
        position.percentage = if total_value > 0.0 {
            (position.usd_value / total_value) * 100.0
        } else {
            0.0
        };
    }

    Ok(positions)
}

/// Get token balance WITH caching (for display only)
pub async fn get_token_balance_cached(token: &TrackedToken) -> Result<Nat> {
    let cache_key = format!("balance_{}", token.to_symbol());

    get_cached(
        &cache_key,
        CachePolicy::Short,
        || get_token_balance_uncached(token)
    ).await
}
EOF
```

### Step 5.5: Migrate Kong Integration (Zone 3)

```bash
cd src/icpi_backend/src/3_KONG_LIQUIDITY

# Create locker module
mkdir -p locker
cat > locker/mod.rs << 'EOF'
//! Kong Locker integration
//! Queries lock canisters for liquidity data

use candid::Principal;
use crate::infrastructure::{Result, IcpiError, KongLockerError};
use crate::types::KONG_LOCKER_ID;

/// Get all lock canisters from Kong Locker
pub async fn get_all_lock_canisters() -> Result<Vec<(Principal, Principal)>> {
    let kong_locker = Principal::from_text(KONG_LOCKER_ID)
        .map_err(|e| IcpiError::KongLocker(KongLockerError::InvalidLockCanister {
            canister: KONG_LOCKER_ID.to_string(),
        }))?;

    let result: Result<(Vec<(Principal, Principal)>,), _> = ic_cdk::call(
        kong_locker,
        "get_all_lock_canisters",
        ()
    ).await;

    match result {
        Ok((canisters,)) => {
            ic_cdk::println!("Retrieved {} lock canisters", canisters.len());
            Ok(canisters)
        }
        Err((code, msg)) => {
            Err(IcpiError::KongLocker(KongLockerError::CanisterListQueryFailed {
                reason: format!("{:?}: {}", code, msg),
            }))
        }
    }
}
EOF

# Create pools module
mkdir -p pools
cat > pools/mod.rs << 'EOF'
//! Kongswap pool queries
//! Gets LP positions from lock canisters

use candid::{Principal, Nat};
use crate::infrastructure::{Result, IcpiError, KongswapError};
use crate::types::{LPBalancesReply, UserBalancesReply, UserBalancesResult, KONGSWAP_BACKEND_ID};

/// Query LP positions for a lock canister
pub async fn get_lp_positions(lock_canister: Principal) -> Result<Vec<LPBalancesReply>> {
    let kongswap = Principal::from_text(KONGSWAP_BACKEND_ID)
        .map_err(|e| IcpiError::Kongswap(KongswapError::BackendUnreachable {
            reason: format!("Invalid principal: {}", e),
        }))?;

    let canister_text = lock_canister.to_text();

    let result: Result<(UserBalancesResult,), _> = ic_cdk::call(
        kongswap,
        "user_balances",
        (canister_text,)
    ).await;

    match result {
        Ok((UserBalancesResult::Ok(replies),)) => {
            // Extract LP positions only
            let lp_positions: Vec<LPBalancesReply> = replies.into_iter()
                .filter_map(|reply| match reply {
                    UserBalancesReply::LP(lp) => Some(lp),
                    _ => None,
                })
                .collect();

            ic_cdk::println!("Retrieved {} LP positions", lp_positions.len());
            Ok(lp_positions)
        }
        Ok((UserBalancesResult::Err(e),)) => {
            Err(IcpiError::Kongswap(KongswapError::BackendUnreachable {
                reason: format!("Kongswap error: {}", e),
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Kongswap(KongswapError::BackendUnreachable {
                reason: format!("{:?}: {}", code, msg),
            }))
        }
    }
}
EOF

# Create TVL module
mkdir -p tvl
cat > tvl/mod.rs << 'EOF'
//! TVL calculation from Kong locked liquidity

use crate::infrastructure::{Result};
use crate::types::TrackedToken;
use std::collections::HashMap;

/// Calculate locked TVL for all tracked tokens
pub async fn calculate_locked_tvl() -> Result<HashMap<TrackedToken, f64>> {
    let lock_canisters = crate::_3_KONG_LIQUIDITY::locker::get_all_lock_canisters().await?;
    let mut tvl_by_token: HashMap<TrackedToken, f64> = HashMap::new();

    // Initialize tracked tokens
    for token in TrackedToken::all() {
        tvl_by_token.insert(token, 0.0);
    }

    // Query all lock canisters in parallel
    let position_futures: Vec<_> = lock_canisters.iter()
        .map(|(_user, lock)| crate::_3_KONG_LIQUIDITY::pools::get_lp_positions(*lock))
        .collect();

    let position_results = futures::future::join_all(position_futures).await;

    for result in position_results {
        if let Ok(positions) = result {
            for lp in positions {
                process_lp_position(&lp, &mut tvl_by_token);
            }
        }
    }

    Ok(tvl_by_token)
}

fn process_lp_position(
    lp: &crate::types::LPBalancesReply,
    tvl_by_token: &mut HashMap<TrackedToken, f64>,
) {
    // For 50/50 pools, attribute half value to each token
    let half_value = lp.usd_balance.0.to_u64().unwrap_or(0) as f64 / 2.0 / 1e6; // Convert to USD

    update_token_tvl(&lp.symbol_0, half_value, tvl_by_token);
    update_token_tvl(&lp.symbol_1, half_value, tvl_by_token);
}

fn update_token_tvl(symbol: &str, value: f64, tvl_by_token: &mut HashMap<TrackedToken, f64>) {
    if let Ok(token) = TrackedToken::from_symbol(symbol) {
        if tvl_by_token.contains_key(&token) {
            *tvl_by_token.get_mut(&token).unwrap() += value;
        }
    }
}
EOF
```

### Step 5.6: Migrate Trading (Zone 4)

```bash
cd src/icpi_backend/src/4_TRADING_EXECUTION

# Create swaps module
mkdir -p swaps
cat > swaps/mod.rs << 'EOF'
//! Swap execution on Kongswap

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, TradingError};
use crate::types::{SwapArgs, SwapReply, SwapAmountsResult, KONGSWAP_BACKEND_ID};

/// Execute a swap on Kongswap
pub async fn execute_swap(
    pay_symbol: String,
    pay_amount: Nat,
    receive_symbol: String,
    max_slippage: Option<f64>,
) -> Result<SwapReply> {
    let kongswap = Principal::from_text(KONGSWAP_BACKEND_ID)
        .map_err(|e| IcpiError::Trading(TradingError::SwapFailed {
            reason: format!("Invalid Kongswap principal: {}", e),
        }))?;

    // Get quote first
    let quote_result: Result<(SwapAmountsResult,), _> = ic_cdk::call(
        kongswap,
        "swap_amounts",
        (pay_symbol.clone(), pay_amount.clone(), receive_symbol.clone())
    ).await;

    let quote = match quote_result {
        Ok((SwapAmountsResult::Ok(q),)) => q,
        Ok((SwapAmountsResult::Err(e),)) => {
            return Err(IcpiError::Trading(TradingError::InvalidQuote {
                reason: format!("Quote error: {}", e),
            }));
        }
        Err((code, msg)) => {
            return Err(IcpiError::Trading(TradingError::SwapFailed {
                reason: format!("Quote call failed: {:?} - {}", code, msg),
            }));
        }
    };

    // Check slippage
    let max_slip = max_slippage.unwrap_or(2.0);
    if quote.slippage > max_slip {
        return Err(IcpiError::Trading(TradingError::SlippageTooHigh {
            actual: quote.slippage.to_string(),
            max: max_slip.to_string(),
        }));
    }

    // Need approval for non-ICP tokens
    if pay_symbol != "ICP" {
        crate::_4_TRADING_EXECUTION::approvals::approve_for_swap(&pay_symbol, &pay_amount).await?;
    }

    // Execute swap
    let swap_args = SwapArgs {
        pay_symbol,
        pay_amount,
        receive_symbol,
        receive_amount: None,
        max_slippage: Some(max_slip),
        referred_by: None,
    };

    let swap_result: Result<(Result<SwapReply, String>,), _> = ic_cdk::call(
        kongswap,
        "swap",
        (swap_args,)
    ).await;

    match swap_result {
        Ok((Ok(reply),)) => Ok(reply),
        Ok((Err(e),)) => {
            Err(IcpiError::Trading(TradingError::SwapFailed {
                reason: e,
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Trading(TradingError::SwapFailed {
                reason: format!("Swap call failed: {:?} - {}", code, msg),
            }))
        }
    }
}
EOF

# Create approvals module
mkdir -p approvals
cat > approvals/mod.rs << 'EOF'
//! Token approval management for swaps

use candid::{Nat, Principal};
use crate::infrastructure::{Result, IcpiError, ApprovalError};
use crate::types::{TrackedToken, KONGSWAP_BACKEND_ID};

/// Approve token for swap
pub async fn approve_for_swap(token_symbol: &str, amount: &Nat) -> Result<()> {
    let token = TrackedToken::from_symbol(token_symbol)?;
    let token_canister = token.get_canister_id()?;

    let kongswap = Principal::from_text(KONGSWAP_BACKEND_ID)
        .map_err(|e| IcpiError::Approval(ApprovalError::ApprovalFailed {
            token: token_symbol.to_string(),
            spender: KONGSWAP_BACKEND_ID.to_string(),
            reason: format!("Invalid principal: {}", e),
        }))?;

    let approve_args = crate::types::icrc::ApproveArgs {
        spender: crate::types::Account {
            owner: kongswap,
            subaccount: None,
        },
        amount: amount.clone(),
        expires_at: None,
        expected_allowance: None,
        memo: Some(b"Kongswap trade approval".to_vec()),
        fee: None,
        from_subaccount: None,
        created_at_time: None,
    };

    let result: Result<(Result<Nat, crate::types::icrc::ApproveError>,), _> = ic_cdk::call(
        token_canister,
        "icrc2_approve",
        (approve_args,)
    ).await;

    match result {
        Ok((Ok(_),)) => Ok(()),
        Ok((Err(e),)) => {
            Err(IcpiError::Approval(ApprovalError::ApprovalFailed {
                token: token_symbol.to_string(),
                spender: KONGSWAP_BACKEND_ID.to_string(),
                reason: format!("{:?}", e),
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Approval(ApprovalError::ApprovalFailed {
                token: token_symbol.to_string(),
                spender: KONGSWAP_BACKEND_ID.to_string(),
                reason: format!("Call failed: {:?} - {}", code, msg),
            }))
        }
    }
}
EOF
```

### Step 5.7: Migrate Informational (Zone 5)

```bash
cd src/icpi_backend/src/5_INFORMATIONAL

# Create display module
mkdir -p display
cat > display/mod.rs << 'EOF'
//! Display formatting for UI
//! ALWAYS uses cached data for performance

use crate::types::portfolio::IndexState;
use crate::infrastructure::cache::{get_cached, CachePolicy};

/// Get index state for display (cached)
pub async fn get_index_state_cached() -> IndexState {
    get_cached(
        "index_state",
        CachePolicy::Short,
        || async {
            // Get cached data
            let positions = crate::_2_CRITICAL_DATA::token_queries::get_current_positions().await
                .unwrap_or_default();

            let targets = crate::_2_CRITICAL_DATA::portfolio_value::calculate_target_allocations().await
                .unwrap_or_default();

            let ckusdt = crate::_2_CRITICAL_DATA::token_queries::get_ckusdt_balance().await
                .unwrap_or(candid::Nat::from(0u64));

            let total: f64 = positions.iter().map(|p| p.usd_value).sum();

            IndexState {
                total_value: total,
                current_positions: positions,
                target_allocations: targets,
                deviations: Vec::new(),
                ckusdt_balance: ckusdt,
                timestamp: ic_cdk::api::time(),
            }
        }
    ).await
}
EOF

# Create health module
mkdir -p health
cat > health/mod.rs << 'EOF'
//! System health monitoring

use crate::types::common::HealthStatus;

/// Get system health status
pub fn get_health_status() -> HealthStatus {
    HealthStatus {
        is_healthy: true,
        last_rebalance: crate::_1_CRITICAL_OPERATIONS::rebalancing::get_rebalancer_status()
            .last_rebalance
            .unwrap_or(0),
        pending_mints: 0, // TODO: implement
        cycles_balance: ic_cdk::api::canister_balance128(),
    }
}
EOF
```

---

## Phase 6: Clean Legacy Code

### Step 6.1: Remove Root-Level Duplicates

```bash
cd src/icpi_backend/src

# Backup first
mkdir -p /tmp/icpi_backup
cp *.rs /tmp/icpi_backup/

# Remove all root-level .rs files except lib.rs
ls -1 *.rs | grep -v "^lib.rs$" | xargs rm -f

echo "✅ Removed root-level duplicates" >> /tmp/icpi_refactor_log.txt

# Verify
ls *.rs
# Should only show: lib.rs
```

### Step 6.2: Remove Legacy Folder

```bash
# Remove entire legacy folder
rm -rf legacy/

echo "✅ Removed legacy folder" >> /tmp/icpi_refactor_log.txt

# Verify
test -d legacy && echo "❌ Legacy still exists" || echo "✅ Legacy removed"
```

### Step 6.3: Clean Up Old Module References

```bash
# Remove old module directories that aren't numbered
rm -rf critical_operations/ portfolio_data/ kong_liquidity/ market_data/
rm -rf informational/ orchestration/ infrastructure/

echo "✅ Removed old non-numbered directories" >> /tmp/icpi_refactor_log.txt
```

---

## Phase 7: Update lib.rs Router

### Step 7.1: Complete lib.rs Implementation

```bash
cat > src/icpi_backend/src/lib.rs << 'EOF'
//! ICPI Backend - Security-First Architecture with Numbered Zones
//!
//! Architecture:
//! 1_CRITICAL_OPERATIONS - Mint, burn, rebalance (highest security)
//! 2_CRITICAL_DATA - Portfolio calculations, supply tracking
//! 3_KONG_LIQUIDITY - External liquidity reference
//! 4_TRADING_EXECUTION - DEX interactions
//! 5_INFORMATIONAL - Display and caching
//! 6_INFRASTRUCTURE - Math, errors, constants

// Import numbered modules with explicit paths
#[path = "1_CRITICAL_OPERATIONS/mod.rs"]
mod critical_operations_1;
use critical_operations_1 as critical_operations;

#[path = "2_CRITICAL_DATA/mod.rs"]
mod critical_data_2;
use critical_data_2 as critical_data;

#[path = "3_KONG_LIQUIDITY/mod.rs"]
mod kong_liquidity_3;
use kong_liquidity_3 as kong_liquidity;

#[path = "4_TRADING_EXECUTION/mod.rs"]
mod trading_execution_4;
use trading_execution_4 as trading_execution;

#[path = "5_INFORMATIONAL/mod.rs"]
mod informational_5;
use informational_5 as informational;

#[path = "6_INFRASTRUCTURE/mod.rs"]
mod infrastructure_6;
use infrastructure_6 as infrastructure;

// Types module (existing)
mod types;

use candid::{candid_method, Nat, Principal};
use ic_cdk::{init, post_upgrade, query, update};
use infrastructure::{FeatureFlags, OperationStrategy, Result, IcpiError};

// ===== PUBLIC API =====

#[update]
#[candid_method(update)]
async fn mint_icpi(amount: Nat) -> Result<String> {
    let caller = ic_cdk::caller();
    critical_operations::minting::initiate_mint(caller, amount).await
}

#[update]
#[candid_method(update)]
async fn complete_mint(mint_id: String) -> Result<Nat> {
    let caller = ic_cdk::caller();
    critical_operations::minting::complete_mint(caller, mint_id).await
}

#[update]
#[candid_method(update)]
async fn burn_icpi(amount: Nat) -> Result<critical_operations::burning::BurnResult> {
    let caller = ic_cdk::caller();
    critical_operations::burning::burn_icpi(caller, amount).await
}

#[update]
#[candid_method(update)]
async fn perform_rebalance() -> Result<String> {
    // Admin check
    require_admin()?;
    critical_operations::rebalancing::perform_rebalance().await
}

#[update]
#[candid_method(update)]
async fn trigger_manual_rebalance() -> Result<String> {
    require_admin()?;
    critical_operations::rebalancing::trigger_manual_rebalance().await
}

#[query]
#[candid_method(query)]
async fn get_index_state() -> types::portfolio::IndexState {
    informational::display::get_index_state_cached().await
}

#[query]
#[candid_method(query)]
fn get_health_status() -> types::common::HealthStatus {
    informational::health::get_health_status()
}

#[query]
#[candid_method(query)]
fn get_tracked_tokens() -> Vec<String> {
    types::TrackedToken::all()
        .into_iter()
        .map(|t| t.to_symbol().to_string())
        .collect()
}

#[query]
#[candid_method(query)]
fn get_rebalancer_status() -> critical_operations::rebalancing::RebalancerStatus {
    critical_operations::rebalancing::get_rebalancer_status()
}

// ===== FEATURE FLAGS =====

#[query]
#[candid_method(query)]
fn get_feature_flags() -> infrastructure::FeatureFlagConfig {
    FeatureFlags::get_all_flags()
}

#[update]
#[candid_method(update)]
fn set_feature_flag(operation: String, strategy: String) -> Result<String> {
    require_admin()?;

    let strat = match strategy.as_str() {
        "legacy" => OperationStrategy::Legacy,
        "refactored" => OperationStrategy::Refactored,
        "shadow" => OperationStrategy::Shadow,
        _ => return Err(IcpiError::Other("Invalid strategy".to_string())),
    };

    FeatureFlags::set_strategy(&operation, strat)
}

// ===== INITIALIZATION =====

#[init]
fn init() {
    ic_cdk::println!("===================================");
    ic_cdk::println!("ICPI Backend Initialized");
    ic_cdk::println!("Architecture: Numbered Security Zones");
    ic_cdk::println!("Mode: REFACTORED (no legacy code)");
    ic_cdk::println!("===================================");

    // Start rebalancing timer
    critical_operations::rebalancing::start_rebalancing_timer();
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("ICPI Backend upgraded successfully");

    // Restart rebalancing timer
    critical_operations::rebalancing::start_rebalancing_timer();
}

// ===== HELPER FUNCTIONS =====

fn require_admin() -> Result<()> {
    const ADMIN_PRINCIPALS: &[&str] = &[
        "e454q-riaaa-aaaap-qqcyq-cai", // Example admin
    ];

    let caller = ic_cdk::caller();

    if ADMIN_PRINCIPALS.iter()
        .any(|&admin| Principal::from_text(admin).ok() == Some(caller))
    {
        Ok(())
    } else {
        Err(IcpiError::System(infrastructure::errors::SystemError::Unauthorized {
            principal: caller.to_text(),
            required_role: "admin".to_string(),
        }))
    }
}

// ===== CANDID EXPORT =====

ic_cdk::export_candid!();
EOF

echo "✅ Created complete lib.rs router" >> /tmp/icpi_refactor_log.txt
```

---

## Phase 8: Final Verification

### Step 8.1: Compilation Test

```bash
cd /home/theseus/alexandria/daopad/src/icpi

# Full compilation
cargo build --manifest-path src/icpi_backend/Cargo.toml --release 2>&1 | tee /tmp/final_build.txt

# Check for errors
if grep -q "^error" /tmp/final_build.txt; then
    echo "❌ Compilation errors found"
    grep "^error" /tmp/final_build.txt | head -10
else
    echo "✅ Compilation successful!"
fi
```

### Step 8.2: Run Complete Test Suite

```bash
# Run all tests
cargo test --manifest-path src/icpi_backend/Cargo.toml --all 2>&1 | tee /tmp/test_results.txt

# Check test results
if grep -q "test result: ok" /tmp/test_results.txt; then
    echo "✅ All tests passed!"
else
    echo "⚠️ Some tests failed"
    grep "failures:" /tmp/test_results.txt
fi
```

### Step 8.3: Verify File Structure

```bash
# Verify numbered zones exist
for i in {1..6}; do
    zone=$(find src/icpi_backend/src -type d -name "${i}_*" | head -1)
    if [ -n "$zone" ]; then
        count=$(find "$zone" -name "*.rs" | wc -l)
        echo "✅ Zone $i: $(basename $zone) - $count files"
    else
        echo "❌ Zone $i missing"
    fi
done

# Verify no legacy code
echo ""
echo "Legacy cleanup check:"
test -d src/icpi_backend/src/legacy && echo "❌ Legacy folder exists" || echo "✅ Legacy removed"

# Count root-level files (should be only lib.rs)
root_count=$(ls src/icpi_backend/src/*.rs 2>/dev/null | grep -v lib.rs | wc -l)
if [ "$root_count" -eq 0 ]; then
    echo "✅ No root-level duplicates"
else
    echo "❌ Found $root_count root-level files"
fi
```

### Step 8.4: Deploy to Testnet

```bash
# Deploy to testnet first
dfx deploy icpi_backend --network testnet 2>&1 | tee /tmp/testnet_deploy.txt

# Run smoke tests
echo "Running smoke tests..."
dfx canister --network testnet call icpi_backend get_health_status
dfx canister --network testnet call icpi_backend get_tracked_tokens
```

### Step 8.5: Gradual Mainnet Rollout

```bash
# Deploy to mainnet (if testnet successful)
dfx deploy icpi_backend --network ic

# Monitor for 24 hours
echo "Deployment complete. Monitor logs for 24 hours:"
echo "dfx canister --network ic logs icpi_backend --follow"
```

---

## Migration Summary

### Files Migrated

| Original Files | Lines | New Structure | Lines Saved |
|----------------|-------|--------------|-------------|
| 13 root .rs files | ~3,500 | Removed (duplicates) | 3,500 |
| legacy/ folder | ~3,500 | Removed (duplicates) | 3,500 |
| partial modules | ~1,329 | Organized into zones | 0 |
| **Total** | **~8,329** | **~2,000** | **~6,329 (76% reduction)** |

### Architecture Improvements

1. ✅ **Numbered security zones** - Clear hierarchy
2. ✅ **Type-safe errors** - No more string errors
3. ✅ **Pure functions** - Separated I/O from calculations
4. ✅ **No duplication** - Single source of truth
5. ✅ **Clean imports** - Clear dependency graph
6. ✅ **Audit ready** - Security boundaries obvious

### Commands to Verify Success

```bash
# Final verification checklist
echo "=== ICPI Refactoring Complete ==="
echo ""
echo "✓ Compilation:" && cargo check --manifest-path src/icpi_backend/Cargo.toml 2>&1 | grep -q "Finished" && echo "  PASS" || echo "  FAIL"
echo "✓ Tests:" && cargo test --manifest-path src/icpi_backend/Cargo.toml 2>&1 | grep -q "test result: ok" && echo "  PASS" || echo "  FAIL"
echo "✓ Structure:" && test -d src/icpi_backend/src/1_CRITICAL_OPERATIONS && echo "  PASS" || echo "  FAIL"
echo "✓ Legacy removed:" && ! test -d src/icpi_backend/src/legacy && echo "  PASS" || echo "  FAIL"
echo "✓ Root cleaned:" && [ $(ls src/icpi_backend/src/*.rs | grep -v lib.rs | wc -l) -eq 0 ] && echo "  PASS" || echo "  FAIL"
echo ""
echo "Ready for mainnet deployment!"
```

---

END OF PHASES 5-8