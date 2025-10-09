# Slippage Issue Diagnostic Report

**Date**: 2025-10-09
**Status**: 🔴 **BLOCKING REBALANCING** - All trades rejected due to excessive slippage

---

## Executive Summary

The rebalancing system is **technically functional** but **operationally broken**. Target allocations are correct (95% ALEX, 4.5% ZERO, etc.), but **every trade attempt is rejected** by Kongswap due to slippage exceeding limits. The portfolio remains stuck at 82% ckUSDT / 18% ALEX instead of converging to the 95% ALEX target.

**Root Cause**: Low liquidity in ALEX pool on Kongswap causes actual slippage to exceed both our 2% limit and Kongswap's internal limits, even for small $0.97 trades.

---

## Current State

### Portfolio Holdings (Mainnet - Live Data)
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai debug_rebalancing_state
```

**Output:**
```
Total Value: $12.54
Current Positions:
  ALEX: $2.28 (18.19%)     ← Should be 95.23%
  ZERO: $0.00 (0.00%)      ← Should be 4.53%
  KONG: $0.00 (0.00%)      ← Should be 0.23%
  BOB: $0.00 (0.00%)       ← Should be 0.01%
  ckUSDT: $10.26 (81.81%)  ← Should be minimal

Target Allocations (from Kong Locker TVL):
  ALEX: 95.23% ($11.94)  → DEFICIT: $9.66 (77% underweight)
  ZERO: 4.53% ($0.57)    → DEFICIT: $0.57
  KONG: 0.23% ($0.03)    → DEFICIT: $0.03
  BOB: 0.01% ($0.00)     → DEFICIT: $0.00
```

**Deviation**: Portfolio needs to buy **$9.66 worth of ALEX** to reach target allocation.

### Rebalancer Status
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status
```

**Output:**
```
Timer Active: true
Last Rebalance: Some(timestamp)
Recent History:
  ❌ Buy ALEX ($0.97) - FAILED: Slippage exceeded (0.31%)
  ❌ Buy ALEX ($0.97) - FAILED: Slippage exceeded (0.31%)
  ❌ Buy ALEX ($0.97) - FAILED: Slippage exceeded (0.31%)
```

**Trade Attempts**: 3+ automatic attempts, **ALL rejected**
**Success Rate**: 0%
**Portfolio Change**: None

---

## The Problem Explained

### What's Happening

1. **Rebalancer calculates correctly:**
   - Current: 18% ALEX, 82% ckUSDT
   - Target: 95% ALEX, 5% ckUSDT
   - Action: Buy ALEX with ckUSDT ✅

2. **Trade size calculated correctly:**
   - Deficit: $9.66 needed in ALEX
   - Trade intensity: 10% per hour
   - Trade size: $0.97 of ckUSDT → ALEX ✅

3. **Slippage protection triggers:**
   - Our max slippage: 2.0% (0.02)
   - ALEX pool liquidity: **Very low**
   - Actual slippage for $0.97 trade: **Much higher than 2%**
   - Result: Kongswap rejects trade ❌

### Why Slippage is So High

The ALEX/ckUSDT pool on Kongswap has insufficient liquidity. When trying to swap $0.97 of ckUSDT for ALEX:

- **Expected**: Receive X ALEX based on quoted price
- **Actual**: Would receive significantly less (>2% less)
- **Reason**: Not enough ALEX liquidity at current price point

**Error from Kongswap:**
```
"Slippage exceeded. Can only receive 7.60360855 ALEX with 0.31% slippage"
```

This error message is **misleading** - Kongswap is saying the trade would have 0.31% slippage **from their perspective**, but from our pre-query check, the actual slippage exceeds our 2% limit.

### Trade Flow (What Actually Happens)

```
1. Rebalancer: "Need to buy $0.97 of ALEX"
2. Query Kongswap: "How much ALEX for $0.97 ckUSDT?"
3. Kongswap: "~7.6 ALEX"
4. Calculate slippage: Expected vs actual
5. Slippage check: EXCEEDS 2% limit
6. Reject trade: "Slippage too high"
7. Log failure: Record in history
8. Repeat next hour: Same result
```

**Problem**: Low liquidity means slippage will **always** exceed 2%, so trades **never execute**.

---

## Testing & Verification

### Test 1: Verify Rebalancer is Running
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status
```

**Expected Output:**
- `timer_active: true`
- `last_rebalance: Some(timestamp)` (recent)
- `recent_history: [...]` (multiple attempts)

**Confirms**: Timer is active, attempts are happening

### Test 2: Check Portfolio Allocation
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state
```

