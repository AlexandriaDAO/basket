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
use critical_operations_1 as _1_CRITICAL_OPERATIONS;

#[path = "2_CRITICAL_DATA/mod.rs"]
mod critical_data_2;
use critical_data_2 as _2_CRITICAL_DATA;

#[path = "3_KONG_LIQUIDITY/mod.rs"]
mod kong_liquidity_3;
use kong_liquidity_3 as _3_KONG_LIQUIDITY;

#[path = "4_TRADING_EXECUTION/mod.rs"]
mod trading_execution_4;
use trading_execution_4 as _4_TRADING_EXECUTION;

#[path = "5_INFORMATIONAL/mod.rs"]
mod informational_5;
use informational_5 as _5_INFORMATIONAL;

#[path = "6_INFRASTRUCTURE/mod.rs"]
mod infrastructure_6;
use infrastructure_6 as infrastructure;

// Types module (existing)
mod types;

use candid::{candid_method, Nat, Principal};
use ic_cdk::{init, pre_upgrade, post_upgrade, query, update};
use infrastructure::{Result, IcpiError};

// ===== PUBLIC API =====

#[update]
#[candid_method(update)]
async fn initiate_mint(amount: Nat) -> Result<String> {
    let caller = ic_cdk::caller();
    _1_CRITICAL_OPERATIONS::minting::initiate_mint(caller, amount).await
}

#[update]
#[candid_method(update)]
async fn complete_mint(mint_id: String) -> Result<Nat> {
    let caller = ic_cdk::caller();
    _1_CRITICAL_OPERATIONS::minting::complete_mint(caller, mint_id).await
}

#[update]
#[candid_method(update)]
async fn burn_icpi(amount: Nat) -> Result<_1_CRITICAL_OPERATIONS::burning::BurnResult> {
    let caller = ic_cdk::caller();
    _1_CRITICAL_OPERATIONS::burning::burn_icpi(caller, amount).await
}

#[update]
#[candid_method(update)]
async fn perform_rebalance() -> Result<String> {
    require_admin()?;
    _1_CRITICAL_OPERATIONS::rebalancing::perform_rebalance().await
}

#[update]
#[candid_method(update)]
async fn trigger_manual_rebalance() -> Result<String> {
    require_admin()?;
    _1_CRITICAL_OPERATIONS::rebalancing::trigger_manual_rebalance().await
}

#[update]
#[candid_method(update)]
async fn get_index_state() -> Result<types::portfolio::IndexState> {
    _5_INFORMATIONAL::display::get_index_state_cached().await
}

/// NOTE (PR #8 Review): Reviewer suggested this should be #[query] for cached reads
/// However, the underlying implementation makes inter-canister calls (get_portfolio_state_uncached)
/// which requires #[update]. The "cached" name is misleading - there's no actual cache implementation yet.
/// TODO: Implement real caching in Phase 2/3, then convert to #[query]
#[update]
#[candid_method(update)]
async fn get_index_state_cached() -> Result<types::portfolio::IndexState> {
    _5_INFORMATIONAL::display::get_index_state_cached().await
}

#[query]
#[candid_method(query)]
fn get_health_status() -> types::common::HealthStatus {
    _5_INFORMATIONAL::health::get_health_status()
}

#[query]
#[candid_method(query)]
fn get_tracked_tokens() -> Vec<String> {
    _5_INFORMATIONAL::health::get_tracked_tokens()
}

#[query]
#[candid_method(query)]
fn get_rebalancer_status() -> _1_CRITICAL_OPERATIONS::rebalancing::RebalancerStatus {
    _1_CRITICAL_OPERATIONS::rebalancing::get_rebalancer_status()
}

/// Get full trade history (all trades since deployment)
#[query]
#[candid_method(query)]
fn get_trade_history() -> Vec<_1_CRITICAL_OPERATIONS::rebalancing::RebalanceRecord> {
    _1_CRITICAL_OPERATIONS::rebalancing::get_full_trade_history()
}

/// Get paginated trade history
#[query]
#[candid_method(query)]
fn get_trade_history_paginated(offset: u64, limit: u64) -> (Vec<_1_CRITICAL_OPERATIONS::rebalancing::RebalanceRecord>, u64) {
    let full_history = _1_CRITICAL_OPERATIONS::rebalancing::get_full_trade_history();
    let total = full_history.len() as u64;

    let start = offset as usize;
    let end = std::cmp::min(start + (limit as usize), full_history.len());

    let page = if start < full_history.len() {
        full_history[start..end].to_vec()
    } else {
        Vec::new()
    };

    (page, total)
}

