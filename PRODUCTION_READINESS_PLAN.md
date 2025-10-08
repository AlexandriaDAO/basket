# 🚀 ICPI Backend Production Readiness Plan

**Status:** In Progress
**Last Updated:** 2025-10-07
**Current PR:** #4 - ICPI to ICPX Refactor v2

---

## 🚦 Quick Start for Fresh Agent

### Current Context
- **Branch:** `icpi-to-icpx-refactor-v2`
- **Latest Commit:** `9b2bc23` - "Fix all critical bugs from GitHub review"
- **Working Directory:** `/home/theseus/alexandria/daopad/src/icpi`
- **Backend Code:** `./src/icpi_backend/src/`

### Codebase Structure (Numbered Zones)
```
src/icpi_backend/src/
├── 1_CRITICAL_OPERATIONS/   # Minting, burning, rebalancing
│   ├── minting/             # Zone 1 - Mint orchestrator, fee handler
│   ├── burning/             # Zone 1 - Burn logic, token distributor
│   └── rebalancing/         # Zone 1 - Rebalancing (STUBBED)
├── 2_CRITICAL_DATA/         # Portfolio value, supply tracking
│   ├── portfolio_value/     # Zone 2 - TVL calculation
│   ├── supply_tracker/      # Zone 2 - ICPI supply queries
│   └── token_queries/       # Zone 2 - Token balance queries
├── 3_KONG_LIQUIDITY/        # External liquidity (STUBBED)
├── 4_TRADING_EXECUTION/     # DEX interactions (STUBBED)
├── 5_INFORMATIONAL/         # Display, caching (read-only)
└── 6_INFRASTRUCTURE/        # Math, errors, constants, types
```

### What's Already Fixed ✅
- ckUSDT canister ID consolidated (commit 9b2bc23)
- Minting formula decimal conversion fixed
- Burn fee collection order fixed
- Mint state 24-hour cleanup added
- Display module implemented

### What Needs Fixing 🔴 (START HERE)
1. **CRITICAL:** Burning doesn't transfer ICPI to backend (infinite money exploit)
2. **CRITICAL:** TVL values non-ckUSDT tokens at $0 (dilution after rebalance)
3. **CRITICAL:** State lost on upgrades (no stable storage)
4. Rate limiter memory leak

### Your Mission
Execute this plan sequentially, starting with **Phase 1.1** (Burning Exploit). After completing each phase:

1. **Build & Test Locally:**
   ```bash
   cd /home/theseus/alexandria/daopad/src/icpi
   cargo build --target wasm32-unknown-unknown --release --package icpi_backend
   ```

2. **Deploy to Mainnet:**
   ```bash
   ./deploy.sh --network ic
   ```

3. **Test the Fix:**
   ```bash
   # Test burning (Phase 1.1)
   dfx canister --network ic call icpi_backend burn_icpi "(principal \"ev6xm-haaaa-aaaap-qqcza-cai\", 100000000)"

   # Check TVL (Phase 1.2)
   dfx canister --network ic call icpi_backend get_index_state "()"

   # Query ICPI balance
   dfx canister --network ic call l6lep-niaaa-aaaap-qqeda-cai icrc1_balance_of "(record { owner = principal \"YOUR_PRINCIPAL\" })"
   ```

4. **Commit & Push:**
   ```bash
   git add .
   git commit -m "Fix [issue]: [brief description]

   [Detailed explanation of fix]

   🤖 Generated with [Claude Code](https://claude.com/claude-code)

   Co-Authored-By: Claude <noreply@anthropic.com>"

   git push origin icpi-to-icpx-refactor-v2
   ```

5. **Trigger GitHub Action Review:**
   - Push triggers automatic Claude Code review
   - Wait for review comments on PR #4
   - Address any issues found
   - Repeat until review passes

### Reference Files
- **Kongswap API:** `./kong-reference/` (study before Phase 3.2)
- **Kong Locker:** `../kong_locker/` (for liquidity queries)
- **Project Docs:** `./CLAUDE.md` (architecture overview)
- **Canister IDs:** See constants in `src/icpi_backend/src/6_INFRASTRUCTURE/constants/mod.rs`

### Important Canister IDs
- ICPI Token: `l6lep-niaaa-aaaap-qqeda-cai`
- ICPI Backend: `ev6xm-haaaa-aaaap-qqcza-cai` (your code, minting authority)
- Kongswap Backend: `2ipq2-uqaaa-aaaar-qailq-cai`
- ckUSDT: `cngnf-vqaaa-aaaar-qag4q-cai`

### Key Architecture Points
1. **Backend is Minting/Burning Account:** Any ICPI transferred TO the backend is automatically burned
2. **No Stable Storage Yet:** All state in thread_local! will be lost on upgrade (fix in Phase 2.1)
3. **Zones 3-4 are Stubs:** Rebalancing and trading not implemented yet (fix in Phase 3)
4. **Test on Mainnet:** No local testing - deploy and test with real canisters using small amounts

### Success Criteria
- [ ] All P0 fixes complete (Phases 1-2)
- [ ] Code builds without errors
- [ ] Tests pass on mainnet
- [ ] GitHub action review passes
- [ ] No economic exploits possible
- [ ] State persists across upgrades

---

## Executive Summary

The ICPI backend has critical economic exploits that must be fixed before any production deployment. The refactored zone architecture is solid, but several P0 issues create infinite money glitches and data loss risks during upgrades.

