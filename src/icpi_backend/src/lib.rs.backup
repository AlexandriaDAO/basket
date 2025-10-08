// Module declarations
mod types;
mod kong_locker;
mod kongswap;
mod tvl_calculator;
mod balance_tracker;
mod index_state;
mod rebalancer;
mod icpi_math;
mod icrc_types;
mod precision;
mod icpi_token;
mod minting;
mod burning;
mod ledger_client;

use types::{IndexState, HealthStatus, RebalanceAction, TrackedToken};
use rebalancer::{RebalancerStatus, start_rebalancing};
use candid::Principal;
use std::cell::RefCell;

// In-memory cache for index state (simple, no stable structures)
thread_local! {
    static INDEX_STATE_CACHE: RefCell<Option<IndexState>> = RefCell::new(None);
}

// Background refresh timer for index state cache
fn start_cache_refresh_timer() {
    ic_cdk::println!("Starting index state cache refresh timer (every 5 minutes)");
    ic_cdk_timers::set_timer_interval(
        std::time::Duration::from_secs(300), // 5 minutes
        || {
            ic_cdk::spawn(async {
                ic_cdk::println!("Refreshing index state cache...");
                match get_index_state().await {
                    Ok(_) => ic_cdk::println!("✓ Cache refreshed"),
                    Err(e) => ic_cdk::println!("✗ Cache refresh failed: {}", e),
                }
            });
        }
    );
}

// Initialize the canister
#[ic_cdk::init]
fn init() {
    ic_cdk::println!("ICPI Backend initialized");

    // Start the hourly rebalancing timer
    start_rebalancing();
    // Start cleanup timer for mint operations
    minting::start_cleanup_timer();
    // Start cache refresh timer
    start_cache_refresh_timer();
}

// Post-upgrade hook
#[ic_cdk::post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("ICPI Backend upgraded");
    // Restart the hourly rebalancing timer
    start_rebalancing();
    // Restart cleanup timer for mint operations
    minting::start_cleanup_timer();
    // Restart cache refresh timer
    start_cache_refresh_timer();
}

// ===== Public Query Endpoints =====

// Get complete index state (cached, fast)
#[ic_cdk::query]
fn get_index_state_cached() -> Result<IndexState, String> {
    INDEX_STATE_CACHE.with(|cache| {
        cache.borrow()
            .clone()
            .ok_or_else(|| "Cache not initialized".to_string())
    })
}

// Get complete index state (slow, populates cache)
#[ic_cdk::update]  // Must be update for inter-canister calls
async fn get_index_state() -> Result<IndexState, String> {
    ic_cdk::println!("Getting index state...");
    let state = index_state::get_index_state().await?;

    // Populate cache
    INDEX_STATE_CACHE.with(|cache| {
        *cache.borrow_mut() = Some(state.clone());
    });

    Ok(state)
}

// Simple test endpoint that doesn't make inter-canister calls
#[ic_cdk::query]
fn get_simple_test() -> String {
    format!("Backend is responding at {}", ic_cdk::api::time())
}

// Test update call (to isolate issue)
#[ic_cdk::update]
fn test_simple_update() -> String {
    format!("Update call succeeded at {}", ic_cdk::api::time())
}

// Get rebalancing recommendation
#[ic_cdk::update]
async fn get_rebalancing_recommendation() -> Result<Option<RebalanceAction>, String> {
    ic_cdk::println!("Getting rebalancing recommendation...");
    index_state::get_rebalancing_recommendation().await
}

// Get TVL summary
#[ic_cdk::update]
async fn get_tvl_summary() -> Result<tvl_calculator::TVLSummary, String> {
    ic_cdk::println!("Getting TVL summary...");
    tvl_calculator::get_tvl_summary().await
}

// Get balance summary
#[ic_cdk::update]
async fn get_balance_summary() -> Result<balance_tracker::BalanceSummary, String> {
    ic_cdk::println!("Getting balance summary...");
    balance_tracker::get_balance_summary().await
}

// Get locked TVL by token
#[ic_cdk::update]
async fn get_locked_tvl_by_token() -> Result<Vec<(String, f64)>, String> {
    ic_cdk::println!("Getting locked TVL by token...");
    let tvl = tvl_calculator::calculate_locked_tvl().await?;

    let mut result = Vec::new();
    for (token, value) in tvl {
        result.push((
            token.to_symbol().to_string(),
            types::decimal_to_f64(value)
        ));
    }

    Ok(result)
}

// Get current positions
#[ic_cdk::update]
async fn get_current_positions() -> Result<Vec<types::CurrentPosition>, String> {
    ic_cdk::println!("Getting current positions...");
    balance_tracker::get_current_positions().await
}

// ===== Rebalancer Controls =====

// Manual rebalance trigger (for testing)
#[ic_cdk::update]
async fn trigger_manual_rebalance() -> Result<String, String> {
    ic_cdk::println!("Manual rebalance triggered");
    rebalancer::trigger_manual_rebalance().await
}

// Get rebalancer status
#[ic_cdk::query]
fn get_rebalancer_status() -> RebalancerStatus {
    rebalancer::get_rebalancer_status()
}

