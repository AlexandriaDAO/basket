# Phase 4: Deployment & Integration Testing

**Agent Type:** Rust Security Engineer
**Mission:** Deploy Phase 4 tests to mainnet and validate all security fixes work in production

---

## 🎯 Context (What's Done)

**Completed and Merged:**
- ✅ **Phase 1** (PR #8): Live prices, minting formula, canister ID consolidation
- ✅ **Phase 2** (PR #9): Admin controls, emergency pause
- ✅ **Phase 3** (PR #12): M-1, M-2, M-3, M-5 medium-severity fixes
- ✅ **M-4** (PR #13): Global operation coordination guards
- ✅ **Code Quality** (PR #14): Enhanced M-1, M-3, M-5 implementations
- ✅ **Phase 4 Tests** (PR #15): Comprehensive test suite ← **JUST MERGED**

**Current State:**
- **Branch:** `main` (all PRs merged, clean)
- **Tests:** 71/80 passing locally (9 require canister environment)
- **Integration script:** `scripts/integration_tests.sh` ready
- **Security rating:** 8.5/10 (up from 6.5/10)

---

## 🚀 Your Mission

Deploy the comprehensive test suite to mainnet and validate all security fixes work correctly. **CRITICAL: If tests fail, fix the CODE to match test expectations, not the other way around.**

### Why Tests Are the Source of Truth

The tests define the **expected behavior** based on security requirements:
- M-2: Fee approval MUST be checked before operations
- M-3: Burns > 10% of supply MUST be rejected
- M-4: Rebalancing MUST NOT run during mints/burns
- M-5: Inconsistent state (supply but no TVL) MUST be an error

**If a test fails:**
1. ❌ **DON'T** change the test to pass
2. ✅ **DO** investigate why the code doesn't meet the spec
3. ✅ **DO** fix the code to satisfy the test requirement
4. ⚠️ **ONLY** change the test if it's factually wrong (rare)

### Example Scenarios

**GOOD:**
```
Test fails: "burn of 10.01% should be rejected"
Investigation: Code allows 11% burns (bug!)
Fix: Update burn validation to properly enforce 10% limit
Result: Test passes, code is now secure
```

**BAD:**
```
Test fails: "burn of 10.01% should be rejected"
"Fix": Change test to expect 11% burns
Result: Test passes, but security vulnerability remains
```

---

## 📋 Task Breakdown

### Step 1: Deploy to Mainnet (15 minutes)

```bash
cd /home/theseus/alexandria/basket

# Build the backend
cargo build --target wasm32-unknown-unknown --release --package icpi_backend

# Deploy to mainnet
./deploy.sh --network ic
# If prompted about breaking changes, review carefully before confirming
```

**Success criteria:**
- Build completes without errors
- Deployment succeeds
- Backend canister upgrades successfully
- No runtime errors in deployment logs

### Step 2: Run Integration Tests (30 minutes)

```bash
# Make script executable (if not already)
chmod +x scripts/integration_tests.sh

# Run the comprehensive test suite
./scripts/integration_tests.sh
```

**What this tests:**
1. System health (supply, TVL queries)
2. Admin controls (pause state, access control)
3. Live price oracle (all 4 basket tokens)
4. Atomic snapshots (M-5 validation)
5. Operation concurrency (M-4 guards)
6. Maximum burn limit (M-3 enforcement)
7. Fee approval validation (M-2 checks)
8. Error handling (clear messages)
9. Cache performance
10. Input validation

**Expected result:** All tests PASS

### Step 3: Handle Test Failures (If Any)

**For each failing test:**

1. **Understand the failure**
   ```bash
   # Read the test output carefully
   # Check what was expected vs actual
   ```

2. **Investigate the code**
   ```bash
   # Find the relevant production code
   # Check if it implements the security requirement correctly
   ```

3. **Fix the CODE (not the test)**
   ```bash
   # Update the implementation to match the security spec
   # Re-deploy: ./deploy.sh --network ic
   # Re-test: ./scripts/integration_tests.sh
   ```

4. **Document the fix**
   - What test failed
   - What was wrong in the code
   - How you fixed it
   - Verification that fix works

### Step 4: Run Unit Tests in Canister

The 9 tests that fail locally require canister environment. After deployment:

```bash
# These tests should now PASS on mainnet:
# - test_cannot_transition_to_idle_via_try_start
# - test_end_rebalancing_clears_state
# - test_has_active_operations
# - test_minting_and_burning_can_coexist
# - test_minting_blocks_rebalancing
# - test_rebalancing_blocks_minting
# - test_min_burn_amount (rate limiting)
# - test_valid_burn_request_structure (rate limiting)
# - test_mint_amount_very_small_deposit
```

**Note:** These tests call `ic_cdk::api::time()` which only works in canisters.

### Step 5: Address Remaining Review Issues (Optional)

From the code review, these MEDIUM/LOW priority items remain:

**MEDIUM Priority:**
1. **Fee approval extraction** (like we did for burn limit)
   - Extract fee approval validation to testable function
   - Add unit tests that call actual validation code
   - Currently tests just simulate the logic

2. **Integration script improvements**
   - Replace `set -e` with better error handling
   - Continue testing after first failure
   - Collect all failures before exiting

**LOW Priority:**
3. Test documentation improvements
4. Edge case coverage for u128::MAX overflow (demonstrate no limit)

**These are NOT blocking** - focus on deployment and test validation first.

---

## 🔧 Workflow

```bash
# 1. Deploy
./deploy.sh --network ic

# 2. Test
./scripts/integration_tests.sh

# 3. If failures, fix CODE
# (Edit production files, not tests)
./deploy.sh --network ic
./scripts/integration_tests.sh

# 4. Repeat until all tests pass

# 5. Create PR documenting results
```

---

## ⚠️ Critical Reminders

### TESTS ARE THE SPECIFICATION

The tests encode security requirements. They are **correct by definition** unless:
- Factually wrong (e.g., "2 + 2 = 5")
- Based on outdated assumptions
- Testing the wrong thing entirely

**99% of the time, failing tests = bugs in code.**

### Common Pitfalls to Avoid

❌ **DON'T:**
- Change test expectations to make them pass
- Comment out failing tests
- Lower assertion strictness
- "Fix" tests without understanding why they fail

✅ **DO:**
- Investigate why code doesn't meet spec
- Fix implementation to satisfy requirements
- Add MORE tests if you find gaps
- Ask if truly unsure if test is wrong

### When a Test IS Wrong

**Legitimate reasons to change a test:**
1. Mathematical error in test logic
2. Incorrect constant (e.g., using wrong fee amount)
3. Test checks wrong function
4. Outdated after requirements change

**In these cases:**
- Document WHY the test is wrong
- Show what the correct behavior should be
- Update test AND verify code is correct
- Add comment explaining the fix

---

## 📊 Success Criteria

**Phase 4 deployment complete when:**
- [ ] Backend deployed to mainnet successfully
- [ ] All 10 integration tests PASS
- [ ] All 80 unit tests PASS (in canister environment)
- [ ] No security regressions
- [ ] Documentation updated with results

**Deliverables:**
1. Deployment confirmation (canister ID, version)
2. Integration test results (all passing)
3. Any fixes made (with explanations)
4. Summary of validation (what works, what was fixed)

---

## 📝 Reporting Template

After completing deployment and testing:

```markdown
## Phase 4 Deployment & Testing Results

### Deployment
- **Date:** [timestamp]
- **Canister:** ev6xm-haaaa-aaaap-qqcza-cai
- **Build:** ✅ Success / ❌ Failed
- **Deploy:** ✅ Success / ❌ Failed

### Integration Tests
- **Total:** 10 tests
- **Passed:** [count]
- **Failed:** [count]

#### Failures (if any)
1. **Test:** [name]
   - **Expected:** [what test expected]
   - **Actual:** [what happened]
   - **Root cause:** [why code didn't match spec]
   - **Fix:** [how you fixed the code]
   - **Verified:** ✅ / ❌

### Unit Tests (Canister Environment)
- **Total:** 80 tests
- **Passed:** [count]
- **Failed:** [count]

### Summary
[Brief summary of deployment validation]

### Security Validation
- M-2 (Fee approval): ✅ / ❌
- M-3 (Burn limit): ✅ / ❌
- M-4 (Operation guards): ✅ / ❌
- M-5 (Atomic snapshots): ✅ / ❌

### Next Steps
[What should happen next]
```

---

## 🎓 Key Principles

1. **Tests define expected behavior** - They're the spec
2. **Code implements behavior** - It should match the spec
3. **Failing test = code bug** - 99% of the time
4. **Fix code, not tests** - Unless test is provably wrong
5. **Security first** - Never weaken tests for convenience

---

## 🚨 Emergency Contacts

If you encounter issues you can't resolve:
- Check `SECURITY_REMEDIATION_PLAN.md` for context
- Review test comments for intent
- Check PR #15 discussions for background
- Document blocker and ask for guidance

---

## Ready to Begin?

1. Read this entire document
2. Understand: **Fix CODE to match TESTS**
3. Deploy to mainnet
4. Run integration tests
5. Fix any failures in the CODE
6. Report results

**START NOW.** 🚀
