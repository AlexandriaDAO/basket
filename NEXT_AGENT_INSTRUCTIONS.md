# Instructions for Next Agent

## 🔴 CRITICAL: Previous Fix Failed

**PR #7 was merged but the frontend still doesn't work.** User reports:
- Skeleton screens after hard refresh
- No wallet balances showing
- No portfolio allocations
- Same problem as before

## Your ACTUAL Task

**Fix the frontend loading issue** by addressing the actor initialization race condition.

**Primary Document**: Read `FRONTEND_LOADING_FAILURE_ANALYSIS.md` first - it contains:
- Complete diagnosis of the actor initialization race condition
- 4 fix approaches with pros/cons
- Proper verification steps (test in BROWSER, not with dfx)
- Success criteria: Wallet balances visible in <5 seconds

**Working Directory**: `/home/theseus/alexandria/basket`
**Network**: Always mainnet (`--network ic`)
**Branch**: Create new branch: `fix/frontend-actor-race-condition`

---

## Quick Start

1. **Read FRONTEND_LOADING_FAILURE_ANALYSIS.md** (entire document)
2. **Reproduce the issue** in browser at https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io
3. **Choose a fix approach** (Option 1 recommended)
4. **Implement the fix**
5. **Verify in browser** (not with dfx commands)
6. **Success criteria**: Wallet balances visible in <5 seconds

## Context

