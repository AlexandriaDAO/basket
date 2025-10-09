# Agent Task: Fix Rebalancing Slippage Issue

## Mission

Fix the rebalancing system's slippage issue preventing ALEX token purchases. All trades are being rejected due to low liquidity causing slippage to exceed the 2% limit. Increase slippage tolerance to 5%, deploy to mainnet, verify trades execute successfully, and iterate on PR feedback until approved.

---

## Context Documents

**CRITICAL - Read These First:**

1. **Problem Analysis**: `@SLIPPAGE_ISSUE_DIAGNOSTIC.md`
   - Complete explanation of the slippage issue
   - Testing methodology
   - Solution options and rationale
   - Verification procedures

2. **Workflow Guide**: `@.claude/prompts/autonomous-pr-orchestrator.md`
   - How to create and iterate on PRs
   - Git worktree usage for parallel work
   - Review cycle automation
   - Success criteria

---

## Task Overview

**Problem**: Rebalancing system calculates correctly but all trades fail with "Slippage exceeded" errors. Portfolio stuck at 82% ckUSDT / 18% ALEX instead of target 95% ALEX.

**Root Cause**: ALEX pool has low liquidity (~$500), causing >2% slippage on even small $0.97 trades.

**Solution**: Increase `MAX_SLIPPAGE_PERCENT` from 2.0 to 5.0 (one-line change).

**Expected Outcome**: Trades execute successfully, portfolio converges to 95% ALEX over ~10 hours.

---

## Implementation Instructions

### Phase 1: Setup Isolated Work Environment

Following the PR automation pattern, create a git worktree:

```bash
cd /home/theseus/alexandria/basket
git worktree add ../basket-slippage-fix -b fix/slippage-tolerance-5pct main
cd ../basket-slippage-fix
```

**Why worktrees?** Allows parallel work without file conflicts. Other agents can work on different branches simultaneously.

### Phase 2: Implement the Fix

**File to Modify**: `src/icpi_backend/src/6_INFRASTRUCTURE/constants/mod.rs`

**Location**: Line 40

**Change**:
```rust
// BEFORE:
pub const MAX_SLIPPAGE_PERCENT: f64 = 2.0; // 2% max slippage

// AFTER:
pub const MAX_SLIPPAGE_PERCENT: f64 = 5.0; // 5% max slippage for ALEX liquidity
```

**Add Detailed Comment** (replace the simple comment with this):
```rust
/// Maximum slippage tolerance for rebalancing trades
///
/// Set to 5% to accommodate low liquidity in ALEX/ckUSDT pool on Kongswap.
/// This is safe because:
/// - Trade size is limited to 10% of deviation per hour (TRADE_INTENSITY)
/// - Small absolute amounts (~$0.97 per trade)
/// - Incremental approach prevents large losses even with higher slippage
///
/// Historical context:
/// - 2% was too strict for ALEX pool (all trades rejected)
/// - ALEX pool has ~$500 liquidity, causing 3-4% slippage on small trades
/// - 5% provides buffer while still protecting against extreme slippage
///
/// See: SLIPPAGE_ISSUE_DIAGNOSTIC.md for full analysis
pub const MAX_SLIPPAGE_PERCENT: f64 = 5.0;
```

### Phase 3: Build and Test Locally

**Build**:
```bash
cargo build --target wasm32-unknown-unknown --release --package icpi_backend
```

**Verify**:
- Build should complete without errors
- Only warnings are acceptable

### Phase 4: Deploy to Mainnet

**IMPORTANT**: This project ALWAYS deploys directly to mainnet for testing.

```bash
./deploy.sh --network ic
```

**Expected Output**:
```
[INFO] Deployment successful!
[INFO] Backend Canister: ev6xm-haaaa-aaaap-qqcza-cai
```

### Phase 5: Verify Fix Works

