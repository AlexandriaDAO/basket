//! Admin Controls Module
//!
//! Provides emergency pause, manual operation triggers, and admin logging
//! for the ICPI backend canister.
//!
//! Security Note: Admin principals are hardcoded and verified at runtime.
//! Only controller/deployer principals should have admin access.

use candid::Principal;
use std::cell::RefCell;
use crate::infrastructure::{IcpiError, Result};

/// Admin principals allowed to call admin functions
///
/// Includes:
/// - Backend canister itself (for timer-triggered operations)
/// - Deployer principal (for manual interventions)
const ADMIN_PRINCIPALS: &[&str] = &[
    "ev6xm-haaaa-aaaap-qqcza-cai",  // Backend (for timers)
    "67ktx-ln42b-uzmo5-bdiyn-gu62c-cd4h4-a5qt3-2w3rs-cixdl-iaso2-mqe",  // Deployer
];

/// Require caller is an admin principal
pub fn require_admin() -> Result<()> {
    let caller = ic_cdk::caller();

    let is_admin = ADMIN_PRINCIPALS.iter().any(|p| {
        Principal::from_text(p)
            .map(|admin| admin == caller)
            .unwrap_or(false)
    });

    if is_admin {
        Ok(())
    } else {
        Err(IcpiError::System(crate::infrastructure::errors::SystemError::Unauthorized {
            principal: caller.to_text(),
            required_role: "admin".to_string(),
        }))
    }
}

/// Emergency pause state
///
/// **CRITICAL UPGRADE BEHAVIOR**: This thread-local state is NOT persisted across canister upgrades.
///
/// **What happens on upgrade:**
/// - Emergency pause resets to `false` (system becomes unpaused)
/// - Admin must manually re-enable pause after upgrade if still needed
///
/// **Impact scenarios:**
/// 1. Admin activates pause → investigates issue → deploys fix → **pause silently disabled**
/// 2. Admin activates pause → unrelated upgrade → **pause silently disabled**
///
/// **Mitigation:**
/// - Document this behavior clearly (done)
/// - Admin should check pause state immediately after upgrade
/// - Consider implementing stable storage in future (Phase 5)
///
/// **Why thread-local:**
/// - Simple implementation for MVP
/// - Pause is typically temporary (minutes/hours, not days)
/// - Full stable storage implementation deferred to Phase 5
thread_local! {
    static EMERGENCY_PAUSE: RefCell<bool> = RefCell::new(false);
}

/// Admin action log entry
#[derive(Clone, candid::CandidType, candid::Deserialize, serde::Serialize)]
pub struct AdminAction {
    pub timestamp: u64,
    pub admin: Principal,
    pub action: String,
}

/// Admin action log storage
thread_local! {
    static ADMIN_LOG: RefCell<Vec<AdminAction>> = RefCell::new(Vec::new());
}

const MAX_LOG_ENTRIES: usize = 1000;

/// Log an admin action
pub fn log_admin_action(action: String) {
    ADMIN_LOG.with(|log| {
        let mut log = log.borrow_mut();

        log.push(AdminAction {
            timestamp: ic_cdk::api::time(),
            admin: ic_cdk::caller(),
            action: action.clone(),
        });

        // Keep only last 1000 entries
        let len = log.len();
        if len > MAX_LOG_ENTRIES {
            log.drain(0..(len - MAX_LOG_ENTRIES));
        }
    });

    ic_cdk::println!("📝 Admin action: {} by {}", action, ic_cdk::caller());
}

/// Check if system is paused
pub fn check_not_paused() -> Result<()> {
    EMERGENCY_PAUSE.with(|p| {
        if *p.borrow() {
            Err(IcpiError::System(crate::infrastructure::errors::SystemError::EmergencyPause))
        } else {
            Ok(())
        }
    })
}

/// Activate emergency pause
pub fn set_pause(paused: bool) {
    EMERGENCY_PAUSE.with(|p| *p.borrow_mut() = paused);
}

/// Get current pause state
pub fn is_paused() -> bool {
    EMERGENCY_PAUSE.with(|p| *p.borrow())
}

/// Get admin action log
pub fn get_admin_log() -> Vec<AdminAction> {
    ADMIN_LOG.with(|log| log.borrow().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_principals_valid() {
        for principal_text in ADMIN_PRINCIPALS {
            assert!(
                Principal::from_text(principal_text).is_ok(),
                "Invalid principal: {}",
                principal_text
            );
        }
    }

    #[test]
    fn test_pause_state_default() {
        // Initial state should be not paused
        assert!(!is_paused());
    }

    #[test]
    fn test_pause_toggle() {
        set_pause(true);
        assert!(is_paused());

        set_pause(false);
        assert!(!is_paused());
    }

    #[test]
    fn test_check_not_paused() {
        set_pause(false);
        assert!(check_not_paused().is_ok());

        set_pause(true);
        assert!(check_not_paused().is_err());

        // Reset for other tests
        set_pause(false);
    }
}
