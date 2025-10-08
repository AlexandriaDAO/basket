//! Mint state management
//! Tracks pending mints and their status

use candid::{CandidType, Deserialize, Nat, Principal};
use std::cell::RefCell;
use std::collections::HashMap;
use crate::infrastructure::{Result, IcpiError, MintError};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum MintStatus {
    Pending,
    CollectingFee,
    Snapshotting,
    CollectingDeposit,
    Calculating,
    Minting,
    Refunding,
    Complete(Nat),
    Failed(String),
    FailedRefunded(String),
    FailedNoRefund(String),
    Expired,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MintSnapshot {
    pub supply: Nat,
    pub tvl: Nat,
    pub timestamp: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PendingMint {
    pub id: String,
    pub user: Principal,
    pub amount: Nat,
    pub status: MintStatus,
    pub created_at: u64,
    pub last_updated: u64,
    pub snapshot: Option<MintSnapshot>,
}

thread_local! {
    static PENDING_MINTS: RefCell<HashMap<String, PendingMint>> =
        RefCell::new(HashMap::new());
}

pub fn store_pending_mint(mint: PendingMint) -> Result<()> {
    PENDING_MINTS.with(|mints| {
        mints.borrow_mut().insert(mint.id.clone(), mint);
        Ok(())
    })
}

pub fn get_pending_mint(mint_id: &str) -> Result<Option<PendingMint>> {
    PENDING_MINTS.with(|mints| {
        Ok(mints.borrow().get(mint_id).cloned())
    })
}

pub fn update_mint_status(mint_id: &str, status: MintStatus) -> Result<()> {
    PENDING_MINTS.with(|mints| {
        let mut mints = mints.borrow_mut();
        match mints.get_mut(mint_id) {
            Some(mint) => {
                mint.status = status;
                mint.last_updated = ic_cdk::api::time();
                Ok(())
            }
            None => Err(IcpiError::Mint(MintError::InvalidMintId {
                id: mint_id.to_string(),
            }))
        }
    })
}

pub fn cleanup_expired_mints() -> Result<u32> {
    const TIMEOUT_NANOS: u64 = 180_000_000_000; // 3 minutes
    const COMPLETED_RETENTION_NANOS: u64 = 86_400_000_000_000; // 24 hours
    let now = ic_cdk::api::time();
    let mut cleaned = 0u32;

    PENDING_MINTS.with(|mints| {
        let mut mints = mints.borrow_mut();
        mints.retain(|_id, mint| {
            let age = now - mint.created_at;

            // Remove failed/expired pending mints after 3 minutes
            if age > TIMEOUT_NANOS && !matches!(mint.status, MintStatus::Complete(_)) {
                cleaned += 1;
                false
            }
            // Remove completed mints after 24 hours to prevent memory leak
            else if age > COMPLETED_RETENTION_NANOS && matches!(mint.status, MintStatus::Complete(_)) {
                cleaned += 1;
                ic_cdk::println!("Cleaned up completed mint {} after 24 hours", _id);
                false
            }
            else {
                true
            }
        });
    });

    Ok(cleaned)
}

pub fn get_pending_count() -> usize {
    PENDING_MINTS.with(|mints| {
        mints.borrow()
            .values()
            .filter(|m| matches!(m.status,
                MintStatus::Pending |
                MintStatus::CollectingFee |
                MintStatus::Snapshotting |
                MintStatus::CollectingDeposit |
                MintStatus::Calculating |
                MintStatus::Minting |
                MintStatus::Refunding))
            .count()
    })
}

pub fn export_state() -> HashMap<String, PendingMint> {
    PENDING_MINTS.with(|mints| {
        mints.borrow().clone()
    })
}

pub fn import_state(state: HashMap<String, PendingMint>) {
    PENDING_MINTS.with(|mints| {
        *mints.borrow_mut() = state;
    })
}