#[update]
#[candid_method(update)]
fn clear_caches() -> Result<String> {
    // Enforce admin check - returns error if unauthorized
    require_admin()?;

    _5_INFORMATIONAL::cache::clear_all_caches();
    ic_cdk::println!("Admin {} cleared all caches", ic_cdk::caller());
    Ok("Caches cleared".to_string())
}

// ===== TESTING =====

/// Test endpoint for Kong Liquidity integration (Zone 3)
///
/// Queries Kong Locker TVL and Kongswap prices to verify integration works.
/// Use this after deployment to validate mainnet connectivity.
///
/// Example:
/// ```bash
/// dfx canister call --network ic ev6xm-haaaa-aaaap-qqcza-cai test_kong_liquidity
/// ```
// ===== ADDITIONAL API ENDPOINTS =====

/// BUGFIX (PR #8 Review): Use getter function instead of direct PENDING_MINTS access
#[query]
#[candid_method(query)]
fn check_mint_status(mint_id: String) -> Result<_1_CRITICAL_OPERATIONS::minting::MintStatus> {
    _1_CRITICAL_OPERATIONS::minting::mint_state::get_mint_status(&mint_id)?
        .ok_or(infrastructure::IcpiError::Other(format!("Mint {} not found", mint_id)))
}

#[update]
#[candid_method(update)]
async fn get_tvl_summary() -> Result<types::portfolio::TvlSummary> {
    // Calculate TVL from Kong Locker
    let tvl_data = _3_KONG_LIQUIDITY::tvl::calculate_kong_locker_tvl().await?;

    // Calculate total and percentages
    let total_tvl: f64 = tvl_data.iter().map(|(_, v)| v).sum();

    let tokens: Vec<types::portfolio::TokenTvl> = tvl_data.iter().map(|(token, usd_value)| {
        types::portfolio::TokenTvl {
            token: token.clone(),
            tvl_usd: *usd_value,  // Fixed field name to match .did file
            percentage: if total_tvl > 0.0 { (usd_value / total_tvl) * 100.0 } else { 0.0 },
        }
    }).collect();

    Ok(types::portfolio::TvlSummary {
        total_tvl_usd: total_tvl,
        tokens: tokens,  // Fixed field name to match .did file
        timestamp: ic_cdk::api::time(),
    })
}

#[query]
#[candid_method(query)]
fn get_token_metadata() -> Result<Vec<types::tokens::TokenMetadata>> {
    use types::TrackedToken;

    // Use TrackedToken methods to avoid hardcoded canister IDs
    let tokens: Result<Vec<types::tokens::TokenMetadata>> = TrackedToken::all()
        .iter()
        .map(|token| {
            let canister_id = token.get_canister_id()
                .map_err(|e| IcpiError::Other(e))?;
            Ok(types::tokens::TokenMetadata {
                symbol: token.to_symbol().to_string(),
                canister_id,
                decimals: token.get_decimals(),
            })
        })
        .collect();

    tokens
}


#[query]
#[candid_method(query)]
fn get_canister_id() -> Principal {
    ic_cdk::id()
}

#[query]
#[candid_method(query)]
fn get_cycles_balance() -> Nat {
    Nat::from(ic_cdk::api::canister_balance128())
}

// ===== ICRC1 TOKEN STANDARD ENDPOINTS =====

#[query]
#[candid_method(query)]
fn icrc1_name() -> String {
    "Internet Computer Portfolio Index".to_string()
}

#[query]
#[candid_method(query)]
fn icrc1_symbol() -> String {
    "ICPI".to_string()
}

#[query]
#[candid_method(query)]
fn icrc1_decimals() -> u8 {
    8
}

#[query]
#[candid_method(query)]
fn icrc1_fee() -> Nat {
    Nat::from(10_000u64) // 0.0001 ICPI
}

#[query]
#[candid_method(query)]
fn icrc1_metadata() -> Vec<(String, types::icrc::MetadataValue)> {
    vec![
        ("icrc1:name".to_string(), types::icrc::MetadataValue::Text("Internet Computer Portfolio Index".to_string())),
        ("icrc1:symbol".to_string(), types::icrc::MetadataValue::Text("ICPI".to_string())),
        ("icrc1:decimals".to_string(), types::icrc::MetadataValue::Nat(Nat::from(8u64))),
        ("icrc1:fee".to_string(), types::icrc::MetadataValue::Nat(Nat::from(10_000u64))),
    ]
}