**Test 1: Manual Rebalance**
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance
```

**Success Criteria**:
```
Ok("Bought X ALEX with $0.97 (slippage: Y%)")
```
Where Y < 5%

**Failure Criteria**:
```
Err("... Slippage exceeded ...")
```
If this happens, may need to increase to 8% (discuss with user)

**Test 2: Check Portfolio Changed**
```bash
# Get current state
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state | grep -A 3 "ALEX"
```

**Look for**:
- ALEX `usd_value` should have increased from ~$2.28
- ALEX `percentage` should have increased from ~18%

**Test 3: Check Trade History**
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status
```

**Look for**:
```
recent_history:
  success = true  ← Should be true now!
  details = "Bought X ALEX ..."
```

**Test 4: Run Diagnostic**
```bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai debug_rebalancing_state
```

**Verify**:
- No errors in portfolio state calculation
- Deviations show progress toward 95% ALEX target

### Phase 6: Create PR

**Commit Changes**:
```bash
git add src/icpi_backend/src/6_INFRASTRUCTURE/constants/mod.rs

git commit -m "$(cat <<'EOF'
Fix: Increase slippage tolerance to 5% for ALEX rebalancing

**Problem**: All rebalancing trades rejected due to slippage exceeding 2% limit

**Root Cause**: ALEX/ckUSDT pool on Kongswap has low liquidity (~$500),
causing 3-4% slippage even on small $0.97 trades. Portfolio stuck at
82% ckUSDT / 18% ALEX instead of target 95% ALEX.

**Solution**: Increase MAX_SLIPPAGE_PERCENT from 2.0 to 5.0

## Changes
- `constants/mod.rs:40`: MAX_SLIPPAGE_PERCENT 2.0 → 5.0
- Added detailed comment explaining rationale

## Why 5% is Safe
- Trade size limited to 10% of deviation per hour (~$0.97)
- Incremental approach prevents large losses
- Still protects against extreme slippage (>5%)
- Standard tolerance for low-liquidity DEX pools

## Verification (Mainnet Testing)

**Before Fix:**
- All trades: FAILED with "Slippage exceeded"
- Portfolio: 82% ckUSDT, 18% ALEX (stuck)
- 3+ failed attempts in history

**After Fix:**
- Manual rebalance: ✅ SUCCESS
- Trade executed: Bought X ALEX with $0.97 (Y% slippage)
- Portfolio: ALEX increasing toward 95% target
- History shows `success = true`

**Commands Used:**
\`\`\`bash
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status
\`\`\`

## Expected Timeline
- ~10 hours to converge to 95% ALEX at 10% trade intensity
- Each hourly cycle should now execute successfully
- Portfolio will automatically rebalance to target allocation

## References
- Full analysis: SLIPPAGE_ISSUE_DIAGNOSTIC.md
- Previous fix: PR #16 (rebalancing TVL allocations)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"

git push -u origin fix/slippage-tolerance-5pct
```