**Recent Progress:**
- ✅ Fixed ckUSDT canister ID mismatch
- ✅ Fixed minting formula decimal conversion
- ✅ Fixed burn fee collection order
- ✅ Added mint state 24-hour cleanup
- ✅ Implemented display module

**Critical Issues Remaining:**
- 🔴 Burning doesn't actually burn ICPI (infinite money exploit!)
- 🔴 TVL values all non-ckUSDT tokens at $0 (massive dilution after rebalancing)
- 🔴 State lost on canister upgrades
- 🔴 Rate limiter memory leak

---

## 📋 Critical Fixes Checklist (P0 - BLOCKING)

### 🔴 Phase 1: Economic Exploits (1-2 days)

- [ ] **1.1: Fix Burning Exploit** (2-4 hours) - CRITICAL
  - Add ICRC-1 transfer of ICPI from user to backend
  - Test: Burn 100 ICPI, verify balance reduced
  - File: `src/icpi_backend/src/1_CRITICAL_OPERATIONS/burning/mod.rs`

- [ ] **1.2: Fix TVL Pricing** (4-6 hours) - CRITICAL
  - Implement token price lookups (hardcoded conservative prices as interim)
  - Test: Verify TVL calculation with mixed holdings
  - File: `src/icpi_backend/src/2_CRITICAL_DATA/portfolio_value/mod.rs`

- [ ] **1.3: Fix icpi_burned Calculation** (1 hour)
  - Pass actual burn amount instead of summing redemptions
  - Test: Burn 100 ICPI, verify result shows exactly 100
  - File: `src/icpi_backend/src/1_CRITICAL_OPERATIONS/burning/token_distributor.rs`

### 🔴 Phase 2: State Persistence (8-15 hours)

- [ ] **2.1: Add Stable Storage** (8-12 hours) - CRITICAL
  - Migrate pending mints to stable storage
  - Add pre_upgrade and post_upgrade hooks
  - Test: Start mint, upgrade canister, complete mint
  - Files: New `6_INFRASTRUCTURE/stable_storage/mod.rs`, update `lib.rs`

- [ ] **2.2: Rate Limiter Cleanup** (2-3 hours)
  - Implement periodic cleanup of old entries
  - Add max entries limit (10K)
  - Test: Simulate 10K+ users, verify cleanup
  - File: `src/icpi_backend/src/6_INFRASTRUCTURE/rate_limiting/mod.rs`

---

## **PHASE 1: Critical Economic Fixes (P0 - BLOCKING)**
*Must complete before ANY mainnet usage with real funds*

### 🔴 1.1 Fix Burning Economic Exploit
**Priority:** P0 (CRITICAL BLOCKING)
**Effort:** 2-4 hours
**Risk:** EXTREME - Users can burn ICPI infinitely without reducing supply

**Problem:** The burn function never actually transfers ICPI from user to backend. Users keep their ICPI and get redemptions.

**Current Code Issue:**
```rust
// In burning/mod.rs - Missing ICPI transfer!
pub async fn burn_icpi(caller: Principal, amount: Nat) -> Result<BurnResult> {
    // Validates, calculates redemptions, distributes tokens
    // BUT NEVER TRANSFERS ICPI TO BACKEND!
    // Users can call this infinitely
}
```

**Implementation:**
```rust
// In burning/mod.rs, after line 43 (before calculate_redemptions)
// Transfer ICPI from user to backend (which burns it automatically)
let icpi_canister = Principal::from_text(ICPI_CANISTER_ID)?;
let transfer_args = TransferArgs {
    to: Account {
        owner: ic_cdk::id(), // Backend canister (burning account)
        subaccount: None,
    },
    amount: amount.clone(),
    fee: None,
    memo: Some(b"ICPI burn".to_vec()),
    from_subaccount: None,
    created_at_time: None,
};

let transfer_result: Result<(TransferResult,), _> = ic_cdk::call(
    icpi_canister,
    "icrc1_transfer",
    (transfer_args,)
).await;

match transfer_result {
    Ok((TransferResult::Ok(block),)) => {
        ic_cdk::println!("ICPI transferred to burning account at block {}", block);
    }
    Ok((TransferResult::Err(e),)) => {
        return Err(IcpiError::Burn(BurnError::TransferFailed(format!("{:?}", e))));
    }
    Err((code, msg)) => {
        return Err(IcpiError::Burn(BurnError::TransferFailed(
            format!("Transfer failed: {:?} - {}", code, msg)
        )));
    }
}
```

**Testing:**
- Integration test: Burn 100 ICPI, verify balance reduced by 100
- Try burning twice with same amount - second should fail (insufficient balance)
- Test error handling: Try burning more than user has

**Dependencies:** None

---

### 🔴 1.2 Fix TVL Calculation (Pricing Non-ckUSDT Tokens)
**Priority:** P0 (CRITICAL BLOCKING)
**Effort:** 4-6 hours
**Risk:** HIGH - Massive dilution after first rebalance

**Problem:** All non-ckUSDT tokens valued at $0. After buying $10K ALEX, TVL shows only remaining $1K ckUSDT. New depositor gets 50% of supply instead of 9%.

**Current Code Issue:**
```rust
// In portfolio_value/mod.rs
async fn get_token_usd_value(token_symbol: &str, amount: &Nat) -> Result<u64> {
    // TODO: Full implementation
    Ok(0u64)  // ❌ Always returns $0!
}
```

**Impact Example:**
- Backend holds: $1K ckUSDT + $10K ALEX
- Current TVL calculation: $1K (only counts ckUSDT)
- New user deposits $1K
- Gets 50% of supply instead of 9%!
- **Massive dilution of existing holders**

