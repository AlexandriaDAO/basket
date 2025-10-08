use ic_cdk::{query, update};
use candid::{Principal, Nat};
use ic_cdk::api::management_canister::main::{canister_status, CanisterIdRecord};

use crate::types::{DetailedCanisterStatus, AnalyticsOverview};
use crate::storage::{StorablePrincipal, USER_LOCK_CANISTERS};

/// Get user's lock canister
#[query]
pub fn get_my_lock_canister() -> Option<Principal> {
    let user = ic_cdk::caller();
    USER_LOCK_CANISTERS.with(|c| 
        c.borrow().get(&StorablePrincipal(user)).map(|sp| sp.0)
    )
}

/// Get all lock canisters
#[query]
pub fn get_all_lock_canisters() -> Vec<(Principal, Principal)> {
    USER_LOCK_CANISTERS.with(|c| {
        c.borrow().iter()
            .map(|(user, canister)| (user.0, canister.0))
            .collect()
    })
}

/// Get detailed status including cycles and blackhole verification
/// Note: This is an update call because it queries the management canister
#[update]
pub async fn get_detailed_canister_status() -> Result<DetailedCanisterStatus, String> {
    let user = ic_cdk::caller();
    
    // Get user's lock canister
    let canister_id = USER_LOCK_CANISTERS.with(|c| 
        c.borrow().get(&StorablePrincipal(user)).map(|sp| sp.0)
    ).ok_or("No lock canister found")?;
    
    // Query canister status from management canister
    let status_result = canister_status(CanisterIdRecord { canister_id }).await
        .map_err(|e| format!("Failed to get canister status: {:?}", e))?;
    
    let status = status_result.0;
    
    // Check if blackholed (no controllers)
    let is_blackholed = status.settings.controllers.is_empty();
    let controller_count = status.settings.controllers.len() as u32;
    
    Ok(DetailedCanisterStatus {
        canister_id,
        is_blackholed,
        controller_count,
        cycle_balance: status.cycles,
        memory_size: status.memory_size,
        module_hash: status.module_hash,
    })
}

/// Get total number of lock positions (unique users with lock canisters)
#[query]
pub fn get_total_positions_count() -> u64 {
    USER_LOCK_CANISTERS.with(|c| c.borrow().len() as u64)
}

/// Get analytics overview for all lock canisters
#[query]
pub fn get_analytics_overview() -> AnalyticsOverview {
    let participants = get_all_lock_canisters();
    let total_count = participants.len() as u64;
    let current_time = ic_cdk::api::time();
    
    AnalyticsOverview {
        total_lock_canisters: total_count,
        participants,
        last_updated: current_time,
    }
}
