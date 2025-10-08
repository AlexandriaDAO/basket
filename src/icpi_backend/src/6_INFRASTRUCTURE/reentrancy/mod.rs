//! Reentrancy guards for critical operations
//! Prevents concurrent execution of sensitive financial operations

use std::cell::RefCell;
use std::collections::HashSet;
use crate::infrastructure::{Result, IcpiError, SystemError};
use candid::Principal;

thread_local! {
    /// Track active minting operations by user
    static ACTIVE_MINTS: RefCell<HashSet<Principal>> = RefCell::new(HashSet::new());

    /// Track active burning operations by user
    static ACTIVE_BURNS: RefCell<HashSet<Principal>> = RefCell::new(HashSet::new());
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
}
