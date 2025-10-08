# Security Remediation - Iteration 2

**Agent Type:** Rust Security Engineer
**Mission:** Complete remaining security fixes from SECURITY_REMEDIATION_PLAN.md

---

## Context (What's Already Done)

**Completed and Merged:**
- ✅ **Phase 1** (PR #8): Live prices, minting formula, canister ID consolidation
- ✅ **Phase 2** (PR #9): Admin controls, emergency pause, logging
- ✅ **Phase 3** (PR #12): M-1, M-2, M-3, M-5 medium-severity fixes

**Current State:**
- **Security Rating:** 7.5/10 (up from 6.5/10)
- **Working Directory:** `/home/theseus/alexandria/basket/`
- **Branch:** `main` (clean, all PRs merged)
- **Last Deployment:** All Phase 1-3 changes live on mainnet

---

## Your Mission

Implement **remaining fixes** from SECURITY_REMEDIATION_PLAN.md:

### Immediate Priorities (This Iteration)

1. **M-4: Concurrent Operation Guards** (1 day)
   - Global operation state tracking
   - Grace period between operations
   - Prevent rebalancing during mints/burns

2. **Phase 4 Code Quality** (from PR #12 review feedback)
   - Replace floating point math in burn limit (M-3 enhancement)
   - Make inconsistent state detection a hard error (M-5 enhancement)
   - Add retry logic to atomic snapshots (M-5 enhancement)

3. **Phase 4 Testing** (unit tests for new code)
   - M-2: Fee approval check tests
   - M-3: Maximum burn limit edge cases
   - M-5: Atomic snapshot tests
   - M-4: Concurrent operation tests

### Lower Priority (Future Iterations)

4. **Phase 4 Integration Tests** (mainnet test script)
5. **Phase 5: Production Preparation** (monitoring, docs, audit)
6. **Low-Severity Issues** (10+ items, non-blocking)

---

## Workflow: Checkpoint Merge Pattern

**This is NOT a single-PR iteration loop. This is a checkpoint-based workflow.**

### Step 1: Read Remaining Work

```bash
# Read the bottom of the plan document
tail -200 /home/theseus/alexandria/basket/SECURITY_REMEDIATION_PLAN.md

# Look for:
# - "REMAINING WORK" section
# - "PR Review Feedback for Phase 4"
# - Specific implementation requirements for M-4
```

### Step 2: Create Worktree for Logical Group

**Group 1: M-4 (Concurrent Guards)**
```bash
git worktree add ../basket-m4-guards -b fix/concurrent-operation-guards main
cd ../basket-m4-guards
```

**Group 2: Code Quality (later)**
```bash
git worktree add ../basket-code-quality -b refactor/phase4-code-quality main
cd ../basket-code-quality
```

**Group 3: Testing (later)**
```bash
git worktree add ../basket-tests -b test/comprehensive-test-suite main
cd ../basket-tests
```

### Step 3: Implement, Build, Test

```bash
# Implement the fix
# (edit Rust files)

# Build
cargo build --target wasm32-unknown-unknown --release --package icpi_backend

# Deploy to mainnet
./deploy.sh --network ic
# NOTE: If prompted for breaking changes and you understand them, use:
# yes | ./deploy.sh --network ic

# Test on mainnet
dfx canister --network ic call ev6xm-haaaa-aaaap-qqcza-cai <test_function>
```

### Step 4: Commit and Push

```bash
git add -A
git commit -m "..."  # Descriptive commit message
git push -u origin <branch-name>
```

### Step 5: Create PR

```bash
gh pr create --title "..." --body "$(cat <<'EOF'
## Summary
...

## Changes
...

## Testing
...

🤖 Generated with Claude Code
EOF
)"
```

### Step 6: Check Review Status (Non-Blocking)

**Don't wait in a loop. Instead:**

```bash
# Check if review is done
gh pr checks <PR_NUM> --repo AlexandriaDAO/basket

# If pending: move to next task
# If complete: read review
gh pr view <PR_NUM> --json comments --jq '.comments[-1].body'
```

### Step 7: Fix Review Issues (If Any)

```bash
# In the same worktree where you implemented
cd /path/to/worktree

# Make fixes based on review
# (edit files)

# Rebuild and test
cargo build --target wasm32-unknown-unknown --release --package icpi_backend
./deploy.sh --network ic

# Test fixes on mainnet
# ...

# Commit and push
git add -A
git commit -m "Fix PR review issues: ..."
git push origin <branch-name>

# Add response comment explaining fixes
gh pr comment <PR_NUM> --body "..."
```

### Step 8: Merge When Approved

```bash
# Check if approved
gh pr view <PR_NUM> --json state,reviewDecision

# If approved, merge
gh pr merge <PR_NUM> --squash --delete-branch

# Clean up worktree
cd /home/theseus/alexandria/basket
git worktree remove ../basket-<name>
git branch -d <branch-name>  # If needed
```

### Step 9: Update Main and Continue

```bash
git pull origin main
./deploy.sh --network ic  # Deploy merged checkpoint

# Update SECURITY_REMEDIATION_PLAN.md with what's complete
# Then start next group (new worktree)
```

---

## Common Issues & Solutions

### Issue: Merge Conflicts on Rebase

**When:** Your PR branch falls behind main

**Solution:**
```bash
cd /path/to/worktree
git fetch origin
git rebase origin/main

# If conflicts:
# 1. Read conflict markers carefully (<<<<<<, =======, >>>>>>>)
# 2. Keep YOUR new code, integrate with upstream structure
# 3. Look for renamed constants/types from other PRs
# 4. Edit files to resolve
# 5. git add <files>
# 6. git rebase --continue
# 7. git push origin <branch> --force-with-lease
```

### Issue: Constant/Type Renamed in Main

**Symptom:** `error: cannot find value MINT_FEE_E6`

**Solution:**
```bash
# Find the new name
rg "MINT_FEE" src/icpi_backend/src/6_INFRASTRUCTURE/constants/

# Update your code to match
# OLD: MINT_FEE_E6
# NEW: MINT_FEE_AMOUNT
```

### Issue: Breaking Candid Change Warning

**Symptom:** Deployment asks "Method X is only in expected type. Proceed? yes/No"

**Solution:**
```bash
# If you removed a method that's not used by frontend: safe
yes | ./deploy.sh --network ic

# If unsure: check frontend code first
rg "debug_rebalancer" src/icpi_frontend/
```

### Issue: Review Still Pending

**Don't:** Wait in a loop checking status

**Do:**
```bash
# Quick check
gh pr checks <PR_NUM>

# If pending: continue other work
# If complete: read review and iterate

# Move to next phase while waiting
```

---

## What Makes This Different from Standard Orchestrator

| Standard Orchestrator | Checkpoint Pattern |
|----------------------|-------------------|
| One PR, iterate 5 times | Multiple PRs, one per phase |
| Wait 4min for reviews in loop | Check reviews non-blockingly |
| Merge when approved | Merge each phase as checkpoint |
| Single worktree | New worktree per phase |
| All-or-nothing | Incremental progress |

---

## TodoList Template for M-4

```
- [ ] Read M-4 requirements from SECURITY_REMEDIATION_PLAN.md
- [ ] Create worktree: ../basket-m4-guards
- [ ] Implement GlobalOperation enum in reentrancy.rs
- [ ] Add try_start_global_operation() and end_global_operation()
- [ ] Apply to rebalancing timer logic
- [ ] Build and test locally
- [ ] Deploy to mainnet
- [ ] Test concurrent operations (pause during mint, etc.)
- [ ] Create PR with comprehensive description
- [ ] Wait for review (check status non-blockingly)
- [ ] Fix review issues if any
- [ ] Merge when approved
- [ ] Clean up worktree
- [ ] Update main and deploy checkpoint
```

---

## Success Criteria for This Round

**M-4 Complete:**
- [ ] Global operation state prevents conflicts
- [ ] Rebalancing skips cycle if mint/burn active
- [ ] Grace period enforced (60 seconds)
- [ ] Multiple users can still mint concurrently
- [ ] Tests pass on mainnet

**Code Quality Complete:**
- [ ] Integer arithmetic replaces f64 in burn limit
- [ ] Inconsistent state is hard error (not warning)
- [ ] Retry logic added to atomic snapshots
- [ ] All tests pass

**Phase 4 Tests Complete:**
- [ ] Unit tests for M-2, M-3, M-4, M-5
- [ ] Integration test script created
- [ ] All tests pass on mainnet
- [ ] Coverage documented

**Ready for Phase 5:**
- [ ] Security rating 8.5/10+
- [ ] All critical/high/medium issues fixed
- [ ] Comprehensive test coverage
- [ ] Production prep can begin

---

## Communication

**Report after each PR merge:**
- What you implemented
- What you tested
- What review feedback you addressed
- What's next

**Example:**
```
✅ M-4 Complete (PR #13 merged)

Implemented:
- GlobalOperation enum with Idle/Minting/Burning/Rebalancing
- 60-second grace period between operation switches
- Rebalancing timer checks global state before executing
- Per-user guards still allow concurrent mints from different users

Tested:
- Rebalancing skips cycle when mint active (mainnet)
- Grace period enforced (tested with rapid operations)
- Multiple users can mint simultaneously

Review Feedback:
- No blocking issues
- Approved and merged

Next: Phase 4 code quality improvements (integer math, retry logic)
```

---

## Ready to Begin?

1. **Read:** Tail of SECURITY_REMEDIATION_PLAN.md ("REMAINING WORK")
2. **Plan:** Create TodoList for M-4
3. **Execute:** Create worktree, implement, test, PR, merge
4. **Repeat:** Move to next logical group

**START NOW.**
