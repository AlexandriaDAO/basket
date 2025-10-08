//! # Rebalancing Module
//!
//! Automatically rebalances the ICPI portfolio every hour to maintain
//! target allocations of 25% each: ALEX, ZERO, KONG, BOB.
//!
//! ## Strategy
//! - **Hourly timer**: Checks portfolio deviations every 3600 seconds
//! - **Sequential trades**: One trade per hour (Kong swap limitation)
//! - **Trade intensity**: 10% of deviation per trade (gradual rebalancing)
//! - **Buy priority**: If ckUSDT >= $10, buy most underweight token
//! - **Sell fallback**: Otherwise, sell most overweight token
//!
//! ## Deviation Calculation
//! For each token:
//! 1. Current allocation = token_value / total_value
//! 2. Target allocation = 25% (equal weight)
//! 3. Deviation = target - current
//! 4. Trade size = 10% of (deviation * total_value)
//!
//! ## Example Flow
//! ```text
//! Portfolio: $100 total
//! - ALEX: $30 (30%, target 25%, +5% overweight)
//! - ZERO: $25 (25%, target 25%, balanced)
//! - KONG: $25 (25%, target 25%, balanced)
//! - BOB: $15 (15%, target 25%, -10% underweight)
//! - ckUSDT: $5
//!
//! Action: Buy BOB with 10% of $10 deficit = $1 of BOB
//! ```
//!
//! ## Safety Features
//! - Minimum $10 trade size prevents dust trades
//! - 2% max slippage on all swaps
//! - Keeps last MAX_REBALANCE_HISTORY records for audit
//! - Comprehensive logging for diagnostics

use std::cell::RefCell;
use candid::{CandidType, Deserialize, Nat};
use num_traits::ToPrimitive;
use crate::infrastructure::{Result, IcpiError, errors::RebalanceError, REBALANCE_INTERVAL_SECONDS, MIN_TRADE_SIZE_USD, MAX_SLIPPAGE_PERCENT};
use crate::types::{TrackedToken, rebalancing::AllocationDeviation};

/// Maximum number of rebalance records to keep in history
const MAX_REBALANCE_HISTORY: usize = 10;

// === TYPES ===

/// Rebalance action to execute
#[derive(Debug, Clone, CandidType, Deserialize, serde::Serialize)]
pub enum RebalanceAction {
    None,
    Buy { token: TrackedToken, usdt_amount: f64 },
    Sell { token: TrackedToken, usdt_value: f64 },
}

/// Record of a rebalance execution
#[derive(Debug, Clone, CandidType, Deserialize, serde::Serialize)]
pub struct RebalanceRecord {
    pub timestamp: u64,
    pub action: RebalanceAction,
    pub success: bool,
    pub details: String,
}

/// Rebalancer status for monitoring
#[derive(CandidType, Deserialize, serde::Serialize, Debug)]
pub struct RebalancerStatus {
    pub timer_active: bool,
    pub last_rebalance: Option<u64>,
    pub next_rebalance: Option<u64>,
    pub recent_history: Vec<RebalanceRecord>,
}

// === STATE ===

/// Thread-local state for rebalancing history
///
/// **IMPORTANT**: Thread-local state is NOT persisted across canister upgrades.
/// After an upgrade:
/// - Rebalancing history will be cleared
/// - Timer must be restarted in post_upgrade
/// - Last rebalance timestamp will be reset
///
/// This is acceptable for rebalancing since:
/// - History is only for debugging/audit trail
/// - Timer restart is handled in post_upgrade
/// - Rebalancing can safely restart fresh after upgrade
struct RebalanceState {
    last_rebalance: Option<u64>,
    history: Vec<RebalanceRecord>,
}

impl Default for RebalanceState {
    fn default() -> Self {
        Self {
            last_rebalance: None,
            history: Vec::new(),
        }
    }
}

thread_local! {
    static REBALANCE_STATE: RefCell<RebalanceState> = RefCell::new(RebalanceState::default());
    static TIMER_ACTIVE: RefCell<bool> = RefCell::new(false);
    static REBALANCING_IN_PROGRESS: RefCell<bool> = RefCell::new(false);
}

