//! Kong Locker Integration
//!
//! Queries kong_locker backend for:
//! - List of all lock canisters
//! - User balances across lock canisters
//!
//! Used for calculating TVL from locked liquidity positions.

use candid::Principal;
use crate::infrastructure::{Result, IcpiError, KONG_LOCKER_ID};

/// Get all lock canisters from kong_locker
///
/// Returns: Vec<(user_principal, lock_canister_principal)>
///
/// This queries the kong_locker backend which tracks all created lock canisters.
/// Each user can have one lock canister that holds their LP tokens.
pub async fn get_all_lock_canisters() -> Result<Vec<(Principal, Principal)>> {
    let kong_locker = Principal::from_text(KONG_LOCKER_ID)
        .map_err(|e| IcpiError::Other(format!("Invalid kong_locker canister ID: {}", e)))?;

    let (canisters,): (Vec<(Principal, Principal)>,) = ic_cdk::call(
        kong_locker,
        "get_all_lock_canisters",
        ()
    ).await.map_err(|e| {
        ic_cdk::println!("Failed to query kong_locker.get_all_lock_canisters: {:?}", e);
        IcpiError::Other(format!("Kong Locker query failed: {:?}", e.1))
    })?;

    ic_cdk::println!("✅ Retrieved {} lock canisters from kong_locker", canisters.len());
    Ok(canisters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kong_locker_canister_id() {
        assert!(Principal::from_text(KONG_LOCKER_ID).is_ok());
    }
}