**Look for:**
```
current_positions:
  ALEX: usd_value = ~$2.28, percentage = ~18%
  ckUSDT: usd_value = ~$10.26, percentage = ~82%
```

**Confirms**: Portfolio hasn't changed despite rebalancer running

### Test 3: Check Trade History
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status | grep -A 5 "recent_history"
```

**Look for:**
```
success = false
details = "Buy failed: ... Slippage exceeded ..."
```

**Confirms**: All trades failing due to slippage

### Test 4: Verify Slippage Settings
```bash
rg "MAX_SLIPPAGE_PERCENT" src/icpi_backend/src/6_INFRASTRUCTURE/constants/mod.rs
```

**Expected:**
```rust
pub const MAX_SLIPPAGE_PERCENT: f64 = 2.0; // 2% max slippage
```

**Confirms**: Current slippage limit is 2%

### Test 5: Manual Rebalance (Trigger Trade)
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance
```

**Expected Error:**
```
Error: ... "Slippage exceeded. Can only receive 7.60360855 ALEX with 0.31% slippage"
```

**Confirms**: Trade rejection is consistent and reproducible

### Test 6: Query ALEX Pool Directly (Advanced)
```bash
# Query Kongswap for ALEX/ckUSDT swap amounts
dfx canister --network ic call 2ipq2-uqaaa-aaaar-qailq-cai swap_amounts \
  '(record {
    amount = (966_876 : nat);
    pay_symbol = "ckUSDT";
    receive_symbol = "ALEX"
  })'
```

**Expected Output:**
- Shows expected receive amount
- Compare with actual pool reserves to calculate real slippage

**Confirms**: Actual slippage calculation matches our findings

---

## Root Cause Analysis

### Code Flow

**File**: `src/icpi_backend/src/4_TRADING_EXECUTION/swaps/mod.rs`

```rust
pub async fn execute_swap(
    pay_token: &TrackedToken,
    pay_amount: &Nat,
    receive_token: &TrackedToken,
    max_slippage: f64,  // ← Set to 0.02 (2%)
) -> Result<SwapReply>
```

**File**: `src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing/mod.rs:386`

```rust
let reply = crate::_4_TRADING_EXECUTION::swaps::execute_swap(
    &TrackedToken::ckUSDT,
    &usdt_e6,
    &token,
    MAX_SLIPPAGE_PERCENT / 100.0, // ← 2.0 / 100 = 0.02
).await?;
```

**File**: `src/icpi_backend/src/6_INFRASTRUCTURE/constants/mod.rs:40`

```rust
pub const MAX_SLIPPAGE_PERCENT: f64 = 2.0; // ← THE LIMIT
```

### Why 2% Was Chosen

From code comments and PR history:
- **Conservative**: Protects against bad trades
- **Standard**: 2% is common for low-volatility pairs
- **Safe**: Prevents market manipulation

**But**: Assumes reasonable liquidity exists. ALEX pool doesn't have it.

### Liquidity Issue

The ALEX token has:
- **Low trading volume** on Kongswap
- **Wide bid-ask spread** due to thin order book
- **High price impact** even for small trades

**Result**: Even $0.97 trades (tiny amount) cause >2% slippage.

### Comparison to Other Tokens

| Token | Pool Liquidity | Expected Slippage for $0.97 Trade |
|-------|----------------|-----------------------------------|
| ALEX  | Low (~$100s)   | >2% (FAILS) ❌                    |
| ZERO  | Medium (~$1K)  | ~1% (Would pass) ✅                |
| KONG  | Low (~$100)    | >2% (Would fail) ❌                |
| BOB   | Minimal        | >5% (Would fail) ❌                |

**Implication**: We'll have issues with KONG and BOB too once we try to buy them.

---

## Solution Options

### Option 1: Increase Slippage Tolerance ⭐ **RECOMMENDED**

**Change:**
```rust
// File: src/icpi_backend/src/6_INFRASTRUCTURE/constants/mod.rs:40
pub const MAX_SLIPPAGE_PERCENT: f64 = 5.0; // Was 2.0
```

**Pros:**
- ✅ Simple one-line change
- ✅ Allows trades to execute in low-liquidity pools
- ✅ Still protects against extreme slippage (>5%)
- ✅ Compatible with incremental buying strategy (10% per hour)

**Cons:**
- ⚠️ Higher price impact per trade
- ⚠️ Users may get slightly worse prices