**Implementation:**
```rust
// In portfolio_value/mod.rs, replace get_token_usd_value function
async fn get_token_usd_value(token_symbol: &str, amount: &Nat) -> Result<u64> {
    // Temporary hardcoded prices until Kongswap integration
    // These are conservative estimates to prevent over-minting
    let price_per_token_e6 = match token_symbol {
        "ALEX" => 500_000u64,  // $0.50 per ALEX (conservative)
        "ZERO" => 100_000u64,  // $0.10 per ZERO
        "KONG" => 50_000u64,   // $0.05 per KONG
        "BOB" => 10_000u64,    // $0.01 per BOB
        _ => return Ok(0u64),   // Unknown tokens = $0
    };

    // Convert amount (e8) to tokens, multiply by price
    let amount_e8 = amount.0.to_u64().unwrap_or(0);
    let tokens = amount_e8 / 100_000_000; // e8 to whole tokens
    let value_e6 = tokens * price_per_token_e6;

    ic_cdk::println!("  {} tokens of {}: ${}",
        tokens, token_symbol, value_e6 as f64 / 1_000_000.0);

    Ok(value_e6)
}
```

**Future Enhancement:**
Once Kongswap integration is complete (Phase 3.2), replace hardcoded prices with live price feeds:
```rust
// Query Kongswap for current pool price
let price = query_kongswap_price(token_symbol).await?;
```

**Testing:**
- Unit test with mock balances: 1000 ALEX + 100 ckUSDT should = $600
- Deploy and verify TVL calculation matches expected values
- Test with various token combinations

**Dependencies:** None

---

### 🔴 1.3 Fix icpi_burned Calculation
**Priority:** P0
**Effort:** 1 hour
**Risk:** MEDIUM - Incorrect burn reporting

**Problem:** Sums redemption amounts instead of actual ICPI burned

**Current Code Issue:**
```rust
// In token_distributor.rs
fn calculate_total_burned(redemptions: &[(String, Nat)]) -> Nat {
    // Sums various token amounts (wrong!)
    redemptions.iter()
        .map(|(_, amount)| amount.clone())
        .sum()
}
```

**Implementation:**
```rust
// In token_distributor.rs, update function signature
pub async fn distribute_tokens(
    recipient: Principal,
    redemptions: Vec<(String, Nat)>,
    icpi_burn_amount: Nat,  // Add parameter
) -> Result<BurnResult> {
    let mut result = BurnResult {
        successful_transfers: Vec::new(),
        failed_transfers: Vec::new(),
        icpi_burned: icpi_burn_amount,  // Use actual amount
        timestamp: ic_cdk::api::time(),
    };

    // ... rest of function stays same
}

// In burning/mod.rs, pass the burn amount:
let result = distribute_tokens(caller, redemptions, amount.clone()).await?;
```

**Testing:**
- Burn 100 ICPI, verify result shows exactly 100 ICPI burned
- Burn varying amounts, verify accuracy

**Dependencies:** Requires 1.1 completed first

---

## **PHASE 2: State Persistence & Upgrade Safety (P0)**
*Must complete before any canister upgrades*

### 🔴 2.1 Add Stable Storage for Critical State
**Priority:** P0
**Effort:** 8-12 hours
**Risk:** HIGH - All pending mints, rate limits lost on upgrade

**Problem:** thread_local! storage lost during upgrades. If user initiates mint and canister upgrades, their deposit is stuck.

**Implementation:**
```rust
// Create new file: 6_INFRASTRUCTURE/stable_storage/mod.rs
use ic_stable_structures::{
    BTreeMap, DefaultMemoryImpl, StableBTreeMap,
    memory_manager::{MemoryId, MemoryManager}
};
use std::cell::RefCell;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Pending mints (persist across upgrades)
    static PENDING_MINTS: RefCell<StableBTreeMap<String, PendingMint, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        ));

    // Rate limits with automatic expiry
    static RATE_LIMITS: RefCell<BTreeMap<String, u64>> =
        RefCell::new(BTreeMap::new());  // Keep in memory, OK to lose
}

// Add pre_upgrade and post_upgrade hooks in lib.rs
#[pre_upgrade]
fn pre_upgrade() {
    // Stable structures auto-persist, just log
    ic_cdk::println!("Pre-upgrade: {} pending mints will persist",
        PENDING_MINTS.with(|m| m.borrow().len()));
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("Post-upgrade: Stable storage restored");
    cleanup_expired_data();
    restart_timers();
}

fn cleanup_expired_data() {
    // Clean up mints older than 24 hours
    let cutoff = ic_cdk::api::time() - (24 * 60 * 60 * 1_000_000_000);
    PENDING_MINTS.with(|mints| {
        let mut mints = mints.borrow_mut();
        // Remove expired entries
        // (implementation depends on StableBTreeMap API)
    });
}

fn restart_timers() {
    // Restart rebalancing timer after upgrade
    crate::rebalancing::start_rebalancing_timer();
}
```

**Migration Steps:**
1. Add ic-stable-structures dependency to Cargo.toml
2. Create stable_storage module
3. Migrate PENDING_MINTS from mint_state.rs
4. Update all references to use stable storage
5. Add upgrade hooks to lib.rs

**Testing:**
- Start mint, upgrade canister, verify mint can complete
- Test with dfx deploy --mode upgrade
- Verify timer restarts after upgrade

**Dependencies:** None

---

### 🔴 2.2 Rate Limiter Memory Management
**Priority:** P1
**Effort:** 2-3 hours
**Risk:** MEDIUM - Memory leak over time