// === PUBLIC API ===

/// Start hourly rebalancing timer
///
/// Called during canister init and post_upgrade.
/// Executes `hourly_rebalance()` every 3600 seconds (1 hour).
pub fn start_rebalancing_timer() {
    ic_cdk::println!("🕐 Starting rebalancing timer (hourly)");

    // Mark timer as active
    TIMER_ACTIVE.with(|active| {
        *active.borrow_mut() = true;
    });

    // Set up recurring timer
    ic_cdk_timers::set_timer_interval(
        std::time::Duration::from_secs(REBALANCE_INTERVAL_SECONDS),
        || {
            // Check if rebalancing is already in progress
            let already_running = REBALANCING_IN_PROGRESS.with(|flag| {
                let is_running = *flag.borrow();
                if !is_running {
                    *flag.borrow_mut() = true;
                }
                is_running
            });

            if already_running {
                ic_cdk::println!("⚠️ Rebalancing already in progress, skipping this cycle");
                return;
            }

            ic_cdk::spawn(async {
                let result = hourly_rebalance().await;

                // Clear the in-progress flag
                REBALANCING_IN_PROGRESS.with(|flag| {
                    *flag.borrow_mut() = false;
                });

                match result {
                    Ok(msg) => ic_cdk::println!("✅ Rebalance: {}", msg),
                    Err(e) => ic_cdk::println!("❌ Rebalance failed: {}", e),
                }
            });
        }
    );

    ic_cdk::println!("✅ Rebalancing timer active");
}

/// Manual rebalancing trigger (admin only)
///
/// Executes a single rebalancing cycle immediately.
/// Useful for testing or emergency interventions.
pub async fn perform_rebalance() -> Result<String> {
    ic_cdk::println!("🔧 Manual rebalance triggered");

    // Check if rebalancing is already in progress
    let already_running = REBALANCING_IN_PROGRESS.with(|flag| {
        let is_running = *flag.borrow();
        if !is_running {
            *flag.borrow_mut() = true;
        }
        is_running
    });

    if already_running {
        return Err(IcpiError::Rebalance(RebalanceError::RebalancingInProgress));
    }

    let result = hourly_rebalance().await;

    // Clear the in-progress flag
    REBALANCING_IN_PROGRESS.with(|flag| {
        *flag.borrow_mut() = false;
    });

    result
}

/// Trigger manual rebalance (alias for perform_rebalance)
pub async fn trigger_manual_rebalance() -> Result<String> {
    perform_rebalance().await
}

/// Get current rebalancer status
pub fn get_rebalancer_status() -> RebalancerStatus {
    let timer_active = TIMER_ACTIVE.with(|active| *active.borrow());

    REBALANCE_STATE.with(|state| {
        let state = state.borrow();
        RebalancerStatus {
            timer_active,
            last_rebalance: state.last_rebalance,
            next_rebalance: state.last_rebalance.map(|last| {
                last + (REBALANCE_INTERVAL_SECONDS * 1_000_000_000)
            }),
            recent_history: state.history.clone(),
        }
    })
}

// === CORE LOGIC ===

/// Execute one hourly rebalancing cycle
///
/// ## Process
/// 1. Get current portfolio state from Zone 5
/// 2. Analyze deviations to determine action
/// 3. Execute buy or sell based on priority
/// 4. Record result for history
async fn hourly_rebalance() -> Result<String> {
    ic_cdk::println!("🔄 Starting hourly rebalance cycle...");

    // Get current portfolio state (includes deviations)
    let state = crate::_5_INFORMATIONAL::display::get_index_state_cached().await;

    ic_cdk::println!(
        "📊 Portfolio: ${:.2} total, ${} ckUSDT available",
        state.total_value,
        state.ckusdt_balance
    );

    // Determine what action to take
    let action = get_rebalancing_action(&state.deviations, &state.ckusdt_balance)?;

    // Execute trade if needed
    let result = match action.clone() {
        RebalanceAction::None => {
            let msg = "No rebalancing needed (all tokens within tolerance)".to_string();
            ic_cdk::println!("✅ {}", msg);
            record_rebalance(action, true, &msg);
            Ok(msg)
        }
        RebalanceAction::Buy { token, usdt_amount } => {
            execute_buy_action(&token, usdt_amount).await
        }
        RebalanceAction::Sell { token, usdt_value } => {
            execute_sell_action(&token, usdt_value).await
        }
    };

    result
}