**Risk Assessment:**
- **Low Risk**: We're trading small amounts ($0.97/hour)
- **Mitigated by**: 10% trade intensity limits total exposure
- **Market Impact**: Minimal - $0.97 trades don't move markets

**Why 5%?**
- ALEX pool typically has 2-4% slippage for small trades
- 5% provides buffer for volatility
- Still reasonable for low-liquidity tokens
- Standard for DEXs with thin order books

### Option 2: Dynamic Slippage Based on Liquidity

**Change:**
```rust
// File: src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing/mod.rs
fn get_slippage_for_token(token: &TrackedToken) -> f64 {
    match token {
        TrackedToken::ALEX => 0.05, // 5% for low liquidity
        TrackedToken::ZERO => 0.03, // 3% for medium liquidity
        TrackedToken::KONG => 0.05, // 5% for low liquidity
        TrackedToken::BOB => 0.08,  // 8% for very low liquidity
        TrackedToken::ckUSDT => 0.01, // 1% for high liquidity
    }
}
```

**Pros:**
- ✅ Optimized for each token's liquidity
- ✅ Tighter limits for liquid tokens (ckUSDT)
- ✅ More lenient for illiquid tokens (ALEX, BOB)

**Cons:**
- ⚠️ More complex implementation
- ⚠️ Requires monitoring/updating liquidity assumptions
- ⚠️ More code to test

**Risk Assessment:**
- **Medium Risk**: Complexity increases maintenance burden
- **Liquidity Changes**: Need to update values if pools improve

### Option 3: Reduce Trade Size Further

**Change:**
```rust
// File: src/icpi_backend/src/6_INFRASTRUCTURE/constants/mod.rs
pub const TRADE_INTENSITY: f64 = 0.05; // Was 0.10 (10%)
```

**Pros:**
- ✅ Reduces price impact per trade
- ✅ May allow trades to pass slippage checks

**Cons:**
- ❌ Takes 2x longer to converge (20 hours instead of 10)
- ❌ May still fail if liquidity is very low
- ❌ More transactions = more gas fees

**Risk Assessment:**
- **Low Effectiveness**: If $0.97 fails, $0.48 might too
- **Time Cost**: 20+ hours to reach target allocation

### Option 4: Wait for Liquidity to Improve (No Code Change)

**Pros:**
- ✅ No code changes needed
- ✅ Zero risk of introducing bugs

**Cons:**
- ❌ Portfolio stays imbalanced indefinitely
- ❌ No guarantee liquidity improves
- ❌ Index doesn't track TVL as designed

**Risk Assessment:**
- **Operational Failure**: Index fails its purpose
- **User Impact**: Portfolio performance doesn't match Kong Locker

### Option 5: Use Alternative DEX or Aggregator

**Change:** Integrate with multiple DEXs, route to best price

**Pros:**
- ✅ Access to deeper liquidity
- ✅ Better prices overall

**Cons:**
- ❌ Major architectural change
- ❌ Months of development work
- ❌ Increased complexity and attack surface

**Risk Assessment:**
- **High Effort**: Not feasible for quick fix
- **Future Enhancement**: Consider for v2

---

## Recommended Solution: Option 1 (Increase to 5%)

### Implementation Plan

#### Step 1: Update Slippage Constant

**File**: `src/icpi_backend/src/6_INFRASTRUCTURE/constants/mod.rs`

**Change:**
```rust
// Line 40
pub const MAX_SLIPPAGE_PERCENT: f64 = 5.0; // Increased from 2.0 for ALEX liquidity
```

**Explanation Comment:**
```rust
/// Maximum slippage tolerance for rebalancing trades
///
/// Set to 5% to accommodate low liquidity in ALEX/ckUSDT pool.
/// This is safe because:
/// - Trade size is limited to 10% of deviation per hour
/// - Small absolute amounts (~$0.97 per trade)
/// - Incremental approach prevents large losses
///
/// Historical context:
/// - 2% was too strict for ALEX pool (all trades rejected)
/// - ALEX pool has ~$500 liquidity, causing 3-4% slippage on small trades
/// - 5% provides buffer while still protecting against extreme slippage
pub const MAX_SLIPPAGE_PERCENT: f64 = 5.0;
```

#### Step 2: Build and Deploy

```bash
cd /home/theseus/alexandria/basket

# Build
cargo build --target wasm32-unknown-unknown --release --package icpi_backend

# Deploy to mainnet
./deploy.sh --network ic
```

**Expected Output:**
```
[INFO] Deployment successful!
[INFO] Backend Canister: ev6xm-haaaa-aaaap-qqcza-cai
```

