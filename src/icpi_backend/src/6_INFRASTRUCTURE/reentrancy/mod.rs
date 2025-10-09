//! Reentrancy guards for critical operations
//! Prevents concurrent execution of sensitive financial operations
//!
//! ## Two Layers of Protection
//!
//! ### Layer 1: Per-User Guards (MintGuard, BurnGuard)
//! - Prevents single user from initiating multiple concurrent operations
//! - Allows different users to operate simultaneously
//! - Fine-grained concurrency control
//!
//! ### Layer 2: Global Operation Coordination (GlobalOperation)
//! - Prevents rebalancing during active mints/burns
//! - Enforces grace period between operation type switches
//! - Coarse-grained system-wide coordination
//!
//! Example: User A and User B can mint simultaneously (Layer 1 allows),
//! but rebalancing will skip if either is active (Layer 2 blocks).

use std::cell::RefCell;
use std::collections::HashSet;
use crate::infrastructure::{Result, IcpiError, SystemError};
use candid::Principal;

// === GLOBAL OPERATION COORDINATION (M-4) ===

/// Global operation states for system-wide coordination
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalOperation {
    /// No operations currently active
    Idle,
    /// At least one mint operation active (any user)
    Minting,
    /// At least one burn operation active (any user)
    Burning,
    /// Rebalancing operation active
    Rebalancing,
}

impl GlobalOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            GlobalOperation::Idle => "idle",
            GlobalOperation::Minting => "minting",
            GlobalOperation::Burning => "burning",
            GlobalOperation::Rebalancing => "rebalancing",
        }
    }
}

/// Grace period between operation type switches (60 seconds)
const GRACE_PERIOD_NANOS: u64 = 60_000_000_000;

thread_local! {
    /// Track active minting operations by user
    static ACTIVE_MINTS: RefCell<HashSet<Principal>> = RefCell::new(HashSet::new());

    /// Track active burning operations by user
    static ACTIVE_BURNS: RefCell<HashSet<Principal>> = RefCell::new(HashSet::new());

    /// Current global operation state
    static CURRENT_GLOBAL_OPERATION: RefCell<GlobalOperation> = RefCell::new(GlobalOperation::Idle);

    /// Timestamp when last operation ended (for grace period)
    static LAST_OPERATION_END_TIME: RefCell<u64> = RefCell::new(0);
}

/// Guard for minting operations
pub struct MintGuard {
    user: Principal,
}

impl MintGuard {
    /// Acquire a mint guard for the user
    pub fn acquire(user: Principal) -> Result<Self> {
        let acquired = ACTIVE_MINTS.with(|mints| {
            let mut mints = mints.borrow_mut();
            if mints.contains(&user) {
                false // Already minting
            } else {
                mints.insert(user);
                true
            }
        });

        if acquired {
            Ok(MintGuard { user })
        } else {
            Err(IcpiError::System(SystemError::OperationInProgress {
                operation: "mint".to_string(),
                user: user.to_text(),
            }))
        }
    }
}

impl Drop for MintGuard {
    fn drop(&mut self) {
        ACTIVE_MINTS.with(|mints| {
            mints.borrow_mut().remove(&self.user);
        });
    }
}

/// Guard for burning operations
pub struct BurnGuard {
    user: Principal,
}

impl BurnGuard {
    /// Acquire a burn guard for the user
    pub fn acquire(user: Principal) -> Result<Self> {
        let acquired = ACTIVE_BURNS.with(|burns| {
            let mut burns = burns.borrow_mut();
            if burns.contains(&user) {
                false // Already burning
            } else {
                burns.insert(user);
                true
            }
        });

        if acquired {
            Ok(BurnGuard { user })
        } else {
            Err(IcpiError::System(SystemError::OperationInProgress {
                operation: "burn".to_string(),
                user: user.to_text(),
            }))
        }
    }
}

impl Drop for BurnGuard {
    fn drop(&mut self) {
        ACTIVE_BURNS.with(|burns| {
            burns.borrow_mut().remove(&self.user);
        });
    }
}

// === GLOBAL OPERATION COORDINATION FUNCTIONS ===