#[query]
#[candid_method(query)]
fn icrc1_supported_standards() -> Vec<types::icrc::StandardRecord> {
    vec![
        types::icrc::StandardRecord {
            name: "ICRC-1".to_string(),
            url: "https://github.com/dfinity/ICRC-1".to_string(),
        },
    ]
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
    _1_CRITICAL_OPERATIONS::rebalancing::start_rebalancing_timer();

    // Start mint cleanup timer to prevent memory leak
    // Runs every hour to clean up completed mints older than 24 hours
    ic_cdk_timers::set_timer_interval(
        std::time::Duration::from_secs(3600), // 1 hour
        || {
            ic_cdk::spawn(async {
                match _1_CRITICAL_OPERATIONS::minting::mint_state::cleanup_expired_mints() {
                    Ok(count) if count > 0 => {
                        ic_cdk::println!("🧹 Periodic cleanup: removed {} expired mints", count);
                    }
                    Ok(_) => {}, // No mints to clean
                    Err(e) => ic_cdk::println!("⚠️ Periodic cleanup failed: {}", e),
                }
            });
        }
    );
}

#[pre_upgrade]
fn pre_upgrade() {
    ic_cdk::println!("===================================");
    ic_cdk::println!("ICPI Backend Pre-Upgrade");
    ic_cdk::println!("===================================");

    let pending_mints = _1_CRITICAL_OPERATIONS::minting::mint_state::export_state();
    let trade_history = _1_CRITICAL_OPERATIONS::rebalancing::export_history_for_stable();

    infrastructure::stable_storage::save_state(pending_mints, trade_history.clone());

    ic_cdk::println!("✅ State saved to stable storage ({} trades)", trade_history.len());
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("===================================");
    ic_cdk::println!("ICPI Backend Post-Upgrade");
    ic_cdk::println!("===================================");

    let (pending_mints, trade_history) = infrastructure::stable_storage::restore_state();
    _1_CRITICAL_OPERATIONS::minting::mint_state::import_state(pending_mints);
    _1_CRITICAL_OPERATIONS::rebalancing::load_history_from_stable(trade_history.clone());

    match _1_CRITICAL_OPERATIONS::minting::mint_state::cleanup_expired_mints() {
        Ok(count) => {
            if count > 0 {
                ic_cdk::println!("🧹 Cleaned up {} expired mints after upgrade", count);
            }
        }
        Err(e) => ic_cdk::println!("⚠️  Failed to cleanup expired mints: {}", e),
    }

    _1_CRITICAL_OPERATIONS::rebalancing::start_rebalancing_timer();

    // Restart mint cleanup timer after upgrade
    ic_cdk_timers::set_timer_interval(
        std::time::Duration::from_secs(3600), // 1 hour
        || {
            ic_cdk::spawn(async {
                match _1_CRITICAL_OPERATIONS::minting::mint_state::cleanup_expired_mints() {
                    Ok(count) if count > 0 => {
                        ic_cdk::println!("🧹 Periodic cleanup: removed {} expired mints", count);
                    }
                    Ok(_) => {}, // No mints to clean
                    Err(e) => ic_cdk::println!("⚠️ Periodic cleanup failed: {}", e),
                }
            });
        }
    );

    ic_cdk::println!("✅ Backend upgraded successfully ({} trades restored)", trade_history.len());
}

// ===== HELPER FUNCTIONS =====

/// Verify caller is an admin principal (uses admin module)
fn require_admin() -> Result<()> {
    infrastructure::require_admin()
}

// ===== ADMIN CONTROLS (Phase 2: H-1) =====