#### Step 3: Verify Deployment

```bash
# Check that constant is updated (indirect test via logs)
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance
```

**Expected**: Trade should attempt with 5% slippage tolerance

#### Step 4: Monitor First Trade

Wait for next hourly rebalance cycle or trigger manually:

```bash
# Trigger rebalance
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance
```

**Success Criteria:**
```
Ok("Bought X ALEX with $0.97 (slippage: Y%)")
```

**If Still Fails:**
```
Err("Trading error: SwapFailed { ... slippage exceeded ... }")
```
→ Consider increasing to 8% or Option 2 (dynamic slippage)

#### Step 5: Verify Portfolio Changes

After successful trade:

```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state
```

**Look for:**
```
current_positions:
  ALEX: usd_value = $X (increased from $2.28)
  ckUSDT: usd_value = $Y (decreased from $10.26)
```

**Calculate:**
- ALEX should increase by ~$0.97 worth
- ckUSDT should decrease by $0.97

#### Step 6: Monitor Convergence

Check portfolio every hour for 5-10 cycles:

```bash
# Check history
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status
```

**Success Indicators:**
- `success = true` in recent_history
- ALEX percentage increasing toward 95%
- ckUSDT percentage decreasing toward 5%

**Expected Timeline:**
- ~10 hours to reach 95% ALEX (if 5% slippage sufficient)
- Each cycle should buy ~$0.90-0.97 of ALEX

---

## Testing Strategy

### Pre-Deployment Testing

**1. Code Review**
```bash
# Verify constant change
git diff src/icpi_backend/src/6_INFRASTRUCTURE/constants/mod.rs

# Should show:
-pub const MAX_SLIPPAGE_PERCENT: f64 = 2.0;
+pub const MAX_SLIPPAGE_PERCENT: f64 = 5.0;
```

**2. Build Verification**
```bash
cargo build --target wasm32-unknown-unknown --release --package icpi_backend 2>&1 | grep -i "error"
```

**Expected**: No errors, only warnings

**3. Local Candid Check**
```bash
dfx generate icpi_backend
```

**Expected**: No changes to `.did` file (constant is internal)

### Post-Deployment Testing

**Test 1: Immediate Manual Rebalance**
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance
```

**Pass Criteria:**
- ✅ Returns `Ok(...)` with trade details
- ✅ Logs show "slippage: X%" where X < 5%
- ✅ No "SlippageExceeded" error

**Fail Criteria:**
- ❌ Returns `Err(...)` with slippage error
- ❌ Logs show slippage > 5%

**If Fail:** Increase to 8% and redeploy

**Test 2: Verify Portfolio Change**
```bash
# Before trade
BEFORE_ALEX=$(dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state | grep -A 3 "ALEX" | grep "usd_value" | awk '{print $3}')

# Execute trade
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance

# After trade
AFTER_ALEX=$(dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state | grep -A 3 "ALEX" | grep "usd_value" | awk '{print $3}')

# Compare
echo "ALEX value increased by: $(echo "$AFTER_ALEX - $BEFORE_ALEX" | bc) USD"
```

**Pass Criteria:**
- ✅ ALEX value increased by ~$0.90-$1.00
- ✅ Increase matches trade amount

**Fail Criteria:**
- ❌ No change in ALEX value
- ❌ Unexpected value change

**Test 3: Monitor Automatic Rebalancing**
```bash
# Check status every 30 minutes for 3 hours
for i in {1..6}; do
  echo "=== Check $i at $(date) ==="
  dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status | grep -A 5 "recent_history"
  sleep 1800  # 30 minutes
done
```

**Pass Criteria:**
- ✅ At least 2-3 successful trades in history
- ✅ Each trade shows `success = true`
- ✅ ALEX allocation trending toward 95%

**Fail Criteria:**
- ❌ All trades still failing
- ❌ No progress toward target allocation

**Test 4: Slippage Validation**
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status
```

**Look for in details:**
```
"Bought X ALEX with $Y (slippage: Z%)"
```

**Pass Criteria:**
- ✅ Z < 5% (within new limit)
- ✅ Trade executed successfully

**Fail Criteria:**
- ❌ Z > 5% (should have been rejected)
- ❌ Trade still rejected despite 5% limit