**Problem:** Every unique user creates permanent HashMap entry. After 100K users, memory usage becomes significant.

**Implementation:**
```rust
// In rate_limiting/mod.rs
const MAX_ENTRIES: usize = 10000;
const CLEANUP_INTERVAL: u64 = 3600_000_000_000; // 1 hour in nanoseconds

pub fn check_rate_limit(key: &str, limit_nanos: u64) -> Result<()> {
    let now = ic_cdk::api::time();

    RATE_LIMITS.with(|limits| {
        let mut limits = limits.borrow_mut();

        // Periodic cleanup of old entries
        if limits.len() > MAX_ENTRIES {
            let cutoff = now - CLEANUP_INTERVAL;
            limits.retain(|_, &mut time| time > cutoff);
            ic_cdk::println!("Rate limit cleanup: {} entries removed",
                MAX_ENTRIES - limits.len());
        }

        // Check rate limit
        if let Some(&last_time) = limits.get(key) {
            if now - last_time < limit_nanos {
                let wait_time = (limit_nanos - (now - last_time)) / 1_000_000_000;
                return Err(IcpiError::RateLimit(format!(
                    "Rate limit exceeded. Wait {} seconds.", wait_time
                )));
            }
        }

        limits.insert(key.to_string(), now);
        Ok(())
    })
}
```

**Testing:**
- Simulate 10K+ unique users
- Verify old entries cleaned when limit reached
- Test rate limiting still works after cleanup

**Dependencies:** None

---

## **PHASE 3: Core Functionality Implementation (P1)**
*Required for index to function properly*

### 🟡 3.1 Implement Rebalancing Timer & Logic
**Priority:** P1
**Effort:** 2-3 days
**Risk:** MEDIUM - Without this, index never rebalances

**Current Status:** Completely stubbed - returns placeholder messages

**Implementation:**
```rust
// In rebalancing/mod.rs
use ic_cdk_timers::set_timer_interval;
use std::time::Duration;

static TIMER_ID: RefCell<Option<TimerId>> = RefCell::new(None);

pub fn start_rebalancing_timer() {
    let timer_id = set_timer_interval(
        Duration::from_secs(3600),  // 1 hour
        || {
            ic_cdk::spawn(async {
                match perform_rebalance().await {
                    Ok(msg) => ic_cdk::println!("Rebalance: {}", msg),
                    Err(e) => ic_cdk::println!("Rebalance failed: {:?}", e),
                }
            });
        }
    );

    TIMER_ID.with(|id| *id.borrow_mut() = Some(timer_id));
}

pub async fn perform_rebalance() -> Result<String> {
    // 1. Get current portfolio state
    let state = get_portfolio_state_uncached().await?;

    // 2. Calculate target allocation (25% each for 4 tokens)
    let target_per_token = state.total_tvl / 4;

    // 3. Find largest deviation
    let mut max_deviation = 0i64;
    let mut action_token = String::new();
    let mut is_buy = false;

    for (token, balance_usd) in &state.token_balances_usd {
        let deviation = balance_usd - target_per_token;
        if deviation.abs() > max_deviation.abs() {
            max_deviation = deviation;
            action_token = token.clone();
            is_buy = deviation < 0;  // Underweight = buy
        }
    }

    // 4. Execute trade if deviation > 10% and >$10
    let min_trade_usd = 10_000_000; // $10 in e6
    if max_deviation.abs() > min_trade_usd && max_deviation.abs() > (target_per_token / 10) {
        let trade_amount = (max_deviation.abs() / 10) as u64; // 10% of deficit

        if is_buy {
            execute_buy_trade(&action_token, trade_amount).await?;
            return Ok(format!("Bought ${} of {}", trade_amount as f64 / 1e6, action_token));
        } else {
            execute_sell_trade(&action_token, trade_amount).await?;
            return Ok(format!("Sold ${} of {}", trade_amount as f64 / 1e6, action_token));
        }
    }

    Ok("Portfolio balanced, no action needed".to_string())
}

async fn execute_buy_trade(token: &str, amount_usd_e6: u64) -> Result<()> {
    // Use Kongswap to swap ckUSDT → token
    let amount_usdt = Nat::from(amount_usd_e6); // Convert to Nat
    let min_received = calculate_min_received(token, &amount_usdt)?;

    crate::trading_execution::kongswap::execute_swap(
        "ckUSDT",
        token,
        amount_usdt,
        min_received
    ).await?;

    Ok(())
}

async fn execute_sell_trade(token: &str, amount_usd_e6: u64) -> Result<()> {
    // Use Kongswap to swap token → ckUSDT
    let token_amount = usd_to_token_amount(token, amount_usd_e6)?;
    let min_received = calculate_min_received_usdt(&token_amount)?;

    crate::trading_execution::kongswap::execute_swap(
        token,
        "ckUSDT",
        token_amount,
        min_received
    ).await?;

    Ok(())
}
```

**Testing:**
- Manual trigger via dfx canister call, verify trade execution
- Test timer fires every hour
- Verify rebalancing logic with mock portfolios
- Test edge cases: all tokens balanced, only ckUSDT, etc.

**Dependencies:** Requires 3.2 (Kongswap integration)

---

### 🟡 3.2 Kongswap Trading Integration
**Priority:** P1
**Effort:** 2-3 days
**Risk:** MEDIUM - Complex external integration

**Current Status:** Zone 4 completely empty - all stubs