**Create PR**:
```bash
gh pr create --title "Fix: Increase Slippage Tolerance to 5% for ALEX Rebalancing" --body "$(cat <<'EOF'
## Summary

Increases slippage tolerance from 2% to 5% to enable ALEX token purchases in low-liquidity pools. All rebalancing trades were being rejected, preventing the portfolio from reaching its 95% ALEX target allocation.

## Problem

**Symptoms:**
- ❌ Every rebalancing trade rejected with "Slippage exceeded" error
- ❌ Portfolio stuck at 82% ckUSDT / 18% ALEX (target: 95% ALEX)
- ❌ 3+ failed trade attempts in hourly cycles
- ❌ No progress toward target allocation

**Root Cause:**
ALEX/ckUSDT pool on Kongswap has low liquidity (~$500 TVL), causing 3-4% slippage even on tiny $0.97 trades. The 2% slippage limit was too strict for this pool's liquidity profile.

## Solution

One-line change:
\`\`\`rust
// File: src/icpi_backend/src/6_INFRASTRUCTURE/constants/mod.rs:40
pub const MAX_SLIPPAGE_PERCENT: f64 = 5.0; // Was 2.0
\`\`\`

## Why This is Safe

1. **Small Trade Sizes**: Limited to ~$0.97 per hour (10% of $9.66 deficit)
2. **Incremental Approach**: 10 cycles to reach target = limited exposure
3. **Still Protected**: 5% is reasonable for low-liquidity pools
4. **Market Standard**: Many DEXs use 3-5% for illiquid pairs

**Risk Assessment**: LOW
- Max loss per trade: ~$0.05 (5% of $0.97)
- Max total exposure: ~$0.50 over 10 trades
- Acceptable for portfolio rebalancing

## Verification (Mainnet)

### Before Fix
\`\`\`bash
$ dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status

recent_history:
  ❌ success = false
  ❌ details = "Buy failed: Slippage exceeded"
  ❌ success = false
  ❌ details = "Buy failed: Slippage exceeded"
  ❌ success = false
  ❌ details = "Buy failed: Slippage exceeded"
\`\`\`

### After Fix
\`\`\`bash
$ dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance

✅ Ok("Bought X ALEX with $0.97 (slippage: Y%)")
\`\`\`

\`\`\`bash
$ dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state

current_positions:
  ALEX: usd_value = $X (increased from $2.28) ✅
  ckUSDT: usd_value = $Y (decreased from $10.26) ✅
\`\`\`

\`\`\`bash
$ dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status

recent_history:
  ✅ success = true
  ✅ details = "Bought X ALEX with $0.97 (slippage: Y%)"
\`\`\`

## Impact

### Fixed ✅
- Rebalancing trades now execute successfully
- Portfolio actively converging to 95% ALEX target
- Automatic hourly rebalancing functional
- Index tracks Kong Locker TVL as designed

### Expected Timeline
- **Hour 0**: 18% ALEX (current)
- **Hour 10**: 50-60% ALEX
- **Hour 20**: 80-90% ALEX
- **Hour 30**: ~95% ALEX ✅ (target reached)

### Monitoring
Automatic hourly trades will gradually buy ALEX:
- Trade size: ~$0.97 per hour (10% of $9.66 deficit)
- Expected slippage: 3-4% (within 5% limit)
- Convergence rate: ~7-8% allocation increase per hour

## Testing Commands

\`\`\`bash
# Trigger manual trade
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai trigger_manual_rebalance

# Check portfolio state
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state

# View trade history
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status

# Full diagnostics
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai debug_rebalancing_state
\`\`\`

## Related

- **Full Analysis**: `SLIPPAGE_ISSUE_DIAGNOSTIC.md`
- **Previous Fix**: PR #16 (rebalancing TVL allocations)
- **Issue Tracker**: Portfolio stuck at 82% ckUSDT

## Rollback Plan

If 5% proves insufficient (trades still fail):
1. Increase to 8%
2. Consider dynamic slippage per token
3. See SLIPPAGE_ISSUE_DIAGNOSTIC.md Section "Solution Options"

If 5% causes issues (prices too poor):
1. Revert commit
2. Redeploy with 2%
3. Wait for ALEX liquidity to improve

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)" --base main
```

### Phase 7: Iterate on Review Feedback

Following the PR automation pattern:

**Wait for GitHub Actions Review** (~4 minutes):
```bash
sleep 240
gh pr checks <PR_NUMBER>
```

**Check for Feedback**:
```bash
gh pr view <PR_NUMBER> --json comments --jq '.comments[-1].body'
```

**If Issues Found:**
1. Read the review feedback carefully
2. Fix identified issues
3. Build, test, deploy to mainnet
4. Commit and push fixes
5. Wait for re-review (~4 minutes)
6. Repeat until approved

**If Approved:**
- Report success to user
- Provide merge recommendation
- Include monitoring instructions

---

## Success Criteria

### ✅ PR Approved
- GitHub Actions review passes with no P0 issues
- All review feedback addressed
- Ready to merge

### ✅ Trades Executing
- Manual rebalance succeeds (no slippage error)
- `success = true` in rebalancer history
- Portfolio ALEX value increasing

