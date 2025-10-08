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
async fn mint_icpi(amount: Nat) -> Result<String> {
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
    Ok(_5_INFORMATIONAL::display::get_index_state_cached().await)
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

#[update]
#[candid_method(update)]
fn clear_caches() -> Result<String> {
    // Enforce admin check - returns error if unauthorized
    require_admin()?;

    _5_INFORMATIONAL::cache::clear_all_caches();
    ic_cdk::println!("Admin {} cleared all caches", ic_cdk::caller());
    Ok("Caches cleared".to_string())
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
    infrastructure::stable_storage::save_state(pending_mints);

    ic_cdk::println!("✅ State saved to stable storage");
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("===================================");
    ic_cdk::println!("ICPI Backend Post-Upgrade");
    ic_cdk::println!("===================================");

    let pending_mints = infrastructure::stable_storage::restore_state();
    _1_CRITICAL_OPERATIONS::minting::mint_state::import_state(pending_mints);

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

    ic_cdk::println!("✅ Backend upgraded successfully");
}

// ===== HELPER FUNCTIONS =====

/// Verify caller is an admin principal
///
/// Admin principals can:
/// - Trigger manual rebalancing
/// - Clear caches
/// - Access diagnostic functions
///
/// **ALPHA V1 LIMITATION**: Currently only the backend canister itself has admin access
/// This is sufficient for Alpha v1 since:
/// - Rebalancing is not yet implemented (stubbed)
/// - Cache clearing can be triggered via canister upgrade if needed
/// - No critical admin operations required during Alpha testing
///
/// **PRODUCTION NOTE**: Beta/Production versions should add:
/// - Deployer principal for manual interventions
/// - Controller principals for emergency operations
/// - Multi-sig or DAO governance for admin actions
///
/// Current admins:
/// - ev6xm-haaaa-aaaap-qqcza-cai (ICPI Backend itself) - For timer-triggered operations
///
/// Security Note: Frontend canister MUST NOT have admin access to prevent
/// potential security vulnerabilities. Frontend should only call public query/update methods.
fn require_admin() -> Result<()> {
    const ADMIN_PRINCIPALS: &[&str] = &[
        "ev6xm-haaaa-aaaap-qqcza-cai",  // ICPI Backend (self, for timers only)
        // ALPHA V1: No manual admin principals added yet
        // Beta will add controller/deployer principals here
    ];

    let caller = ic_cdk::caller();

    // Allow admin principals
    if ADMIN_PRINCIPALS.iter()
        .any(|&admin| Principal::from_text(admin).ok() == Some(caller))
    {
        return Ok(());
    }

    // For debugging: log unauthorized attempts
    ic_cdk::println!("⚠️  Unauthorized admin access attempt from {}", caller);

    Err(IcpiError::System(infrastructure::errors::SystemError::Unauthorized {
        principal: caller.to_text(),
        required_role: "admin (frontend or backend canister)".to_string(),
    }))
}

// ===== CANDID EXPORT =====

ic_cdk::export_candid!();