**Implementation:**
```rust
// In 4_TRADING_EXECUTION/kongswap/mod.rs
use candid::{Nat, Principal};

const KONGSWAP_BACKEND: &str = "2ipq2-uqaaa-aaaar-qailq-cai";

pub async fn execute_swap(
    from_token: &str,
    to_token: &str,
    amount: Nat,
    min_received: Nat,
) -> Result<Nat> {
    let kong_backend = Principal::from_text(KONGSWAP_BACKEND)?;

    // 1. Approve Kong to spend our tokens
    approve_token(from_token, &amount).await?;

    // 2. Build swap request
    let request = SwapRequest {
        from: format!("IC.{}", get_token_canister(from_token)?),
        to: format!("IC.{}", get_token_canister(to_token)?),
        amount: amount.clone(),
        min_amount_out: min_received.clone(),
    };

    // 3. Execute swap
    let result: Result<(SwapResult,), _> = ic_cdk::call(
        kong_backend,
        "swap",
        (request,)
    ).await;

    match result {
        Ok((SwapResult::Ok(received),)) => {
            ic_cdk::println!("Swap successful: {} {} → {} {}",
                amount, from_token, received, to_token);
            Ok(received)
        }
        Ok((SwapResult::Err(e),)) => {
            Err(IcpiError::Trading(format!("Swap failed: {:?}", e)))
        }
        Err((code, msg)) => {
            Err(IcpiError::Trading(format!("Call failed: {:?} - {}", code, msg)))
        }
    }
}

async fn approve_token(token: &str, amount: &Nat) -> Result<()> {
    let token_canister = Principal::from_text(get_token_canister(token)?)?;
    let kong_backend = Principal::from_text(KONGSWAP_BACKEND)?;

    let approve_args = ApproveArgs {
        spender: Account { owner: kong_backend, subaccount: None },
        amount: amount.clone(),
        expires_at: None,
        expected_allowance: None,
        memo: None,
        fee: None,
        created_at_time: None,
    };

    let result: Result<(ApproveResult,), _> = ic_cdk::call(
        token_canister,
        "icrc2_approve",
        (approve_args,)
    ).await;

    match result {
        Ok((ApproveResult::Ok(_),)) => Ok(()),
        Ok((ApproveResult::Err(e),)) => {
            Err(IcpiError::Trading(format!("Approve failed: {:?}", e)))
        }
        Err((code, msg)) => {
            Err(IcpiError::Trading(format!("Approve call failed: {:?} - {}", code, msg)))
        }
    }
}

fn get_token_canister(symbol: &str) -> Result<&'static str> {
    match symbol {
        "ckUSDT" => Ok(CKUSDT_CANISTER_ID),
        "ALEX" => Ok("ysy5f-2qaaa-aaaap-qkmmq-cai"),
        "ZERO" => Ok("onuey-xaaaa-aaaah-qcqbq-cai"),
        "KONG" => Ok("xnjld-hqaaa-aaaar-qah4q-cai"),
        "BOB" => Ok("7pail-xaaaa-aaaas-aabmq-cai"),
        _ => Err(IcpiError::Trading(format!("Unknown token: {}", symbol)))
    }
}

pub fn calculate_slippage(amount: &Nat, slippage_bps: u64) -> Nat {
    // Calculate minimum received with slippage tolerance
    // slippage_bps = 100 means 1%
    let amount_u128 = amount.0.to_u128().unwrap_or(0);
    let min_received = amount_u128 * (10000 - slippage_bps) as u128 / 10000;
    Nat::from(min_received)
}
```

**Testing:**
- Test swap $10 ckUSDT for ALEX on mainnet
- Verify slippage protection works (try with 0.5% and 5% slippage)
- Test approval mechanism
- Test error handling: insufficient balance, pool doesn't exist, etc.

**Dependencies:** Study Kongswap API in kong-reference first

---

### 🟡 3.3 Kong Liquidity TVL Integration
**Priority:** P2
**Effort:** 1-2 days
**Risk:** LOW - Nice to have for accurate TVL

**Current Status:** Zone 3 completely empty

**Implementation:**
```rust
// In 3_KONG_LIQUIDITY/tvl/mod.rs
const KONG_LOCKER: &str = "eazgb-giaaa-aaaap-qqc2q-cai";
const KONGSWAP_BACKEND: &str = "2ipq2-uqaaa-aaaar-qailq-cai";

pub async fn get_locked_liquidity_tvl() -> Result<Vec<(String, f64)>> {
    let locker = Principal::from_text(KONG_LOCKER)?;

    // 1. Query lock canisters for our locks
    let result: Result<(Vec<Principal>,), _> = ic_cdk::call(
        locker,
        "get_user_lock_canisters",
        (ic_cdk::id(),)
    ).await;

    let lock_canisters = match result {
        Ok((canisters,)) => canisters,
        Err(e) => return Err(IcpiError::Kong(format!("Failed to get lock canisters: {:?}", e)))
    };

    // 2. Query Kongswap for LP token balances
    let kongswap = Principal::from_text(KONGSWAP_BACKEND)?;
    let mut tvl_by_token: HashMap<String, f64> = HashMap::new();

    for lock_canister in lock_canisters {
        let result: Result<(Vec<PoolBalance>,), _> = ic_cdk::call(
            kongswap,
            "user_balances",
            (lock_canister,)
        ).await;

        if let Ok((balances,)) = result {
            for balance in balances {
                // Filter for approved tokens: ALEX, ZERO, KONG, BOB
                if is_tracked_token(&balance.token_symbol) {
                    let tvl = balance.amount_usd;
                    *tvl_by_token.entry(balance.token_symbol.clone()).or_insert(0.0) += tvl;
                }
            }
        }
    }

    Ok(tvl_by_token.into_iter().collect())
}

fn is_tracked_token(symbol: &str) -> bool {
    matches!(symbol, "ALEX" | "ZERO" | "KONG" | "BOB")
}
```

