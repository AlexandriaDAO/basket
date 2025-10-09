# Rebalancing System Diagnostic Report

**Date**: 2025-10-09
**Status**: ⚠️ **REBALANCING DISABLED - Critical Issues Found**

## Executive Summary

The rebalancing system is **not functional** due to hardcoded target allocations that don't match Kong Locker TVL. Manual rebalancing returns "No rebalancing needed" even though the portfolio is severely imbalanced.

## Current Portfolio State

### Backend Holdings (Actual)
```
Token    Balance         Value (approx)
------   -------------   --------------
ALEX     19.52728260     ~$10 USD (depends on price)
ZERO     0               $0
KONG     0               $0
BOB      0               $0
ckUSDT   10.267453       $10.27
```

### Target Allocations (From Kong Locker TVL)
```
Token    TVL          Target %    Target $ (based on $11,460)
------   ----------   ---------   --------------------------
ALEX     $10,913      95.23%      $10,913
ZERO     $519         4.53%       $519
KONG     $26          0.23%       $26
BOB      $0.89        0.01%       $0.89
```

**Total Kong Locker TVL**: $11,460

### What Backend SHOULD Hold (Target)
```
ALEX: $10,913 worth (~545 ALEX @ $20/ALEX)
ZERO: $519 worth
KONG: $26 worth
BOB: $0.89 worth
ckUSDT: Minimal reserves for trading
```

### Current Deviation
- **ALEX**: Have ~$10, need $10,913 → **DEFICIT: ~$10,903**
- **ZERO**: Have $0, need $519 → **DEFICIT: $519**
- **KONG**: Have $0, need $26 → **DEFICIT: $26**
- **BOB**: Have $0, need $0.89 → **DEFICIT: $0.89**

**Portfolio is severely under-allocated** - should hold ~$11,460 but only has ~$20 in value.

## Root Cause Analysis

### Issue #1: Hardcoded 25% Allocations (CRITICAL)
**Location**: `src/icpi_backend/src/2_CRITICAL_DATA/portfolio_value/mod.rs:230-252`

```rust
// For now, target allocations are equal (25% each for 4 tokens)
let target_allocations = vec![
    TargetAllocation {
        token: TrackedToken::ALEX,
        target_percentage: 25.0,  // ❌ WRONG! Should be 95.23%
        target_usd_value: total_value_f64 * 0.25,
    },
    TargetAllocation {
        token: TrackedToken::ZERO,
        target_percentage: 25.0,  // ❌ WRONG! Should be 4.53%
        target_usd_value: total_value_f64 * 0.25,
    },
    // ... etc
];
```

**Impact**:
- Rebalancer thinks portfolio should be 25%/25%/25%/25%
- Actual target from Kong Locker is 95.23%/4.53%/0.23%/0.01%
- Current holdings (~100% ALEX, 0% others) appear "balanced" under wrong math
- Manual rebalance returns "No rebalancing needed" even though massively imbalanced

### Issue #2: Silent Error Handling
**Location**: `src/icpi_backend/src/5_INFORMATIONAL/display/mod.rs:15-29`

```rust
match crate::_2_CRITICAL_DATA::portfolio_value::get_portfolio_state_uncached().await {
    Ok(state) => state,
    Err(e) => {
        ic_cdk::println!("⚠️ Failed to get portfolio state: {}", e);
        // Return empty state on error rather than panicking
        IndexState {
            total_value: 0.0,
            current_positions: Vec::new(),  // ❌ Returns zeros silently
            // ...
        }
    }
}
```

**Impact**:
- When called via `get_index_state_cached()`, returns zeros
- Hides the actual error from API consumers
- Makes debugging difficult