/// Try to start a global operation
///
/// This enforces:
/// 1. Grace period between different operation types
/// 2. Rebalancing blocked during mints/burns
/// 3. Mints/burns can coexist (per-user guards handle conflicts)
///
/// Returns Ok if operation can proceed, Err if blocked
pub fn try_start_global_operation(op: GlobalOperation) -> Result<()> {
    CURRENT_GLOBAL_OPERATION.with(|current| {
        let current_op = *current.borrow();

        // Check grace period (except when transitioning from Idle)
        if current_op != GlobalOperation::Idle && current_op != op {
            LAST_OPERATION_END_TIME.with(|last| {
                let last_end = *last.borrow();
                let now = ic_cdk::api::time();

                if last_end > 0 && now > last_end && (now - last_end) < GRACE_PERIOD_NANOS {
                    let wait_seconds = (GRACE_PERIOD_NANOS - (now - last_end)) / 1_000_000_000;
                    return Err(IcpiError::System(SystemError::GracePeriodActive {
                        wait_seconds,
                        current_operation: current_op.as_str().to_string(),
                    }));
                }
                Ok(())
            })?;
        }

        // Check operation conflicts
        match (current_op, op) {
            // Idle → any operation OK (including back to Idle as no-op)
            (GlobalOperation::Idle, _) => {
                if op != GlobalOperation::Idle {
                    *current.borrow_mut() = op;
                    ic_cdk::println!("🔒 Global operation started: {:?}", op);
                }
                Ok(())
            },

            // Any state → Idle: Invalid (use end_global_operation instead)
            (_, GlobalOperation::Idle) => {
                ic_cdk::println!("⚠️  WARNING: Cannot transition to Idle via try_start_global_operation, use end_global_operation instead");
                Err(IcpiError::System(SystemError::StateCorrupted {
                    reason: "Invalid transition to Idle state".to_string(),
                }))
            },

            // Rebalancing blocks new mints/burns (but existing ones can finish)
            (GlobalOperation::Rebalancing, GlobalOperation::Minting) |
            (GlobalOperation::Rebalancing, GlobalOperation::Burning) => {
                Err(IcpiError::System(SystemError::RebalancingInProgress))
            },

            // Mints/burns block new rebalancing
            (GlobalOperation::Minting, GlobalOperation::Rebalancing) |
            (GlobalOperation::Burning, GlobalOperation::Rebalancing) => {
                Err(IcpiError::System(SystemError::CriticalOperationInProgress {
                    operation: current_op.as_str().to_string(),
                }))
            },

            // Mints and burns can coexist (per-user guards prevent same-user conflicts)
            (GlobalOperation::Minting, GlobalOperation::Minting) |
            (GlobalOperation::Burning, GlobalOperation::Burning) |
            (GlobalOperation::Minting, GlobalOperation::Burning) |
            (GlobalOperation::Burning, GlobalOperation::Minting) => {
                // Allow - per-user guards will handle concurrency
                Ok(())
            },

            // Same operation type - allow (multiple concurrent operations)
            (GlobalOperation::Rebalancing, GlobalOperation::Rebalancing) => {
                // Rebalancing timer should prevent this, but if it happens, allow
                ic_cdk::println!("⚠️  WARNING: Multiple rebalancing attempts detected");
                Ok(())
            },
        }
    })
}

/// End a global operation
///
/// Call this when operation completes (success or failure)
/// Records timestamp for grace period enforcement
pub fn end_global_operation(op: GlobalOperation) {
    CURRENT_GLOBAL_OPERATION.with(|current| {
        let current_op = *current.borrow();

        // Only transition to Idle if we're ending the current operation
        // (Handles case where multiple mints/burns active - only go Idle when last one finishes)
        match (current_op, op) {
            // Ending rebalancing always transitions to Idle
            (GlobalOperation::Rebalancing, GlobalOperation::Rebalancing) => {
                *current.borrow_mut() = GlobalOperation::Idle;

                LAST_OPERATION_END_TIME.with(|last| {
                    *last.borrow_mut() = ic_cdk::api::time();
                });

                ic_cdk::println!("🔓 Global operation ended: {:?}", op);
            },

            // Ending mint/burn: check if any other mints/burns still active
            (GlobalOperation::Minting, GlobalOperation::Minting) => {
                let has_active_mints = ACTIVE_MINTS.with(|m| !m.borrow().is_empty());
                let has_active_burns = ACTIVE_BURNS.with(|b| !b.borrow().is_empty());

                if !has_active_mints && !has_active_burns {
                    *current.borrow_mut() = GlobalOperation::Idle;

                    LAST_OPERATION_END_TIME.with(|last| {
                        *last.borrow_mut() = ic_cdk::api::time();
                    });

                    ic_cdk::println!("🔓 Global operation ended: all mints/burns complete");
                }
            },

            (GlobalOperation::Burning, GlobalOperation::Burning) => {
                let has_active_mints = ACTIVE_MINTS.with(|m| !m.borrow().is_empty());
                let has_active_burns = ACTIVE_BURNS.with(|b| !b.borrow().is_empty());

                if !has_active_mints && !has_active_burns {
                    *current.borrow_mut() = GlobalOperation::Idle;

                    LAST_OPERATION_END_TIME.with(|last| {
                        *last.borrow_mut() = ic_cdk::api::time();
                    });

                    ic_cdk::println!("🔓 Global operation ended: all mints/burns complete");
                }
            },

            // Mismatched operation end (shouldn't happen, but handle gracefully)
            _ => {
                ic_cdk::println!("⚠️  WARNING: Attempted to end {:?} but current state is {:?}",
                    op, current_op);
            }
        }
    });
}