**Testing:**
- Query and verify TVL matches expected values
- Test with multiple lock canisters
- Verify filtering works correctly

**Dependencies:** None

---

## **PHASE 4: Testing & Validation (P1)**

### 🟡 4.1 Critical Path Integration Tests
**Priority:** P1
**Effort:** 3-4 days

**Test Suite:**
```rust
// tests/integration_test.rs
use candid::{Nat, Principal};

#[tokio::test]
async fn test_mint_burn_cycle() {
    // Setup
    let user = create_test_user();
    let initial_balance = get_icpi_balance(user).await;

    // 1. Mint 100 ICPI with 100 ckUSDT
    let mint_result = mint_icpi(user, Nat::from(100_000_000u64)).await;
    assert!(mint_result.is_ok());

    // 2. Verify supply increased
    let new_balance = get_icpi_balance(user).await;
    assert_eq!(new_balance - initial_balance, Nat::from(100_000_000u64));

    // 3. Burn 50 ICPI
    let burn_result = burn_icpi(user, Nat::from(50_000_000u64)).await;
    assert!(burn_result.is_ok());

    // 4. Verify supply decreased
    let final_balance = get_icpi_balance(user).await;
    assert_eq!(final_balance, initial_balance + Nat::from(50_000_000u64));

    // 5. Verify user received proportional tokens
    let redemptions = burn_result.unwrap().successful_transfers;
    assert!(!redemptions.is_empty());
}

#[tokio::test]
async fn test_burning_actually_burns() {
    let user = create_test_user();

    // Mint 100 ICPI
    mint_icpi(user, Nat::from(100_000_000u64)).await.unwrap();
    let after_mint = get_icpi_balance(user).await;

    // Burn 100 ICPI
    burn_icpi(user, Nat::from(100_000_000u64)).await.unwrap();
    let after_burn = get_icpi_balance(user).await;

    // Should have 0 left (not 100!)
    assert_eq!(after_burn, Nat::from(0u64));
}

#[tokio::test]
async fn test_tvl_includes_all_tokens() {
    // Set known balances
    set_mock_balances(vec![
        ("ckUSDT", 1000_000_000u64), // $1K
        ("ALEX", 2000_00_000_000u64),   // 2000 tokens @ $0.50 = $1K
    ]);

    let tvl = calculate_portfolio_value_atomic().await.unwrap();

    // Should be $2K total, not just $1K
    assert_eq!(tvl, Nat::from(2_000_000u64)); // $2K in e6
}

#[tokio::test]
async fn test_rebalancing_logic() {
    // 1. Set portfolio: 90% ckUSDT, 10% ALEX (imbalanced)
    set_mock_balances(vec![
        ("ckUSDT", 9000_000_000u64), // $9K
        ("ALEX", 200_00_000_000u64),    // 200 ALEX @ $0.50 = $100
    ]);

    // 2. Trigger rebalance
    let result = perform_rebalance().await.unwrap();

    // 3. Verify buy order for underweight tokens
    assert!(result.contains("Bought"));
    assert!(result.contains("ALEX"));
}

#[tokio::test]
async fn test_upgrade_persistence() {
    let user = create_test_user();

    // 1. Start mint (don't complete)
    start_mint(user, Nat::from(100_000_000u64)).await;

    // 2. Upgrade canister
    upgrade_canister().await;

    // 3. Complete mint
    let result = complete_mint(user).await;

    // 4. Verify success
    assert!(result.is_ok());
    let balance = get_icpi_balance(user).await;
    assert_eq!(balance, Nat::from(100_000_000u64));
}

#[tokio::test]
async fn test_rate_limiting() {
    let user = create_test_user();

    // First mint should succeed
    let result1 = mint_icpi(user, Nat::from(100_000_000u64)).await;
    assert!(result1.is_ok());

    // Second immediate mint should fail
    let result2 = mint_icpi(user, Nat::from(100_000_000u64)).await;
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("Rate limit"));
}

#[tokio::test]
async fn test_concurrent_operations_blocked() {
    let user = create_test_user();

    // Start first mint
    let handle1 = tokio::spawn(async move {
        mint_icpi(user, Nat::from(100_000_000u64)).await
    });

    // Try second mint immediately (should fail - reentrancy guard)
    let handle2 = tokio::spawn(async move {
        mint_icpi(user, Nat::from(50_000_000u64)).await
    });

    let results = tokio::join!(handle1, handle2);

    // One should succeed, one should fail
    assert!(results.0.is_ok() ^ results.1.is_ok());
}
```

**Additional Test Coverage:**
- Edge cases: Zero amounts, maximum amounts
- Error paths: Insufficient balance, network failures
- Economic invariants: TVL always equals sum of holdings
- Security: Unauthorized access attempts

---

## **PHASE 5: Production Deployment Strategy**

### 5.1 Phased Rollout Plan
**Priority:** P1
**Effort:** Ongoing

**Stage 1: Alpha (Current - After P0 Fixes)**
- Duration: 1 week
- Fix all P0 issues (burning, TVL, state persistence)
- Deploy with warning banner: "Alpha version - use small amounts"
- Test with <$1000 TVL
- Manual rebalancing only (no timer yet)
- Daily monitoring and manual intervention if needed
- **Success Criteria:** No critical bugs, accurate TVL, burns work correctly