// Debug rebalancer logic (shows detailed state + logs action determination)
#[ic_cdk::update]
async fn debug_rebalancer() -> String {
    use index_state::{get_index_state, get_rebalancing_action};

    ic_cdk::println!("🔍 DEBUG: Manually checking rebalancer state...");

    let state = match get_index_state().await {
        Ok(s) => s,
        Err(e) => return format!("❌ Failed to get index state: {}", e),
    };

    // This will log detailed output via ic_cdk::println
    let action = get_rebalancing_action(&state.deviations, &state.ckusdt_balance);

    format!(
        "ckUSDT: {} (${:.2})\nTotal Value: ${:.2}\nAction: {:?}\n\nCheck canister logs for detailed breakdown",
        state.ckusdt_balance,
        state.ckusdt_balance.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0,
        state.total_value,
        action
    )
}

// Stop rebalancing
#[ic_cdk::update]
fn stop_rebalancing() -> String {
    rebalancer::stop_rebalancing();
    "Rebalancing stopped".to_string()
}

// Start rebalancing
#[ic_cdk::update]
fn start_rebalancing_timer() -> String {
    rebalancer::start_rebalancing();
    "Rebalancing started".to_string()
}

// ===== Health and Status =====

// Health check endpoint
#[ic_cdk::query]
fn get_health_status() -> HealthStatus {
    HealthStatus {
        version: "1.0.0".to_string(),
        tracked_tokens: TrackedToken::all().iter().map(|t| t.to_symbol().to_string()).collect(),
        last_rebalance: rebalancer::get_last_rebalance_time(),
        cycles_balance: ic_cdk::api::canister_balance128(),
    }
}

// ===== Test Endpoints =====

// Test kong_locker connection
#[ic_cdk::update]
async fn test_kong_locker_connection() -> Result<String, String> {
    ic_cdk::println!("Testing kong_locker connection...");
    let canisters = kong_locker::get_all_lock_canisters().await?;
    Ok(format!("Successfully retrieved {} lock canisters", canisters.len()))
}

// Test Kongswap connection
#[ic_cdk::update]
async fn test_kongswap_connection() -> Result<String, String> {
    ic_cdk::println!("Testing Kongswap connection...");
    use types::TrackedToken;
    let price = kongswap::get_token_price_in_usdt(&TrackedToken::ALEX).await?;
    Ok(format!("ALEX price: {} USDT", price))
}

// Test ICRC1 balance query
#[ic_cdk::update]
async fn test_balance_query() -> Result<String, String> {
    ic_cdk::println!("Testing balance query...");
    use types::TrackedToken;
    let balance = balance_tracker::get_token_balance(&TrackedToken::ALEX).await?;
    Ok(format!("ALEX balance: {}", balance))
}

// Clear caches (for testing)
#[ic_cdk::update]
fn clear_all_caches() -> String {
    kong_locker::clear_lock_canisters_cache();
    "All caches cleared".to_string()
}

// Get cache statistics
#[ic_cdk::query]
fn get_cache_stats() -> String {
    let (cached, last_updated, count) = kong_locker::get_cache_stats();
    format!(
        "Lock canisters cache: cached={}, last_updated={:?}, count={:?}",
        cached, last_updated, count
    )
}

// Get tracked tokens list
#[ic_cdk::query]
fn get_tracked_tokens() -> Vec<String> {
    TrackedToken::all().iter().map(|t| t.to_symbol().to_string()).collect()
}

// Get token metadata for frontend balance queries
#[ic_cdk::query]
fn get_token_metadata() -> Result<Vec<types::TokenMetadata>, String> {

    let tokens: Vec<types::TokenMetadata> = TrackedToken::all()
        .iter()
        .map(|token| {
            Ok(types::TokenMetadata {
                symbol: token.to_symbol().to_string(),
                canister_id: token.get_canister_id()?,
                decimals: token.get_decimals(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(tokens)
}

// ===== Utility Functions =====

// Get canister ID
#[ic_cdk::query]
fn get_canister_id() -> Principal {
    ic_cdk::id()
}

// Get cycles balance
#[ic_cdk::query]
fn get_cycles_balance() -> u128 {
    ic_cdk::api::canister_balance128()
}

// Legacy greet function (for compatibility)
#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}! Welcome to ICPI.", name)
}

// ===== ICRC1 Token Endpoints =====

// Re-export ICRC1 methods
pub use icpi_token::{
    icrc1_name, icrc1_symbol, icrc1_decimals, icrc1_total_supply,
    icrc1_balance_of, icrc1_fee, icrc1_metadata, icrc1_supported_standards,
    get_all_balances,
};

// ===== Minting Endpoints =====

#[ic_cdk::update]
async fn initiate_mint(ckusdt_amount: candid::Nat) -> Result<String, String> {
    minting::initiate_mint(ckusdt_amount).await
}

#[ic_cdk::update]
async fn complete_mint(mint_id: String) -> Result<candid::Nat, String> {
    minting::complete_mint(mint_id).await
}

#[ic_cdk::query]
fn check_mint_status(mint_id: String) -> Result<minting::MintStatus, String> {
    minting::check_mint_status(mint_id)
}

// ===== Burning Endpoint =====

#[ic_cdk::update]
async fn burn_icpi(amount: candid::Nat) -> Result<burning::BurnResult, String> {
    burning::burn_icpi(amount).await
}