/// Get current global operation state (for monitoring/debugging)
pub fn get_current_operation() -> GlobalOperation {
    CURRENT_GLOBAL_OPERATION.with(|current| *current.borrow())
}

/// Check if any operations are active (for testing/monitoring)
pub fn has_active_operations() -> bool {
    let has_mints = ACTIVE_MINTS.with(|m| !m.borrow().is_empty());
    let has_burns = ACTIVE_BURNS.with(|b| !b.borrow().is_empty());
    let current_op = get_current_operation();

    has_mints || has_burns || current_op != GlobalOperation::Idle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mint_guard_prevents_reentrancy() {
        let user = Principal::anonymous();

        // First guard should succeed
        let _guard1 = MintGuard::acquire(user).expect("First guard should succeed");

        // Second guard for same user should fail
        let result = MintGuard::acquire(user);
        assert!(result.is_err());

        // Drop first guard
        drop(_guard1);

        // Now should succeed again
        let _guard2 = MintGuard::acquire(user).expect("Should succeed after drop");
    }

    #[test]
    fn test_burn_guard_prevents_reentrancy() {
        let user = Principal::anonymous();

        // First guard should succeed
        let _guard1 = BurnGuard::acquire(user).expect("First guard should succeed");

        // Second guard for same user should fail
        let result = BurnGuard::acquire(user);
        assert!(result.is_err());

        // Drop first guard
        drop(_guard1);

        // Now should succeed again
        let _guard2 = BurnGuard::acquire(user).expect("Should succeed after drop");
    }

    #[test]
    fn test_different_users_can_mint_concurrently() {
        let user1 = Principal::from_text("2vxsx-fae").unwrap();
        let user2 = Principal::from_text("aaaaa-aa").unwrap();

        // Both users should be able to acquire guards
        let _guard1 = MintGuard::acquire(user1).expect("User 1 should succeed");
        let _guard2 = MintGuard::acquire(user2).expect("User 2 should succeed");

        // Both guards active simultaneously
        assert!(ACTIVE_MINTS.with(|m| m.borrow().len() == 2));
    }

    // === M-4: Global Operation Coordination Tests ===

    #[test]
    fn test_idle_to_minting_transition() {
        // Reset state
        CURRENT_GLOBAL_OPERATION.with(|c| *c.borrow_mut() = GlobalOperation::Idle);

        let result = try_start_global_operation(GlobalOperation::Minting);
        assert!(result.is_ok(), "Should allow idle → minting");
        assert_eq!(get_current_operation(), GlobalOperation::Minting);
    }

    #[test]
    fn test_idle_to_rebalancing_transition() {
        // Reset state
        CURRENT_GLOBAL_OPERATION.with(|c| *c.borrow_mut() = GlobalOperation::Idle);

        let result = try_start_global_operation(GlobalOperation::Rebalancing);
        assert!(result.is_ok(), "Should allow idle → rebalancing");
        assert_eq!(get_current_operation(), GlobalOperation::Rebalancing);
    }

    #[test]
    fn test_minting_blocks_rebalancing() {
        // Reset state
        CURRENT_GLOBAL_OPERATION.with(|c| *c.borrow_mut() = GlobalOperation::Idle);

        // Start minting
        let _ = try_start_global_operation(GlobalOperation::Minting).unwrap();

        // Try to start rebalancing - should fail
        let result = try_start_global_operation(GlobalOperation::Rebalancing);
        assert!(result.is_err(), "Minting should block rebalancing");
        assert!(matches!(result, Err(IcpiError::System(SystemError::CriticalOperationInProgress { .. }))));
    }

    #[test]
    fn test_rebalancing_blocks_minting() {
        // Reset state
        CURRENT_GLOBAL_OPERATION.with(|c| *c.borrow_mut() = GlobalOperation::Idle);

        // Start rebalancing
        let _ = try_start_global_operation(GlobalOperation::Rebalancing).unwrap();

        // Try to start minting - should fail
        let result = try_start_global_operation(GlobalOperation::Minting);
        assert!(result.is_err(), "Rebalancing should block minting");
        assert!(matches!(result, Err(IcpiError::System(SystemError::RebalancingInProgress))));
    }

    #[test]
    fn test_minting_and_burning_can_coexist() {
        // Reset state
        CURRENT_GLOBAL_OPERATION.with(|c| *c.borrow_mut() = GlobalOperation::Idle);

        // Start minting
        let _ = try_start_global_operation(GlobalOperation::Minting).unwrap();

        // Start burning - should succeed
        let result = try_start_global_operation(GlobalOperation::Burning);
        assert!(result.is_ok(), "Minting and burning should coexist");

        // State should still be one of them (implementation detail)
        let current = get_current_operation();
        assert!(current == GlobalOperation::Minting || current == GlobalOperation::Burning);
    }

    #[test]
    fn test_multiple_mints_allowed() {
        // Reset state
        CURRENT_GLOBAL_OPERATION.with(|c| *c.borrow_mut() = GlobalOperation::Idle);

        // Start first mint
        let _ = try_start_global_operation(GlobalOperation::Minting).unwrap();

        // Start second mint - should succeed
        let result = try_start_global_operation(GlobalOperation::Minting);
        assert!(result.is_ok(), "Multiple concurrent mints should be allowed");
    }

    #[test]
    fn test_end_rebalancing_clears_state() {
        // Reset state
        CURRENT_GLOBAL_OPERATION.with(|c| *c.borrow_mut() = GlobalOperation::Idle);

        // Start and end rebalancing
        let _ = try_start_global_operation(GlobalOperation::Rebalancing).unwrap();
        assert_eq!(get_current_operation(), GlobalOperation::Rebalancing);

        end_global_operation(GlobalOperation::Rebalancing);
        assert_eq!(get_current_operation(), GlobalOperation::Idle);
    }

    #[test]
    fn test_cannot_transition_to_idle_via_try_start() {
        // Reset state
        CURRENT_GLOBAL_OPERATION.with(|c| *c.borrow_mut() = GlobalOperation::Minting);

        // Try to transition to Idle via try_start (invalid)
        let result = try_start_global_operation(GlobalOperation::Idle);
        assert!(result.is_err(), "Should not allow transitioning to Idle via try_start");
        assert!(matches!(result, Err(IcpiError::System(SystemError::StateCorrupted { .. }))));
    }

    #[test]
    fn test_has_active_operations() {
        // Reset state
        CURRENT_GLOBAL_OPERATION.with(|c| *c.borrow_mut() = GlobalOperation::Idle);
        ACTIVE_MINTS.with(|m| m.borrow_mut().clear());
        ACTIVE_BURNS.with(|b| b.borrow_mut().clear());

        // No operations active
        assert!(!has_active_operations());

        // Start minting
        let _ = try_start_global_operation(GlobalOperation::Minting).unwrap();
        assert!(has_active_operations());

        // End minting
        end_global_operation(GlobalOperation::Minting);
        // Note: Still might show active if per-user guards are active
    }

    #[test]
    fn test_global_operation_enum_as_str() {
        assert_eq!(GlobalOperation::Idle.as_str(), "idle");
        assert_eq!(GlobalOperation::Minting.as_str(), "minting");
        assert_eq!(GlobalOperation::Burning.as_str(), "burning");
        assert_eq!(GlobalOperation::Rebalancing.as_str(), "rebalancing");
    }
}