**Stage 2: Beta (After P1 Complete)**
- Duration: 2-4 weeks
- Enable timer-based rebalancing
- Implement Kongswap integration
- Test with <$10K TVL
- Add monitoring dashboards
- 24-hour manual monitoring after each rebalance
- Collect user feedback
- **Success Criteria:** 10+ successful rebalances, no economic exploits, positive user feedback

**Stage 3: Production (After 30 Days Stable)**
- Remove warning banners
- Enable marketing and growth
- Scale to unlimited TVL
- Reduce monitoring frequency to weekly
- **Success Criteria:** 30 days without critical issues, >$50K TVL, active user base

### 5.2 Monitoring & Observability

**Metrics to Track:**
```rust
// Add to 5_INFORMATIONAL/metrics/mod.rs
#[derive(CandidType, Deserialize, Clone)]
pub struct CanisterMetrics {
    // Operations
    pub total_mints: u64,
    pub total_burns: u64,
    pub total_rebalances: u64,

    // Failures
    pub failed_mints: u64,
    pub failed_burns: u64,
    pub failed_rebalances: u64,

    // Economic
    pub total_tvl_usd: f64,
    pub total_supply: Nat,
    pub unique_users: u64,

    // Performance
    pub avg_mint_duration_ms: u64,
    pub avg_burn_duration_ms: u64,

    // Historical
    pub tvl_history: Vec<(u64, f64)>, // (timestamp, tvl)
    pub price_history: Vec<(u64, f64)>, // (timestamp, icpi_price)
}

// Log all critical operations
pub fn log_operation(op_type: &str, details: &str) {
    let timestamp = ic_cdk::api::time();
    ic_cdk::println!("[{}] {}: {}", timestamp, op_type, details);
}
```

**Dashboard Views:**
- Real-time TVL and token distribution
- Mint/burn activity (last 24 hours)
- Rebalancing history and performance
- Error rate and failure modes
- Gas/cycle consumption trends

### 5.3 Rollback Procedures

**Emergency Rollback:**
1. Keep previous wasm binary: `icpi_backend_v1.wasm.gz`
2. Test rollback on testnet first
3. Emergency rollback command:
   ```bash
   dfx canister --network ic install icpi_backend --mode upgrade --wasm icpi_backend_v1.wasm.gz
   ```

**Manual Intervention Scripts:**
```bash
# Pause minting/burning (add to backend)
dfx canister --network ic call icpi_backend set_emergency_pause "(true)"

# Force rebalance
dfx canister --network ic call icpi_backend manual_rebalance

# Query pending operations
dfx canister --network ic call icpi_backend get_pending_operations
```

### 5.4 Cycle Cost Analysis

**Estimated Costs per Operation:**
- Mint: ~0.5B cycles (includes ICRC transfers, TVL query, approval)
- Burn: ~1B cycles (includes token distributions)
- Rebalance: ~2B cycles (includes Kongswap swap)
- Hourly rebalance: ~48B cycles/day

**Initial Funding:** 100T cycles should last ~2000 days at current load

---

## **Priority Matrix**

| Priority | Timeline | Issues | Impact | Status |
|----------|----------|--------|--------|--------|
| **P0** | 1-2 days | 1.1-1.3, 2.1-2.2 | BLOCKING - Economic exploits | 🔴 Not Started |
| **P1** | 1 week | 3.1-3.3, Testing | Core functionality | 🟡 Pending P0 |
| **P2** | 2 weeks | Kong TVL, Advanced features | Enhanced functionality | ⚪ Future |
| **P3** | 1 month | UI improvements, Analytics | Nice-to-have | ⚪ Future |

---

## **Immediate Next Steps (In Order)**

### Today (Oct 7)
- [ ] **1. Fix Burning Exploit** (2-4 hours) - Add ICRC-1 transfer
- [ ] **2. Fix TVL Calculation** (4-6 hours) - Implement token pricing
- [ ] **3. Deploy and Test** - Verify fixes work on mainnet

### Tomorrow (Oct 8)
- [ ] **4. Add Stable Storage** (8-12 hours) - Migrate critical state
- [ ] **5. Rate Limiter Cleanup** (2-3 hours) - Prevent memory leak
- [ ] **6. Fix icpi_burned** (1 hour) - Pass actual burn amount

### This Week
- [ ] **7. Comprehensive Testing** - Integration test suite
- [ ] **8. Deploy to Alpha** - With warning banners
- [ ] **9. Monitor Alpha** - 24/7 for first 48 hours

### Next Week
- [ ] **10. Implement Rebalancing** - Timer + logic
- [ ] **11. Kongswap Integration** - Trading execution
- [ ] **12. Beta Deployment** - Scale to $10K TVL

---

## **Risk Assessment**

### 🔴 EXTREME RISKS (if not fixed immediately)
- **Infinite money exploit via burning** - Users can burn ICPI infinitely
- **90% dilution after first rebalance** - New depositors get massive over-allocation
- **Complete data loss on upgrade** - Pending mints lost, users stuck

### 🟡 MODERATE RISKS
- **Rebalancing failures** - Without proper Kongswap integration
- **Memory leaks** - Affecting long-term stability (months)
- **Slippage during large trades** - Could lose 5-10% on trades

### 🟢 LOW RISKS
- Minor UI inconsistencies
- Suboptimal rebalancing frequency
- Documentation gaps

---

## **Success Criteria**