/// Determine rebalancing action based on current state
///
/// ## Priority Logic
/// 1. **If ckUSDT >= $10**: Buy most underweight token (10% of deficit)
/// 2. **Else if overweight tokens exist**: Sell most overweight (10% of excess)
/// 3. **Else**: No action (portfolio balanced or insufficient funds)
///
/// ## Parameters
/// - `deviations`: Current vs target allocations for all tokens
/// - `ckusdt_balance`: Available ckUSDT for purchases (e6 decimals)
pub fn get_rebalancing_action(
    deviations: &[AllocationDeviation],
    ckusdt_balance: &Nat,
) -> Result<RebalanceAction> {
    // Convert ckUSDT balance to USD
    let ckusdt_usd = ckusdt_balance.0.to_u64().unwrap_or(0) as f64 / 1_000_000.0;

    // Find most underweight token (largest positive usd_difference)
    let most_underweight = deviations.iter()
        .filter(|d| d.usd_difference > 0.0) // Needs more tokens
        .max_by(|a, b| a.usd_difference.partial_cmp(&b.usd_difference)
            .unwrap_or(std::cmp::Ordering::Equal));

    // Check if we can buy
    if ckusdt_usd >= MIN_TRADE_SIZE_USD {
        if let Some(deficit) = most_underweight {
            if deficit.usd_difference > MIN_TRADE_SIZE_USD {
                ic_cdk::println!(
                    "📈 Buy signal: {} is {:.2}% underweight (deficit: ${:.2})",
                    deficit.token.to_symbol(),
                    deficit.deviation_pct.abs(),
                    deficit.usd_difference
                );

                return Ok(RebalanceAction::Buy {
                    token: deficit.token.clone(),
                    usdt_amount: deficit.trade_size_usd, // Already 10% of deficit
                });
            }
        }
    }

    // Find most overweight token (largest negative usd_difference)
    let most_overweight = deviations.iter()
        .filter(|d| d.usd_difference < 0.0) // Has excess tokens
        .min_by(|a, b| a.usd_difference.partial_cmp(&b.usd_difference)
            .unwrap_or(std::cmp::Ordering::Equal));

    if let Some(excess) = most_overweight {
        if excess.usd_difference.abs() > MIN_TRADE_SIZE_USD {
            ic_cdk::println!(
                "📉 Sell signal: {} is {:.2}% overweight (excess: ${:.2})",
                excess.token.to_symbol(),
                excess.deviation_pct.abs(),
                excess.usd_difference.abs()
            );

            return Ok(RebalanceAction::Sell {
                token: excess.token.clone(),
                usdt_value: excess.trade_size_usd, // Already 10% of excess
            });
        }
    }

    ic_cdk::println!("⚖️  Portfolio balanced (no significant deviations)");
    Ok(RebalanceAction::None)
}

