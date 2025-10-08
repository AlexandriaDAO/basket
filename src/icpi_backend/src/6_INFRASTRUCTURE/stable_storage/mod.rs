//! Stable storage management for upgrade persistence

use candid::{CandidType, Deserialize};
use std::collections::HashMap;
use crate::_1_CRITICAL_OPERATIONS::minting::mint_state::PendingMint;

#[derive(CandidType, Deserialize, Default)]
pub struct StableState {
    pub pending_mints: HashMap<String, PendingMint>,
}

pub fn save_state(pending_mints: HashMap<String, PendingMint>) {
    let state = StableState { pending_mints };
    ic_cdk::println!("💾 Saving {} pending mints to stable storage", state.pending_mints.len());

    // Handle serialization errors gracefully - log but don't panic
    // This is critical for production: if stable storage fails, we log the error
    // but continue operation. The pending mints will be lost on upgrade, but
    // the canister won't trap during upgrade.
    match ic_cdk::storage::stable_save((state,)) {
        Ok(_) => {
            ic_cdk::println!("✅ Successfully saved state to stable memory");
        }
        Err(e) => {
            ic_cdk::println!("⚠️ WARNING: Failed to save state to stable memory: {}", e);
            ic_cdk::println!("⚠️ Pending mints may be lost on upgrade, but canister will continue operating");
            // Don't panic - allow the upgrade to continue even if stable storage fails
            // This is better than trapping the entire canister upgrade
        }
    }
}

pub fn restore_state() -> HashMap<String, PendingMint> {
    match ic_cdk::storage::stable_restore::<(StableState,)>() {
        Ok((state,)) => {
            ic_cdk::println!("✅ Restored {} pending mints from stable storage", state.pending_mints.len());
            let now = ic_cdk::api::time();
            let cleaned: HashMap<_, _> = state.pending_mints.into_iter()
                .filter(|(id, mint)| {
                    let age = now.saturating_sub(mint.created_at);
                    let is_valid = age < 86_400_000_000_000; // 24 hours
                    if !is_valid {
                        ic_cdk::println!("Dropping expired mint {} from stable storage", id);
                    }
                    is_valid
                })
                .collect();
            cleaned
        }
        Err(e) => {
            ic_cdk::println!("⚠️  No stable state to restore (first deployment or empty): {}", e);
            HashMap::new()
        }
    }
}