### ✅ Portfolio Converging
- ALEX percentage moving toward 95%
- ckUSDT percentage decreasing
- Progress visible in hourly cycles

### ✅ No Regressions
- Minting still works
- Burning still works
- TVL calculations correct
- Other functionality unchanged

---

## Monitoring Instructions for User

After PR is merged and deployed:

**24-Hour Monitoring** (check every 4-6 hours):
```bash
# Check current allocation
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state | \
  grep -A 3 "ALEX" | grep "percentage"

# Check trade history
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_rebalancer_status | \
  grep -B 2 "success = true"
```

**Expected Progress**:
- Day 1: ALEX should reach 50-60%
- Day 2: ALEX should reach 90-95%
- Steady increase of ~7-8% per hour

**Red Flags**:
- No change in ALEX percentage after 6 hours
- Trades still showing `success = false`
- Slippage consistently at 4.9-5.0% (hitting limit)

**If Red Flags Appear**: Consider increasing to 8% or implementing dynamic slippage.

---

## Edge Cases to Handle

### If Trades Still Fail (Slippage >5%)

**Response**:
```
It looks like 5% is still insufficient for current ALEX liquidity.
I recommend increasing to 8% as an interim solution.

Options:
1. Quick fix: Increase to 8% (30 min)
2. Better solution: Implement dynamic slippage per token (2 hours)
3. Wait: ALEX liquidity may improve naturally

Which would you prefer?
```

### If Multiple Review Iterations Needed

**Pattern**: Follow PR automation guide
- Fix issues one at a time
- Deploy and test each fix
- Don't batch fixes without testing
- Maximum 5 iterations before escalating

### If Convergence is Slower Than Expected

**Response**:
```
Portfolio is converging but slower than the ~10 hour estimate.
Current rate: X% per hour (expected: 7-8%)

Possible causes:
1. Slippage near limit (4-5%) reduces trade size
2. ALEX price volatility affecting calculations
3. Liquidity varying throughout the day

Recommendation: Monitor for 24 hours. As long as progress
continues, this is normal variation.
```

---

## Resources

**Documentation:**
- Full analysis: `SLIPPAGE_ISSUE_DIAGNOSTIC.md`
- PR workflow: `.claude/prompts/autonomous-pr-orchestrator.md`
- Codebase guide: `CLAUDE.md`

**Testing Commands** (all in diagnostic doc):
- Portfolio state checks
- Trade execution verification
- History analysis
- Convergence monitoring

**Mainnet Canister IDs:**
- Backend: `ev6xm-haaaa-aaaap-qqcza-cai`
- ICPI Token: `l6lep-niaaa-aaaap-qqeda-cai`
- ALEX Token: `ysy5f-2qaaa-aaaap-qkmmq-cai`
- ckUSDT Token: `cngnf-vqaaa-aaaar-qag4q-cai`

---

## Communication Template

**After Successful Deployment:**

```
✅ Slippage fix deployed and verified!

Results:
- Manual rebalance: SUCCESS
- Trade executed: Bought X ALEX with $0.97 (Y% slippage)
- Portfolio: ALEX increasing from 18% → Z%
- History: Now showing successful trades

Expected Timeline:
- In 10 hours: ~60-70% ALEX
- In 20 hours: ~90% ALEX
- In 30 hours: ~95% ALEX (target reached)

Monitoring:
Portfolio will automatically rebalance hourly. Check progress:
`dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai get_index_state`

PR: https://github.com/AlexandriaDAO/basket/pull/[NUMBER]
Status: [Approved/Pending Review]
```

---

## START NOW

You have everything you need:
1. ✅ Problem analysis (SLIPPAGE_ISSUE_DIAGNOSTIC.md)
2. ✅ Solution (increase slippage to 5%)
3. ✅ Workflow guide (autonomous-pr-orchestrator.md)
4. ✅ Testing methodology
5. ✅ Success criteria

**Begin with Phase 1: Create git worktree**
**Then proceed through all phases systematically**
**Iterate on review feedback until PR is approved**

Good luck! 🚀
