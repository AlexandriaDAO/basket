# Phase 4: Deployment & Test Validation (PR Automation)

**Agent Type:** Rust Security Engineer + DevOps
**Mission:** Deploy Phase 4 tests to mainnet and validate all security fixes using checkpoint merge workflow

---

## 🎯 Context

**Completed:**
- ✅ Phase 1-3: All security fixes (C-1, H-1, H-2, H-3, M-1, M-2, M-3, M-4, M-5)
- ✅ Phase 4 Tests (PR #15): Comprehensive test suite merged to main
- ✅ Tests: 71/80 passing locally, 9 require canister environment

**Current State:**
- **Branch:** `main` (clean, all PRs merged)
- **Working Directory:** `/home/theseus/alexandria/basket/`
- **Security Rating:** 8.5/10
- **Ready for:** Mainnet deployment and validation

---

## 🚀 Your Mission (Checkpoint Merge Pattern)

Deploy and validate Phase 4 tests using **incremental PRs** (not one big PR).

**CRITICAL PRINCIPLE:** If tests fail, **fix the CODE** to match test expectations, NOT the tests themselves. Tests define security requirements.

---

## 📋 Workflow: Checkpoint PRs

### PR #16: Mainnet Deployment & Initial Validation

**Goal:** Deploy backend with tests, run integration suite, document baseline

**Steps:**

1. **Create worktree**
   ```bash
   git worktree add ../basket-phase4-deploy -b deploy/phase4-mainnet main
   cd ../basket-phase4-deploy
   ```

2. **Build and deploy**
   ```bash
   # Build backend
   cargo build --target wasm32-unknown-unknown --release --package icpi_backend

   # Deploy to mainnet
   ./deploy.sh --network ic
   ```

3. **Run integration tests**
   ```bash
   chmod +x scripts/integration_tests.sh
   ./scripts/integration_tests.sh > integration_test_results.txt 2>&1
   ```

4. **Document results**
   Create `DEPLOYMENT_VALIDATION.md`:
   ```markdown
   # Phase 4 Deployment Validation

   ## Deployment
   - **Date:** [timestamp]
   - **Canister:** ev6xm-haaaa-aaaap-qqcza-cai
   - **Build:** ✅ Success
   - **Deploy:** ✅ Success

   ## Integration Test Results
   - **Total:** 10 tests
   - **Passed:** [X]
   - **Failed:** [Y]

   ### Failed Tests (if any)
   [List each failure with expected vs actual]

   ### Analysis
   [Why tests failed - what in CODE needs fixing]

   ## Next Steps
   [What code fixes are needed]
   ```

5. **Commit and push**
   ```bash
   git add DEPLOYMENT_VALIDATION.md integration_test_results.txt
   git commit -m "Deploy Phase 4 tests to mainnet and run integration suite

   Deployment:
   - Built backend successfully
   - Deployed to ev6xm-haaaa-aaaap-qqcza-cai
   - No deployment errors

   Integration Tests:
   - Ran scripts/integration_tests.sh
   - [X] tests passed
   - [Y] tests failed (see DEPLOYMENT_VALIDATION.md for analysis)

   Next: Fix code issues identified by failing tests (if any)

   🤖 Generated with Claude Code"

   git push -u origin deploy/phase4-mainnet
   ```

6. **Create PR**
   ```bash
   gh pr create --title "Phase 4: Mainnet Deployment & Integration Tests" --body "$(cat <<'EOF'
   ## Summary
   Deploys Phase 4 comprehensive test suite to mainnet and validates security fixes.

   ## Deployment
   - ✅ Backend built successfully
   - ✅ Deployed to mainnet (ev6xm-haaaa-aaaap-qqcza-cai)
   - ✅ Integration test suite executed

   ## Test Results
   - **Total:** 10 integration tests
   - **Passed:** [X]
   - **Failed:** [Y]

   See DEPLOYMENT_VALIDATION.md for detailed analysis.

   ## Issues Found (if any)
   [List code issues that need fixing]

   ## Next Steps
   [What fixes are needed, or "All tests pass - ready for Phase 5"]

   🤖 Generated with [Claude Code](https://claude.com/claude-code)
   EOF
   )"
   ```

7. **Check review (non-blocking)**
   ```bash
   gh pr checks 16
   # If pending: continue to next PR
   # If complete: read review
   ```

8. **Merge when approved**
   ```bash
   gh pr merge 16 --squash --delete-branch
   ```

---

### PR #17: Code Fixes for Test Failures (Only if needed)

**Skip this PR if all tests passed in PR #16**

**Goal:** Fix production code to match test requirements

**Steps:**

1. **Create worktree**
   ```bash
   cd /home/theseus/alexandria/basket
   git pull origin main
   git worktree add ../basket-test-fixes -b fix/integration-test-failures main
   cd ../basket-test-fixes
   ```

2. **Fix CODE (not tests)**

   For each failing test:

   ```bash
   # Read DEPLOYMENT_VALIDATION.md to understand failures
   cat DEPLOYMENT_VALIDATION.md

   # Example: M-3 burn limit test failed
   # Test expects: Burns > 10% rejected
   # Actual: 15% burns allowed (BUG IN CODE)

   # Fix: Update burn_validator.rs to properly enforce 10% limit
   # DO NOT change the test expectations!
   ```

3. **Deploy and re-test**
   ```bash
   cargo build --target wasm32-unknown-unknown --release --package icpi_backend
   ./deploy.sh --network ic
   ./scripts/integration_tests.sh
   ```

4. **Verify fixes**
   - All integration tests now pass
   - No security regressions
   - Tests still define the same requirements

5. **Commit and push**
   ```bash
   git add -A
   git commit -m "Fix [issue]: Update code to match security test requirements

   Issue: [Test name] was failing
   Expected: [What test required]
   Actual: [What code was doing wrong]

   Fix: [How you updated the CODE]
   - Updated [file]
   - Fixed [specific issue]
   - Preserves security requirement

   Validation:
   - Re-deployed to mainnet
   - Integration tests now pass
   - No tests were modified (code fixed to match spec)

   🤖 Generated with Claude Code"

   git push -u origin fix/integration-test-failures
   ```

6. **Create PR**
   ```bash
   gh pr create --title "Fix: [Brief description of code fixes]" --body "$(cat <<'EOF'
   ## Summary
   Fixes production code to satisfy security test requirements.

   ## Failures Addressed
   1. **[Test name]**
      - Expected: [requirement]
      - Issue: [what was wrong in code]
      - Fix: [code changes made]

   ## Changes
   - Updated [file]: [specific fix]
   - [Other changes]

   ## Validation
   - ✅ All integration tests now pass
   - ✅ No tests were modified
   - ✅ Code now meets security specifications

   ## Critical Note
   **Tests were NOT changed** - they define security requirements.
   Production code was updated to meet those requirements.

   🤖 Generated with [Claude Code](https://claude.com/claude-code)
   EOF
   )"
   ```

7. **Merge when approved**

---

### PR #18: Final Validation & Phase 4 Completion

**Goal:** Document comprehensive validation and mark Phase 4 complete

**Steps:**

1. **Create worktree**
   ```bash
   cd /home/theseus/alexandria/basket
   git pull origin main
   git worktree add ../basket-phase4-final -b docs/phase4-complete main
   cd ../basket-phase4-final
   ```

2. **Run comprehensive validation**
   ```bash
   # Integration tests
   ./scripts/integration_tests.sh > final_integration_results.txt 2>&1

   # Unit tests (if you can run in canister)
   # Document that 9 tests require canister environment
   ```

3. **Create comprehensive report**
   Create `PHASE4_COMPLETION_REPORT.md`:
   ```markdown
   # Phase 4: Comprehensive Testing - COMPLETE

   ## Summary
   Phase 4 comprehensive testing is complete. All security fixes validated on mainnet.

   ## Test Suite Deployed
   - **Total Unit Tests:** 80
     - 71 pass locally
     - 9 require canister (ic_cdk::api::time() calls)
   - **Integration Tests:** 10
     - All 10 pass on mainnet

   ## Security Validation

   ### M-2: Fee Approval Checks ✅
   - Burns require ckUSDT fee approval
   - Insufficient approval rejected with clear error
   - Tested on mainnet: PASS

   ### M-3: Maximum Burn Limit ✅
   - Burns > 10% of supply rejected
   - Exactly 10% allowed
   - Uses pure integer arithmetic (no float errors)
   - Tested on mainnet: PASS

   ### M-4: Global Operation Coordination ✅
   - Rebalancing blocked during mints/burns
   - Concurrent user mints allowed
   - 60-second grace period enforced
   - Tested on mainnet: PASS

   ### M-5: Atomic Snapshots ✅
   - Supply and TVL queried in parallel
   - Inconsistent state (supply but no TVL) = hard error
   - Retry logic (3 attempts) working
   - Tested on mainnet: PASS

   ### General Security ✅
   - BigUint arithmetic (no u128 ceiling)
   - Overflow protection
   - Decimal conversion accuracy
   - Tested on mainnet: PASS

   ## Issues Fixed (if any)
   [List any code issues found and fixed during testing]

   ## Security Rating
   - **Before Phase 4:** 8.0/10
   - **After Phase 4:** 8.5/10
   - **Improvement:** Comprehensive test coverage validates all fixes

   ## Next Phase
   **Phase 5: Production Preparation**
   - Monitoring and alerting
   - Documentation (user guides, admin runbook)
   - External security audit preparation
   - Target security rating: 9.0/10
   ```

4. **Update SECURITY_REMEDIATION_PLAN.md**
   - Mark Phase 4 complete
   - Document test coverage achieved
   - Update "REMAINING WORK" section

5. **Commit and push**
   ```bash
   git add -A
   git commit -m "Phase 4: Comprehensive testing complete - all security fixes validated

   Achievements:
   - 80 unit tests (71 pass locally, 9 require canister)
   - 10 integration tests (all pass on mainnet)
   - M-2, M-3, M-4, M-5 validated in production
   - Security rating: 8.5/10

   Validation:
   - All integration tests pass
   - Fee approval checks working
   - Burn limits enforced (10% max)
   - Operation coordination preventing conflicts
   - Atomic snapshots detecting inconsistent state

   Code fixes applied (if any):
   - [List any fixes made]

   Next: Phase 5 (monitoring, docs, external audit)

   🤖 Generated with Claude Code"

   git push -u origin docs/phase4-complete
   ```

6. **Create PR**
   ```bash
   gh pr create --title "Phase 4: Comprehensive Testing Complete" --body "$(cat <<'EOF'
   ## Summary
   Phase 4 comprehensive testing is complete. All security fixes validated on mainnet.

   ## Test Coverage
   - ✅ 80 unit tests (comprehensive)
   - ✅ 10 integration tests (all passing)
   - ✅ M-2, M-3, M-4, M-5 validated

   ## Security Validation
   - ✅ Fee approval checks (M-2)
   - ✅ Burn limit enforcement (M-3)
   - ✅ Operation coordination (M-4)
   - ✅ Atomic snapshots (M-5)
   - ✅ BigUint arithmetic (no artificial limits)

   ## Security Rating
   **8.5/10** - Ready for Phase 5 (production preparation)

   ## Next Steps
   Phase 5: Monitoring, documentation, external audit

   See PHASE4_COMPLETION_REPORT.md for detailed results.

   🤖 Generated with [Claude Code](https://claude.com/claude-code)
   EOF
   )"
   ```

7. **Merge and celebrate** 🎉

---

## ⚠️ CRITICAL: Fix Code, Not Tests

### The Rule

**If a test fails:**
1. ❌ **NEVER** change the test to pass
2. ✅ **ALWAYS** investigate why code doesn't meet spec
3. ✅ **ALWAYS** fix the CODE to satisfy the test
4. ⚠️ **ONLY** change test if it's factually wrong (rare)

### Why This Matters

Tests encode **security requirements**:
- M-2: Fee approval MUST be checked
- M-3: Burns > 10% MUST be rejected
- M-4: Rebalancing MUST NOT run during mints
- M-5: Inconsistent state MUST be an error

**Changing tests = hiding vulnerabilities**

### Example: CORRECT Approach

```
Test fails: "burn of 10.01% should be rejected"
Investigation: Code allows 15% burns (BUG!)
Fix: Update burn_validator.rs to enforce 10% limit
Deploy: ./deploy.sh --network ic
Verify: Test now passes
Result: Security requirement satisfied ✅
```

### Example: WRONG Approach

```
Test fails: "burn of 10.01% should be rejected"
"Fix": Change test to expect 15% burns
Result: Test passes but SECURITY HOLE REMAINS ❌
```

### When to Change a Test

**Only if test is factually wrong:**
- Mathematical error (e.g., "2 + 2 = 5")
- Using wrong constant (e.g., testing 0.2 ckUSDT fee when spec says 0.1)
- Testing wrong function entirely
- Outdated after legitimate requirements change

**In these rare cases:**
- Document WHY test is wrong
- Show correct behavior
- Update test AND verify code is correct
- Add comment explaining the correction

---

## 📊 Success Criteria

### PR #16 Complete When:
- [ ] Backend deployed to mainnet
- [ ] Integration tests executed
- [ ] Results documented
- [ ] Issues identified (if any)

### PR #17 Complete When:
- [ ] All code issues fixed
- [ ] Integration tests pass
- [ ] No tests were modified
- [ ] Fixes documented

### PR #18 Complete When:
- [ ] Final validation complete
- [ ] Comprehensive report created
- [ ] SECURITY_REMEDIATION_PLAN.md updated
- [ ] Phase 4 marked complete

### Overall Success:
- [ ] All 10 integration tests PASS
- [ ] All 80 unit tests PASS (in canister)
- [ ] M-2, M-3, M-4, M-5 validated
- [ ] Security rating: 8.5/10
- [ ] Ready for Phase 5

---

## 🔧 Common Issues & Solutions

### Issue: Integration Test Fails

**DO:**
1. Read test output carefully
2. Understand what test expects
3. Find production code that should implement it
4. Fix the CODE to match the spec
5. Re-deploy and verify

**DON'T:**
- Change test expectations
- Lower assertion strictness
- Comment out failing tests

### Issue: Merge Conflict

```bash
cd /path/to/worktree
git fetch origin
git rebase origin/main
# Fix conflicts in favor of keeping tests as-is
git add <files>
git rebase --continue
git push origin <branch> --force-with-lease
```

### Issue: Can't Figure Out What's Wrong

1. Read test comments (they explain intent)
2. Check PR #15 discussions
3. Review SECURITY_REMEDIATION_PLAN.md
4. Document blocker and ask for guidance

---

## 📝 Ready to Begin?

1. **Understand the mission:** Deploy, test, fix CODE (not tests)
2. **Create PR #16:** Deploy and document results
3. **Create PR #17:** Fix any code issues (if needed)
4. **Create PR #18:** Final validation report
5. **Celebrate:** Phase 4 complete! 🚀

**START NOW.**