### Issue #3: Token Pricing May Be Failing
The portfolio calculation calls `get_token_price_in_usdt()` for each token. If any token pricing fails, the entire state calculation fails and returns empty state (due to Issue #2).

**Potential causes**:
- Kongswap integration issues
- Network errors
- Token pool not found
- Price calculation errors

## How Rebalancing SHOULD Work

1. **Calculate Target Allocations** (from Kong Locker TVL)
   ```
   1. Query Kong Locker for all LP locks
   2. Filter for [ALEX, ZERO, KONG, BOB] pools
   3. Calculate each token's % of total locked liquidity
   4. These percentages become target allocations
   ```

2. **Calculate Current Allocations**
   ```
   1. Query backend's token balances
   2. Get real-time prices from Kongswap
   3. Calculate USD value of each position
   4. Calculate percentages
   ```

3. **Calculate Deviations**
   ```
   deviation_pct = target_pct - current_pct
   usd_difference = target_usd_value - current_usd_value
   ```

4. **Execute Rebalancing Trades**
   ```
   IF ckUSDT available (>$10):
       - Buy token with largest DEFICIT
       - Trade size = 10% of deficit
   ELSE IF tokens over-allocated:
       - Sell most overweight token to ckUSDT
       - Trade size = 10% of excess
   ```

## Fix Implementation Plan

### Step 1: Fix Target Allocations (PRIORITY 1)
Replace hardcoded 25% allocations with TVL-based calculations.

**Option A: Use Existing TVL Calculation**
```rust
// In portfolio_value/mod.rs:get_portfolio_state_uncached()

// Get TVL data from Kong Locker
let tvl_data = crate::_3_KONG_LIQUIDITY::tvl::calculate_kong_locker_tvl().await?;

// Calculate total TVL
let total_tvl: f64 = tvl_data.iter().map(|(_, v)| v).sum();

// Build target allocations from TVL percentages
let target_allocations: Vec<TargetAllocation> = tvl_data.iter()
    .map(|(token, tvl_usd)| {
        let target_percentage = if total_tvl > 0.0 {
            (tvl_usd / total_tvl) * 100.0
        } else {
            0.0
        };

        TargetAllocation {
            token: token.clone(),
            target_percentage,
            target_usd_value: total_value_f64 * (target_percentage / 100.0),
        }
    })
    .collect();
```

**Option B: Cache TVL Allocations**
If TVL calculation is expensive, cache the percentages and refresh hourly:
```rust
thread_local! {
    static TARGET_ALLOCATIONS: RefCell<Option<(Vec<TargetAllocation>, u64)>> = RefCell::new(None);
}

async fn get_or_refresh_target_allocations() -> Result<Vec<TargetAllocation>> {
    // Check cache (refresh if > 1 hour old)
    // Otherwise, calculate fresh from Kong Locker TVL
}
```

### Step 2: Fix Error Handling (PRIORITY 2)
Don't silently return empty state - propagate errors up:

```rust
// In display/mod.rs
pub async fn get_index_state_cached() -> Result<IndexState> {
    crate::_2_CRITICAL_DATA::portfolio_value::get_portfolio_state_uncached().await
    // Let errors bubble up instead of returning empty state
}

// Update lib.rs API signature
#[update]
async fn get_index_state_cached() -> Result<IndexState> {
    _5_INFORMATIONAL::display::get_index_state_cached().await
}
```

### Step 3: Debug Token Pricing (PRIORITY 3)
Add detailed logging to understand pricing failures:

```rust
async fn get_token_usd_value(token_symbol: &str, amount: &Nat) -> Result<u64> {
    ic_cdk::println!("🔍 Pricing {} ({} tokens)", token_symbol, amount);

    let price_result = crate::_3_KONG_LIQUIDITY::pools::get_token_price_in_usdt(&token).await;

    match price_result {
        Ok(price) => {
            ic_cdk::println!("✅ {} price: ${}", token_symbol, price);
            // ... continue calculation
        }
        Err(e) => {
            ic_cdk::println!("❌ {} pricing failed: {}", token_symbol, e);
            return Err(e);
        }
    }
}
```

### Step 4: Add Rebalancing Status Diagnostics (PRIORITY 4)
Add an admin function to debug rebalancing logic:

```rust
#[update]
async fn debug_rebalancing_state() -> Result<String> {
    require_admin()?;

    let mut output = String::new();

    // Get TVL targets
    let tvl_data = calculate_kong_locker_tvl().await?;
    output.push_str(&format!("TVL Targets:\n{:?}\n\n", tvl_data));

    // Get current holdings
    let balances = get_all_balances_uncached().await?;
    output.push_str(&format!("Current Balances:\n{:?}\n\n", balances));

    // Get portfolio state
    match get_portfolio_state_uncached().await {
        Ok(state) => output.push_str(&format!("Portfolio State:\n{:?}\n", state)),
        Err(e) => output.push_str(&format!("Portfolio State Error: {}\n", e)),
    }

    Ok(output)
}
```

## Testing Plan

### Test 1: Verify TVL Calculation
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_tvl_summary
```
**Expected**: Returns Kong Locker TVL with correct percentages (ALEX ~95%, etc)

### Test 2: Verify Portfolio State
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state
```
**Expected**: Returns non-zero state with correct target allocations matching TVL

### Test 3: Verify Deviation Calculation
After fixing targets, portfolio state should show:
- ALEX: current ~100%, target ~95% → slight overweight
- ZERO: current 0%, target ~4.5% → massive underweight
- KONG: current 0%, target ~0.2% → underweight
- BOB: current 0%, target ~0.01% → underweight

### Test 4: Manual Rebalance
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance
```
**Expected**: Should identify ZERO as most underweight and execute buy trade

### Test 5: Iterative Rebalancing
Run manual rebalance 10+ times (or wait for hourly timer):
```bash
for i in {1..10}; do
  echo "=== Rebalance iteration $i ==="
  dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance
  sleep 5
done
```
**Expected**: Portfolio gradually converges toward target allocations

## Deployment Strategy

### Phase 1: Read-Only Diagnostics (SAFE)
1. Add `debug_rebalancing_state()` function
2. Deploy and test
3. Verify TVL and pricing work correctly
4. No changes to rebalancing logic yet

### Phase 2: Fix Target Allocations (MEDIUM RISK)
1. Implement TVL-based target calculation
2. Update error handling to propagate errors
3. Deploy to mainnet
4. Verify `get_index_state()` returns correct targets
5. **Do NOT enable automatic rebalancing yet**

### Phase 3: Test Manual Rebalancing (MEDIUM RISK)
1. Run `trigger_manual_rebalance()` with small amounts
2. Verify trades execute correctly
3. Monitor results
4. If successful, run multiple iterations to converge toward target

### Phase 4: Enable Automatic Rebalancing (HIGH RISK)
1. Only after manual rebalancing proven successful
2. Monitor first few automatic cycles closely
3. Keep emergency_pause() ready

## Current Recommendations

### IMMEDIATE ACTIONS (Do Now)
1. ✅ **Do NOT attempt rebalancing** - system is fundamentally broken
2. ✅ **Document findings** - this report
3. Create tracking issue with fix plan

### SHORT TERM (Next Agent)
1. Implement `debug_rebalancing_state()` diagnostic function
2. Fix target allocation calculation to use Kong Locker TVL
3. Fix error handling to propagate instead of returning zeros
4. Add comprehensive logging to pricing functions

### MEDIUM TERM (After Fixes)
1. Test manual rebalancing with small amounts
2. Verify trades execute as expected
3. Iterate until portfolio matches target
4. Document rebalancing behavior for monitoring

### LONG TERM (Production)
1. Monitor automatic hourly rebalancing
2. Track convergence toward target over time
3. Add alerting for rebalancing failures
4. Consider dynamic trade sizing based on liquidity

## Risk Assessment

### Current State: **HIGH RISK**
- Rebalancing produces wrong results (thinks portfolio is balanced when it's not)
- Portfolio severely imbalanced (~100% ALEX instead of diverse allocation)
- Silent errors hide real issues

### After Fixes: **MEDIUM RISK**
- Rebalancing will start executing real trades
- Need to verify Kongswap integration works correctly
- Monitor for gas costs, slippage, failed trades

### Mitigations
1. Use `emergency_pause()` if issues detected
2. Start with manual rebalancing before enabling automatic
3. Monitor first several rebalance cycles closely
4. Keep small amounts in portfolio during testing phase

## Appendix: Relevant Code Locations

### Files to Modify
- `src/icpi_backend/src/2_CRITICAL_DATA/portfolio_value/mod.rs:230-252` - Fix target allocations
- `src/icpi_backend/src/5_INFORMATIONAL/display/mod.rs:15-29` - Fix error handling
- `src/icpi_backend/src/lib.rs:82-94` - Update API signature if needed

### Files to Review
- `src/icpi_backend/src/3_KONG_LIQUIDITY/tvl/mod.rs` - TVL calculation (seems to work)
- `src/icpi_backend/src/3_KONG_LIQUIDITY/pools/mod.rs` - Token pricing (may have issues)
- `src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing/mod.rs` - Rebalancing logic

### Testing Commands
```bash
# Check TVL (works)
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_tvl_summary

# Check portfolio state (currently returns zeros)
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state

# Check backend balances directly
dfx canister --network ic call ysy5f-2qaaa-aaaap-qkmmq-cai icrc1_balance_of '(record { owner = principal "ev6xm-haaaa-aaaap-qqcza-cai" })'  # ALEX
dfx canister --network ic call cngnf-vqaaa-aaaar-qag4q-cai icrc1_balance_of '(record { owner = principal "ev6xm-haaaa-aaaap-qqcza-cai" })'  # ckUSDT

# Try manual rebalance (currently does nothing)
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance

# Check rebalancer status
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status
```

## Conclusion

The rebalancing system **cannot function correctly** until target allocations are fixed to match Kong Locker TVL instead of hardcoded 25% splits. This is a fundamental architectural issue that requires code changes before any rebalancing should be attempted.

**Recommended approach**: Create a new agent session with this diagnostic document to implement the fix plan systematically, starting with Phase 1 diagnostics before attempting any actual rebalancing.