/// Execute a buy action (ckUSDT → token)
///
/// ## Process
/// 1. Convert USD amount to ckUSDT (e6 decimals)
/// 2. Execute swap via Zone 4
/// 3. Log results and update history
async fn execute_buy_action(token: &TrackedToken, usd_amount: f64) -> Result<String> {
    let ckusdt_amount = Nat::from((usd_amount * 1_000_000.0).round() as u64);

    ic_cdk::println!(
        "💰 Buying {} with ${:.2} ({} ckUSDT)",
        token.to_symbol(),
        usd_amount,
        ckusdt_amount
    );

    // Execute swap via Zone 4
    let swap_result = crate::_4_TRADING_EXECUTION::swaps::execute_swap(
        &TrackedToken::ckUSDT,
        ckusdt_amount.clone(),
        token,
        MAX_SLIPPAGE_PERCENT / 100.0, // Convert percentage to decimal
    ).await;

    match swap_result {
        Ok(reply) => {
            let msg = format!(
                "Bought {} {} with ${:.2} (slippage: {:.2}%)",
                reply.receive_amount,
                token.to_symbol(),
                usd_amount,
                reply.slippage * 100.0
            );
            ic_cdk::println!("✅ {}", msg);
            record_rebalance(
                RebalanceAction::Buy { token: token.clone(), usdt_amount: usd_amount },
                true,
                &msg
            );
            Ok(msg)
        }
        Err(e) => {
            let msg = format!("Buy failed: {}", e);
            ic_cdk::println!("❌ {}", msg);
            record_rebalance(
                RebalanceAction::Buy { token: token.clone(), usdt_amount: usd_amount },
                false,
                &msg
            );
            Err(e)
        }
    }
}

/// Execute a sell action (token → ckUSDT)
///
/// ## Process
/// 1. Get current token price from Zone 3
/// 2. Calculate token amount to sell (USD value / price)
/// 3. Execute swap via Zone 4
/// 4. Log results and update history
async fn execute_sell_action(token: &TrackedToken, usd_value: f64) -> Result<String> {
    // Get current token price
    let price = crate::_3_KONG_LIQUIDITY::pools::get_token_price_in_usdt(token).await?;

    // Calculate token amount to sell (in token's base units)
    let token_decimals = token.get_decimals() as u32;
    let decimal_multiplier = 10f64.powi(token_decimals as i32);
    let token_amount_f64 = (usd_value / price) * decimal_multiplier;
    let token_amount = Nat::from(token_amount_f64.round() as u64);

    // Check if we have sufficient balance
    let balance = crate::_2_CRITICAL_DATA::token_queries::get_token_balance_uncached(token).await?;
    if balance < token_amount {
        return Err(IcpiError::Rebalance(RebalanceError::InsufficientBalance {
            token: token.to_symbol().to_string(),
            available: balance.to_string(),
            required: token_amount.to_string(),
        }));
    }

    ic_cdk::println!(
        "💸 Selling {} {} (~${:.2}) for ckUSDT (price: ${:.6})",
        token_amount,
        token.to_symbol(),
        usd_value,
        price
    );

    // Execute swap via Zone 4
    let swap_result = crate::_4_TRADING_EXECUTION::swaps::execute_swap(
        token,
        token_amount.clone(),
        &TrackedToken::ckUSDT,
        MAX_SLIPPAGE_PERCENT / 100.0, // Convert percentage to decimal
    ).await;

    match swap_result {
        Ok(reply) => {
            let received_usd = reply.receive_amount.0.to_u64().unwrap_or(0) as f64 / 1_000_000.0;
            let msg = format!(
                "Sold {} {} for ${:.2} (slippage: {:.2}%)",
                token_amount,
                token.to_symbol(),
                received_usd,
                reply.slippage * 100.0
            );
            ic_cdk::println!("✅ {}", msg);
            record_rebalance(
                RebalanceAction::Sell { token: token.clone(), usdt_value: usd_value },
                true,
                &msg
            );
            Ok(msg)
        }
        Err(e) => {
            let msg = format!("Sell failed: {}", e);
            ic_cdk::println!("❌ {}", msg);
            record_rebalance(
                RebalanceAction::Sell { token: token.clone(), usdt_value: usd_value },
                false,
                &msg
            );
            Err(e)
        }
    }
}

/// Record rebalance result in history
///
/// Keeps last MAX_REBALANCE_HISTORY records for audit trail and debugging.
fn record_rebalance(action: RebalanceAction, success: bool, details: &str) {
    REBALANCE_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.last_rebalance = Some(ic_cdk::api::time());

        state.history.push(RebalanceRecord {
            timestamp: ic_cdk::api::time(),
            action,
            success,
            details: details.to_string(),
        });

        // Keep only last MAX_REBALANCE_HISTORY records
        if state.history.len() > MAX_REBALANCE_HISTORY {
            state.history.remove(0);
        }
    });
}
