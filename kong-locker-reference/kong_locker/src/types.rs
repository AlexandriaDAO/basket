use candid::{CandidType, Deserialize, Nat, Principal};

/// Detailed canister status with cycles and controller info
#[derive(CandidType, Deserialize)]
pub struct DetailedCanisterStatus {
    pub canister_id: Principal,
    pub is_blackholed: bool,
    pub controller_count: u32,
    pub cycle_balance: Nat,
    pub memory_size: Nat,
    pub module_hash: Option<Vec<u8>>,
}

/// Analytics overview for all lock canisters
#[derive(CandidType, Deserialize)]
pub struct AnalyticsOverview {
    pub total_lock_canisters: u64,
    pub participants: Vec<(Principal, Principal)>, // (user, lock_canister)
    pub last_updated: u64,
}