### What's Been Done
1. ✅ Fixed backend type mismatches (didn't help frontend)
2. ✅ Removed hardcoded canister IDs (didn't help frontend)
3. ✅ Added error logging (didn't help frontend)
4. ❌ **Merged PR #7** thinking it was fixed
5. 🔴 **Frontend still broken** - actor initialization race condition

### Current State
- ❌ Skeleton screens persist after hard refresh
- ❌ Wallet balances don't show (says "Your wallet is empty")
- ❌ Portfolio allocations don't show
- ❌ Same exact problem as the beginning
- ✅ Backend API methods work (verified with dfx)
- 🔴 **Frontend doesn't call them properly** (race condition)

---

## Implementation Workflow

### Step 1: Create New Branch
```bash
cd /home/theseus/alexandria/basket
git checkout main
git pull origin main
git checkout -b fix/frontend-actor-race-condition
```

### Step 2: Read Diagnosis Document
```bash
# Open and read the entire document
cat FRONTEND_LOADING_FAILURE_ANALYSIS.md

# Key sections:
# - Actual Root Cause Analysis (explains the race condition)
# - How to Actually Fix This (4 options, Option 1 recommended)
# - Proper Verification Steps (MUST verify in browser)
# - Success Criteria (wallet balances in <5 seconds)
```

### Step 3: Reproduce the Issue
1. Open browser: https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io
2. Open dev tools (F12)
3. Hard refresh (Ctrl+Shift+R)
4. Observe:
   - ❌ Skeleton screens persist
   - ❌ "Your wallet is empty" even with tokens
   - ❌ No portfolio allocations
   - Check console for errors
   - Check Network tab for API call timing

### Step 4: Implement Fix
**Recommended**: Option 1 - Manual Actor with Better Initialization

Key changes needed in `src/icpi_frontend/src/App.tsx`:
1. Add `isInitialized` state
2. Initialize actor atomically in single useEffect
3. Don't render hooks until `isInitialized=true`
4. Separate component for authenticated content

See FRONTEND_LOADING_FAILURE_ANALYSIS.md for full code examples.

### Step 5: Build Frontend
```bash
cd src/icpi_frontend
npm run prebuild
npm run build
cd ../..
```

### Step 6: Deploy to Mainnet
```bash
# Deploy ONLY frontend (backend is fine)
dfx deploy --network ic icpi_frontend
```

### Step 7: Verify in Browser (CRITICAL)
**Don't skip this - most important step!**

1. Open: https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io
2. Hard refresh (Ctrl+Shift+R) to clear cache
3. Open dev tools (F12)
4. Login if needed
5. Check success criteria:

   ✅ **Must achieve ALL of these:**
   - Wallet balances visible in <5 seconds (not 20s)
   - Portfolio allocations visible in <5 seconds
   - No "Your wallet is empty" when wallet has tokens
   - No skeleton screens lasting >2 seconds
   - Console shows no "Actor not initialized" errors
   - Network tab shows get_index_state_cached completes in <3s

6. Test both scenarios:
   - Fresh page load (hard refresh)
   - Returning user (already authenticated)

**If any criteria fail, the fix didn't work. Go back to Step 4.**

### Step 8: Commit and Push

```bash
# Stage all changes
git add -A

# Commit with clear message
git commit -m "Fix frontend actor initialization race condition

Fixed skeleton screens and missing data by resolving actor initialization
race condition. Hooks were firing with actor=null before actor was created,
causing all queries to wait and fire simultaneously with 5-20s delays.

## Solution
[Describe which option you implemented from FRONTEND_LOADING_FAILURE_ANALYSIS.md]

## Testing - Verified in Browser
✅ Built successfully
✅ Deployed to mainnet: qhlmp-5aaaa-aaaam-qd4jq-cai
✅ Tested at https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io
✅ Wallet balances visible in <5 seconds (was 20s)
✅ Portfolio allocations visible in <5 seconds
✅ No skeleton screens >2 seconds
✅ Console: No 'Actor not initialized' errors
✅ Network: get_index_state_cached completes in <3s

## Changes Made
- src/icpi_frontend/src/App.tsx: [describe changes]
- [other files modified]

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
"

# Push to branch
git push origin fix/frontend-actor-race-condition
```

### Step 9: Create Pull Request
```bash
gh pr create --title "Fix: Frontend actor initialization race condition" --body "$(cat <<'EOF'
## Problem
Frontend showed persistent skeleton screens and no data after hard refresh.
Backend API methods worked via dfx but frontend never loaded the data properly.

Root cause: Actor initialization race condition - hooks fired with actor=null
before actor was created, causing 5-20 second delays.

## Solution
[Describe which approach from FRONTEND_LOADING_FAILURE_ANALYSIS.md you used]

## Testing
✅ Wallet balances visible in <5 seconds (was 20s+)
✅ Portfolio allocations visible in <5 seconds
✅ No "Your wallet is empty" when wallet has tokens
✅ No skeleton screens lasting >2 seconds
✅ Console: No errors
✅ Tested: Hard refresh + returning user scenarios

## Before/After
**Before**: 20 second delays, skeleton screens, no data
**After**: <5 second load, all data visible, no skeletons

Closes issue with frontend loading.
EOF
)"
```

---

## Important Notes

### Always Verify in Browser, Not with dfx
**❌ Wrong approach:**
```bash
# Backend testing proves nothing about frontend
dfx canister call --network ic ev6xm-haaaa-aaaap-qqcza-cai get_index_state_cached
```

**✅ Correct approach:**
1. Open https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io in browser
2. Open dev tools (F12)
3. Check Network tab, Console, and UI
4. Verify all success criteria met

### Canister IDs (Mainnet)
- icpi_frontend: `qhlmp-5aaaa-aaaam-qd4jq-cai`
- icpi_backend: `ev6xm-haaaa-aaaap-qqcza-cai`
- ICPI token: `l6lep-niaaa-aaaap-qqeda-cai`

### Key Files
- **FRONTEND_LOADING_FAILURE_ANALYSIS.md** ⭐ READ THIS FIRST
- `FRONTEND_REFACTOR_BLOCKED.md` - Why ic-use-actor failed
- `src/icpi_frontend/src/App.tsx` - Actor initialization (needs fixing)
- `src/icpi_frontend/src/hooks/useICPI.ts` - Query hooks
- `src/icpi_backend/src/lib.rs` - Backend API (working, don't change)

---

## If You Get Stuck

### Fix Doesn't Work (Still Seeing Skeleton Screens)
1. Check browser console for errors
2. Check Network tab - are API calls completing?
3. Add debug logging to App.tsx:
   ```typescript
   useEffect(() => {
     console.log('🔍 Debug State:', {
       actor: !!actor,
       agent: !!agent,
       principal,
       isInitialized,
     });
   }, [actor, agent, principal, isInitialized]);
   ```
4. Try a different approach from FRONTEND_LOADING_FAILURE_ANALYSIS.md

### Build Fails
```bash
# Re-install dependencies
cd src/icpi_frontend
npm install --legacy-peer-deps
npm run prebuild
npm run build
```

### Need to Revert Changes
```bash
git checkout src/icpi_frontend/src/App.tsx
```

---

## Success Criteria

✅ **Wallet balances visible in <5 seconds** (not 20s)
✅ **Portfolio allocations visible in <5 seconds**
✅ **No "Your wallet is empty"** when wallet has tokens
✅ **No skeleton screens lasting >2 seconds**
✅ **Console: No "Actor not initialized" errors**
✅ **Network: get_index_state_cached completes in <3s**

Test both:
- Hard refresh (Ctrl+Shift+R)
- Returning user (already authenticated)

---

**START HERE**:
1. Read FRONTEND_LOADING_FAILURE_ANALYSIS.md (entire document)
2. Reproduce issue in browser
3. Implement Option 1 (recommended)
4. Verify in browser (not with dfx)
5. Only commit if ALL success criteria met