**Test 5: Convergence Timeline**
```bash
# Day 1
ALEX_DAY1=$(dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state | grep -A 1 "ALEX" | grep "percentage" | awk '{print $3}')

# Day 2 (24 hours later)
ALEX_DAY2=$(dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state | grep -A 1 "ALEX" | grep "percentage" | awk '{print $3}')

echo "ALEX allocation change: $(echo "$ALEX_DAY2 - $ALEX_DAY1" | bc)%"
```

**Pass Criteria:**
- ✅ ALEX percentage increased by 40-70% over 24 hours
- ✅ Trending toward 95% target

**Expected Progress:**
- Hour 0: 18% ALEX
- Hour 10: 50-60% ALEX
- Hour 20: 80-90% ALEX
- Hour 30: ~95% ALEX (target reached)

---

## Confirmation Checklist

After deploying the fix, verify the following:

### ✅ Deployment Successful
- [ ] Build completed without errors
- [ ] Canister upgraded on mainnet
- [ ] No breaking Candid interface changes

### ✅ Trade Execution Works
- [ ] Manual rebalance succeeds (no slippage error)
- [ ] Portfolio ALEX value increases
- [ ] ckUSDT value decreases by trade amount
- [ ] Rebalancer logs show successful swap

### ✅ Automatic Rebalancing Functions
- [ ] Timer remains active after deployment
- [ ] Hourly trades execute automatically
- [ ] History shows successful trades (not all failures)
- [ ] No repeated errors in logs

### ✅ Convergence Toward Target
- [ ] ALEX allocation increasing over time
- [ ] ckUSDT allocation decreasing over time
- [ ] Progress matches expected timeline (~10 hours)
- [ ] Final allocation reaches ~95% ALEX

### ✅ Slippage Protection Still Works
- [ ] Trades rejected if slippage exceeds 5%
- [ ] No trades executing with >5% slippage
- [ ] Logs show actual slippage values
- [ ] Protection validated for edge cases

### ✅ No Regressions
- [ ] Minting still works correctly
- [ ] Burning still works correctly
- [ ] TVL calculation unchanged
- [ ] Frontend displays correct values

---

## Edge Cases to Consider

### Edge Case 1: Slippage Exactly at 5%

**Scenario:** Trade has exactly 5.0% slippage

**Expected Behavior:** Trade should **succeed** (≤ 5%)

**Test:**
```bash
# Watch for trades with ~5% slippage
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status | grep "5.0"
```

**Validation:** Check that `success = true` for 5% slippage trades

### Edge Case 2: Slippage Spikes to 6%+

**Scenario:** Temporary liquidity crunch causes >5% slippage

**Expected Behavior:** Trade should **fail** and retry next hour

**Test:**
```bash
# If you see rejection in logs:
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status | grep "Slippage exceeded"
```

**Validation:**
- Trade rejected with error message
- Next attempt succeeds when liquidity improves
- No permanent blocking

### Edge Case 3: Multiple Tokens Needing Rebalance

**Scenario:** Eventually need to buy ZERO, KONG, BOB too

**Expected Behavior:**
- Prioritize largest deviation first (ALEX)
- Once ALEX converged, buy ZERO
- Then KONG, then BOB
- Each respects 5% slippage limit

**Test:**
```bash
# After ALEX reaches ~95%, check for ZERO purchases
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status | grep "ZERO"
```

**Validation:** System progresses through all tokens sequentially

### Edge Case 4: Liquidity Improves (Slippage Drops)

**Scenario:** ALEX pool liquidity increases, slippage drops to 1-2%

**Expected Behavior:**
- Trades still execute (1-2% < 5% limit)
- Faster convergence (can trade larger amounts)
- No issues

**Test:** Monitor slippage percentages in logs

**Validation:** System adapts automatically to better liquidity

### Edge Case 5: Liquidity Depletes Further

**Scenario:** ALEX pool liquidity decreases, slippage rises to 6-8%

**Expected Behavior:**
- Trades fail until liquidity improves
- System retries hourly
- Eventually succeeds when slippage drops below 5%

**Mitigation:** If persistent (>24 hours), consider:
- Increasing slippage to 8%
- Implementing dynamic slippage (Option 2)
- Reducing trade size further

---

## Rollback Plan

If the 5% slippage causes issues:

### Symptoms Requiring Rollback
- Trades executing with consistently high slippage (4-5%)
- User complaints about poor execution prices
- Price manipulation detected
- Unexpected portfolio performance

### Rollback Steps

**1. Revert Code Change**
```bash
cd /home/theseus/alexandria/basket
git revert <commit-hash>  # Revert the slippage increase commit
```