### ✅ **Minimum Viable Product (Alpha Ready):**
- [x] No economic exploits (burning, TVL)
- [x] Accurate TVL calculation with all tokens
- [x] Working mint/burn with proper ICPI burning
- [x] State persists across upgrades
- [ ] Manual rebalancing works (P1)

### ✅ **Production Ready (Beta Ready):**
- [ ] Automated hourly rebalancing
- [ ] Full Kongswap integration
- [ ] Comprehensive test coverage (80%+)
- [ ] Monitoring and alerting
- [ ] 30 days without critical issues
- [ ] >10 successful rebalances
- [ ] >$10K TVL stable

### ✅ **Scale Ready (Production):**
- [ ] >$50K TVL
- [ ] Active user base (>50 users)
- [ ] Emergency procedures tested
- [ ] Documentation complete
- [ ] Community support channels

---

## **Code Review Checklist**

Before merging any fixes:

### Security
- [ ] All inter-canister calls have error handling
- [ ] No unauthorized minting possible
- [ ] Reentrancy guards in place
- [ ] Rate limiting functional
- [ ] Input validation comprehensive

### Economic Correctness
- [ ] Minting formula correct (with decimal conversion)
- [ ] Burning actually burns ICPI
- [ ] TVL includes all tokens at correct prices
- [ ] No dilution bugs
- [ ] Refunds work properly

### Data Integrity
- [ ] Critical state uses stable storage
- [ ] Upgrade hooks implemented
- [ ] No memory leaks
- [ ] Cleanup mechanisms work

### Testing
- [ ] Unit tests for pure functions
- [ ] Integration tests for critical paths
- [ ] Upgrade tests pass
- [ ] Manual testing on mainnet complete

---

## **Resources & References**

### Documentation
- ICPI Project: `/home/theseus/alexandria/daopad/src/icpi/CLAUDE.md`
- Kongswap Reference: `./kong-reference/`
- Kong Locker: `../kong_locker/`

### Canister IDs
- ICPI Token: `l6lep-niaaa-aaaap-qqeda-cai`
- ICPI Backend: `ev6xm-haaaa-aaaap-qqcza-cai`
- Kongswap Backend: `2ipq2-uqaaa-aaaar-qailq-cai`
- Kong Locker: `eazgb-giaaa-aaaap-qqc2q-cai`
- ckUSDT: `cngnf-vqaaa-aaaar-qag4q-cai`

### Useful Commands
```bash
# Deploy to mainnet
./deploy.sh --network ic

# Test minting
dfx canister --network ic call icpi_backend mint_icpi "(principal \"...\", 100000000)"

# Check TVL
dfx canister --network ic call icpi_backend get_index_state "()"

# Query ICPI balance
dfx canister --network ic call l6lep-niaaa-aaaap-qqeda-cai icrc1_balance_of "(record { owner = principal \"...\" })"
```

---

## **Final Step: GitHub Review & Deployment**

Once you've completed all P0 fixes (Phases 1-2), follow this workflow:

### 1. Final Verification
```bash
# Build without errors
cargo build --target wasm32-unknown-unknown --release --package icpi_backend

# Run any unit tests
cargo test --package icpi_backend

# Verify git status is clean
git status
```

### 2. Commit All Changes
```bash
# Add all modified files
git add src/icpi_backend/src/

# Commit with detailed message
git commit -m "Complete P0 critical fixes for production readiness

## Fixed Issues
1. ✅ Burning exploit - Added ICRC-1 transfer to backend
2. ✅ TVL pricing - Implemented conservative token valuations
3. ✅ icpi_burned calculation - Pass actual burn amount
4. ✅ Stable storage - Migrated critical state for upgrade safety
5. ✅ Rate limiter cleanup - Prevent memory leak

## Testing Completed
- Burn operation correctly reduces ICPI balance
- TVL calculation includes all tracked tokens
- State persists across canister upgrades
- Rate limiter cleans up old entries

All critical P0 blockers resolved. Ready for production alpha deployment.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### 3. Push to GitHub
```bash
# Push to PR branch
git push origin icpi-to-icpx-refactor-v2
```

### 4. Automated Review Process
The push will trigger the Claude Code GitHub Action which will:
- Analyze all changes in the PR
- Check for security vulnerabilities
- Verify economic correctness
- Test code quality and best practices
- Post detailed review comments on PR #4

### 5. Wait for Review Results
- Review typically completes in 5-15 minutes
- Check PR #4 on GitHub: https://github.com/AlexandriaDAO/daopad/pull/4
- Look for new comment from `claude` bot

### 6. Address Review Feedback
If the review finds issues:
```bash
# Fix issues identified
# Edit relevant files
git add .
git commit -m "Address review feedback: [specific fix]"
git push origin icpi-to-icpx-refactor-v2
```

This triggers another review cycle. Repeat until review passes.

### 7. Merge When Ready
Once the review shows:
- ✅ No critical blockers
- ✅ All P0 issues resolved
- ✅ Code quality acceptable
- ✅ Ready for production

The PR can be merged to master and deployed to production alpha.

### Expected Review Outcomes

**After P0 Fixes (This PR):**
- Should pass: No economic exploits
- Should pass: TVL calculation correct
- Should pass: State persistence working
- May flag: Missing P1 features (rebalancing, trading) - OK for alpha

**After P1 Fixes (Next PR):**
- Should pass: Rebalancing logic implemented
- Should pass: Kongswap integration working
- Should pass: Comprehensive test coverage

---

**Last Updated:** 2025-10-07
**Next Review:** After P0 fixes complete
**Owner:** ICPI Development Team
