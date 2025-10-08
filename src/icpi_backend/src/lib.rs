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
    Ok(_5_INFORMATIONAL::display::get_index_state_cached().await)
}

/// NOTE (PR #8 Review): Reviewer suggested this should be #[query] for cached reads
/// However, the underlying implementation makes inter-canister calls (get_portfolio_state_uncached)
/// which requires #[update]. The "cached" name is misleading - there's no actual cache implementation yet.
/// TODO: Implement real caching in Phase 2/3, then convert to #[query]
#[update]
#[candid_method(update)]
async fn get_index_state_cached() -> Result<types::portfolio::IndexState> {
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
            locked_value_usd: *usd_value,
            percentage: if total_tvl > 0.0 { (usd_value / total_tvl) * 100.0 } else { 0.0 },
        }
    }).collect();

    Ok(types::portfolio::TvlSummary {
        total_tvl_usd: total_tvl,
        token_tvls: tokens,
        timestamp: ic_cdk::api::time(),
    })
}

#[query]
#[candid_method(query)]
fn get_token_metadata() -> Result<Vec<types::tokens::TokenMetadata>> {
    use types::TrackedToken;

    let tokens: Vec<types::tokens::TokenMetadata> = vec![
        types::tokens::TokenMetadata {
            symbol: "ALEX".to_string(),
            canister_id: Principal::from_text("ysy5f-2qaaa-aaaap-qkmmq-cai").unwrap(),
            decimals: 8,
        },
        types::tokens::TokenMetadata {
            symbol: "ZERO".to_string(),
            canister_id: Principal::from_text("rffwt-piaaa-aaaaa-qaacq-cai").unwrap(),
            decimals: 8,
        },
        types::tokens::TokenMetadata {
            symbol: "KONG".to_string(),
            canister_id: Principal::from_text("73wnl-eqaaa-aaaal-qddaa-cai").unwrap(),
            decimals: 8,
        },
        types::tokens::TokenMetadata {
            symbol: "BOB".to_string(),
            canister_id: Principal::from_text("7pail-xaaaa-aaaas-aabmq-cai").unwrap(),
            decimals: 8,
        },
    ];
    Ok(tokens)
}

#[query]
#[candid_method(query)]
fn get_simple_test() -> String {
    format!("Backend is responding at {}", ic_cdk::api::time())
}

#[update]
#[candid_method(update)]
fn test_simple_update() -> String {
    format!("Update call succeeded at {}", ic_cdk::api::time())
}

#[update]
#[candid_method(update)]
async fn debug_rebalancer() -> String {
    format!("Rebalancer status: {:?}", _1_CRITICAL_OPERATIONS::rebalancing::get_rebalancer_status())
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

#[query]
#[candid_method(query)]
fn greet(name: String) -> String {
    format!("Hello, {}! Welcome to ICPI.", name)
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

#[update]
#[candid_method(update)]
async fn icrc1_total_supply() -> Nat {
    _2_CRITICAL_DATA::supply_tracker::get_icpi_supply_uncached().await
        .unwrap_or(Nat::from(0u64))
}

#[update]
#[candid_method(update)]
async fn icrc1_balance_of(account: types::icrc::Account) -> Nat {
    // Query ICPI token ledger for balance
    let icpi_canister_id = Principal::from_text("l6lep-niaaa-aaaap-qqeda-cai").unwrap();

    match ic_cdk::call::<(types::icrc::Account,), (Nat,)>(
        icpi_canister_id,
        "icrc1_balance_of",
        (account,)
    ).await {
        Ok((balance,)) => balance,
        Err(_) => Nat::from(0u64),
    }
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

#[query]
#[candid_method(query)]
fn get_all_balances() -> Vec<(Principal, Nat)> {
    // This would require maintaining a balance map, which we don't do
    // Return empty for now as balances are in the ICPI token canister
    vec![]
}

// ===== TESTING =====

#[update]
#[candid_method(update)]
async fn test_kong_liquidity() -> Result<String> {
    ic_cdk::println!("🧪 Testing Kong Liquidity integration...");

    // Test 1: Get TVL from Kong Locker
    let tvl_result = _3_KONG_LIQUIDITY::tvl::calculate_kong_locker_tvl().await;
    let tvl = match tvl_result {
        Ok(data) => data,
        Err(e) => return Err(IcpiError::Other(format!("TVL calculation failed: {}", e))),
    };

    // Test 2: Get ALEX price from Kongswap
    let alex_price = _3_KONG_LIQUIDITY::pools::get_token_price_in_usdt(&types::TrackedToken::ALEX).await?;

    // Format results
    let mut output = String::from("Kong Liquidity Integration Test Results:\n\n");
    output.push_str("TVL by Token:\n");
    for (token, usd_value) in &tvl {
        output.push_str(&format!("  {}: ${:.2}\n", token.to_symbol(), usd_value));
    }
    output.push_str(&format!("\nALEX Price: {} ckUSDT\n", alex_price));
    output.push_str("\n✅ Zone 3 integration working!");

    Ok(output)
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