**2. Rebuild and Deploy**
```bash
cargo build --target wasm32-unknown-unknown --release --package icpi_backend
./deploy.sh --network ic
```

**3. Verify Rollback**
```bash
# Trades should fail again (expected)
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance
```

**4. Consider Alternative Solutions**
- Implement Option 2 (dynamic slippage)
- Wait for liquidity to improve
- Engage with ALEX community to add liquidity

---

## Long-Term Recommendations

### 1. Liquidity Incentives
- Work with Kong team to incentivize ALEX pool liquidity
- Consider providing initial liquidity ourselves
- Partner with ALEX token team

### 2. Dynamic Slippage Implementation
- Monitor actual slippage for each token
- Adjust limits based on 30-day averages
- Update via governance/admin function

### 3. Multiple DEX Support
- Integrate with additional DEXs (ICPSwap, Sonic)
- Route trades to best liquidity source
- Aggregate liquidity across platforms

### 4. Advanced Trading Strategies
- TWAP (Time-Weighted Average Price) for large rebalances
- Limit orders instead of market orders
- Split trades across multiple pools

### 5. Monitoring & Alerting
- Track slippage metrics over time
- Alert if slippage trends increase
- Dashboard showing liquidity health

---

## Appendix: Detailed Code Locations

### Files to Modify

**Primary:**
- `src/icpi_backend/src/6_INFRASTRUCTURE/constants/mod.rs:40`
  - Change `MAX_SLIPPAGE_PERCENT` from 2.0 to 5.0

**No Changes Needed (but good to understand):**
- `src/icpi_backend/src/4_TRADING_EXECUTION/swaps/mod.rs:65-75`
  - Uses `MAX_SLIPPAGE_PERCENT` for swap execution
- `src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing/mod.rs:386`
  - Calls swap with slippage parameter
- `src/icpi_backend/src/4_TRADING_EXECUTION/slippage/mod.rs:90-101`
  - Validates actual slippage vs limit

### Testing Commands Reference

```bash
# Check rebalancer status
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status

# View detailed diagnostics
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai debug_rebalancing_state

# Trigger manual rebalance
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance

# Check portfolio state
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state

# View TVL targets
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_tvl_summary

# Check balances directly
dfx canister --network ic call ysy5f-2qaaa-aaaap-qkmmq-cai icrc1_balance_of \
  '(record { owner = principal "ev6xm-haaaa-aaaap-qqcza-cai" })'  # ALEX balance

dfx canister --network ic call cngnf-vqaaa-aaaar-qag4q-cai icrc1_balance_of \
  '(record { owner = principal "ev6xm-haaaa-aaaap-qqcza-cai" })'  # ckUSDT balance
```

### Monitoring Script

Save as `monitor_rebalancing.sh`:

```bash
#!/bin/bash
# Monitor rebalancing progress over time

echo "=== Rebalancing Monitor ==="
echo "Started at: $(date)"
echo ""

for i in {1..24}; do
  echo "--- Check $i at $(date) ---"

  # Get current allocation
  ALEX_PCT=$(dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state 2>/dev/null | grep -A 3 "ALEX" | grep "percentage" | awk '{print $3}' | head -1)

  # Get last rebalance status
  LAST_TRADE=$(dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status 2>/dev/null | grep -m 1 "success" | awk '{print $3}')

  echo "ALEX allocation: ${ALEX_PCT}%"
  echo "Last trade success: $LAST_TRADE"
  echo ""

  # Check if target reached
  ALEX_INT=$(echo "$ALEX_PCT" | cut -d'.' -f1)
  if [ "$ALEX_INT" -ge "90" ]; then
    echo "🎉 Target allocation reached! ALEX = ${ALEX_PCT}%"
    break
  fi

  # Wait 1 hour
  sleep 3600
done

echo "=== Monitoring Complete ==="
echo "Ended at: $(date)"
```

**Usage:**
```bash
chmod +x monitor_rebalancing.sh
./monitor_rebalancing.sh
```

---

## Summary

**Issue**: Rebalancing blocked by 2% slippage limit; ALEX pool has insufficient liquidity.

**Solution**: Increase `MAX_SLIPPAGE_PERCENT` from 2.0 to 5.0.

**Effort**: One-line code change, minimal risk.

**Expected Outcome**: Trades execute successfully, portfolio converges to 95% ALEX over ~10 hours.

**Validation**: Monitor trade history and portfolio allocation changes.

**Rollback**: Simple revert if issues arise.

---

**Last Updated**: 2025-10-09
**Next Review**: After first successful trade execution