/// Debug rebalancing state (admin only)
///
/// Returns comprehensive diagnostic information about:
/// - TVL targets from Kong Locker
/// - Current token balances
/// - Portfolio state calculation
/// - Pricing data
#[update]
#[candid_method(update)]
async fn debug_rebalancing_state() -> Result<String> {
    require_admin()?;

    let mut output = String::new();
    output.push_str("=== REBALANCING DIAGNOSTIC REPORT ===\n\n");

    // 1. Get TVL targets from Kong Locker
    output.push_str("1. Kong Locker TVL (Target Allocations):\n");
    match _3_KONG_LIQUIDITY::tvl::calculate_kong_locker_tvl().await {
        Ok(tvl_data) => {
            let total_tvl: f64 = tvl_data.iter().map(|(_, v)| v).sum();
            output.push_str(&format!("   Total TVL: ${:.2}\n", total_tvl));
            for (token, usd_value) in &tvl_data {
                let percentage = if total_tvl > 0.0 { (usd_value / total_tvl) * 100.0 } else { 0.0 };
                output.push_str(&format!("   {}: ${:.2} ({:.2}%)\n", token.to_symbol(), usd_value, percentage));
            }
        }
        Err(e) => output.push_str(&format!("   ❌ ERROR: {}\n", e)),
    }
    output.push_str("\n");

    // 2. Get current token balances
    output.push_str("2. Current Token Balances:\n");
    match _2_CRITICAL_DATA::token_queries::get_all_balances_uncached().await {
        Ok(balances) => {
            for (symbol, balance) in balances {
                output.push_str(&format!("   {}: {}\n", symbol, balance));
            }
        }
        Err(e) => output.push_str(&format!("   ❌ ERROR: {}\n", e)),
    }
    output.push_str("\n");

    // 3. Get portfolio state
    output.push_str("3. Portfolio State:\n");
    match _2_CRITICAL_DATA::portfolio_value::get_portfolio_state_uncached().await {
        Ok(state) => {
            output.push_str(&format!("   Total Value: ${:.2}\n", state.total_value));
            output.push_str(&format!("   Timestamp: {}\n", state.timestamp));
            output.push_str("   Current Positions:\n");
            for pos in &state.current_positions {
                output.push_str(&format!("     {}: ${:.2} ({:.2}%)\n",
                    pos.token.to_symbol(), pos.usd_value, pos.percentage));
            }
            output.push_str("   Target Allocations:\n");
            for target in &state.target_allocations {
                output.push_str(&format!("     {}: {:.2}% (${:.2})\n",
                    target.token.to_symbol(), target.target_percentage, target.target_usd_value));
            }
            output.push_str("   Deviations:\n");
            for dev in &state.deviations {
                output.push_str(&format!("     {}: current={:.2}% target={:.2}% deviation={:.2}% usd_diff=${:.2}\n",
                    dev.token.to_symbol(), dev.current_pct, dev.target_pct, dev.deviation_pct, dev.usd_difference));
            }
        }
        Err(e) => output.push_str(&format!("   ❌ ERROR: {}\n", e)),
    }
    output.push_str("\n");

    // 4. Get rebalancer status
    output.push_str("4. Rebalancer Status:\n");
    let status = _1_CRITICAL_OPERATIONS::rebalancing::get_rebalancer_status();
    output.push_str(&format!("   Timer Active: {}\n", status.timer_active));
    output.push_str(&format!("   Last Rebalance: {:?}\n", status.last_rebalance));
    output.push_str(&format!("   Next Rebalance: {:?}\n", status.next_rebalance));
    output.push_str(&format!("   Recent History Entries: {}\n", status.recent_history.len()));

    Ok(output)
}

/// Emergency pause - stops all minting and burning
#[update]
#[candid_method(update)]
fn emergency_pause() -> Result<()> {
    infrastructure::require_admin()?;
    infrastructure::set_pause(true);
    infrastructure::log_admin_action("EMERGENCY_PAUSE_ACTIVATED".to_string());
    ic_cdk::println!("🚨 EMERGENCY PAUSE ACTIVATED");
    Ok(())
}

/// Resume operations after emergency pause
#[update]
#[candid_method(update)]
fn emergency_unpause() -> Result<()> {
    infrastructure::require_admin()?;
    infrastructure::set_pause(false);
    infrastructure::log_admin_action("EMERGENCY_PAUSE_DEACTIVATED".to_string());
    ic_cdk::println!("✅ EMERGENCY PAUSE DEACTIVATED");
    Ok(())
}

/// Check if system is currently paused
#[query]
#[candid_method(query)]
fn is_emergency_paused() -> bool {
    infrastructure::is_paused()
}

/// Get admin action log (admin only)
#[query]
#[candid_method(query)]
fn get_admin_action_log() -> Result<Vec<infrastructure::AdminAction>> {
    infrastructure::require_admin()?;
    Ok(infrastructure::get_admin_log())
}

/// Clear all caches (admin only)
#[update]
#[candid_method(update)]
fn clear_all_caches() -> Result<()> {
    infrastructure::require_admin()?;
    infrastructure::log_admin_action("CACHES_CLEARED".to_string());
    _5_INFORMATIONAL::cache::clear_all_caches();
    ic_cdk::println!("✅ All caches cleared");
    Ok(())
}

// ===== CANDID EXPORT =====

ic_cdk::export_candid!();