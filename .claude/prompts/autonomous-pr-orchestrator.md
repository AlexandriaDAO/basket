# 🤖 Autonomous PR Orchestrator Prompt

**Purpose**: Autonomous agent that implements features, creates PRs, and iterates on review feedback until approval using git worktrees for true parallel work.

**Use Case**: Give this prompt to a fresh Claude Code agent to handle the entire feature → implementation → review → fix → approval cycle autonomously. Each agent works in an isolated git worktree so multiple agents can run in parallel without conflicts.

---

## ⚡ Quick Start (One-Line Commands)

### 1. Iterate on Existing PR
```
Iterate on https://github.com/AlexandriaDAO/daopad/pull/4 until approved.
```

### 2. Implement New Feature
```
Implement user authentication system on a new branch and create PR. Iterate until approved.
```

### 3. Fix Known Issue
```
Fix the burning exploit where users can burn ICPI without transferring tokens. Create PR and iterate until approved.
```

**That's it!** The agent handles everything:
- ✅ Git worktree creation (for parallel safety)
- ✅ Branch creation and management
- ✅ Code implementation
- ✅ Build and testing
- ✅ Push and PR creation
- ✅ Review iteration (3-5 cycles)
- ✅ Success/escalation reporting

---

## 🔧 Advanced: Parallel Work Setup

**Want to work on 3 things simultaneously?**

```bash
# Terminal 1: Fix PR #4
cd /home/theseus/alexandria/daopad/src/icpi
git worktree add ../icpi-pr-4 icpi-to-icpx-refactor-v2
cd ../icpi-pr-4
claude → "Iterate on https://github.com/AlexandriaDAO/daopad/pull/4"

# Terminal 2: New feature (runs in parallel!)
cd /home/theseus/alexandria/daopad/src/icpi
git worktree add -b feature/rebalancing ../icpi-rebalance master
cd ../icpi-rebalance
claude → "Implement automated hourly rebalancing and create PR"

# Terminal 3: Another feature (also parallel!)
cd /home/theseus/alexandria/daopad/src/icpi
git worktree add -b feature/ui-charts ../icpi-charts master
cd ../icpi-charts
claude → "Add portfolio value chart component and create PR"
```

**Result**: 3 agents working simultaneously on 3 different PRs, each iterating on their own review feedback! 🚀

---

## 📋 Generic Orchestrator Prompt Template (Advanced)

```
You are an autonomous PR orchestrator. Implement changes and iterate on review feedback until approved.

INPUT: [REPLACE WITH ONE OF THE FOLLOWING]
- Feature request: "Add user authentication system"
- PR URL: "https://github.com/AlexandriaDAO/daopad/pull/4"
- Issue: "Fix burning exploit in ICPI backend"

Main Repo: [REPLACE - e.g., /home/theseus/alexandria/daopad/src/icpi]
Base Branch: [REPLACE - e.g., master]

WORKFLOW:

## Phase 1: Worktree Setup (For Parallel Safety)

IF input is a feature request:
  1. Generate branch name: feature/[descriptive-slug]
  2. Create isolated worktree:
     cd [Main Repo]
     git worktree add ../[repo-name]-[slug] -b feature/[slug] [Base Branch]
  3. Work in new directory: cd ../[repo-name]-[slug]
  4. Implement the feature
  5. Build and test
  6. Push: git push -u origin feature/[slug]
  7. Create PR: gh pr create --title "[Feature]" --body "[Description]"
  8. Proceed to Phase 2

IF input is a PR URL:
  1. Extract PR number and branch name: gh pr view [PR_NUM] --json headRefName
  2. Create worktree for existing branch:
     cd [Main Repo]
     git worktree add ../[repo-name]-pr-[NUM] [branch-name]
  3. Work in new directory: cd ../[repo-name]-pr-[NUM]
  4. Pull latest: git pull origin [branch-name]
  5. Proceed to Phase 2

IF input is an issue description:
  1. Generate branch name: fix/[issue-slug]
  2. Create worktree:
     cd [Main Repo]
     git worktree add ../[repo-name]-[slug] -b fix/[slug] [Base Branch]
  3. Work in new directory: cd ../[repo-name]-[slug]
  4. Implement the fix
  5. Build and test
  6. Push and create PR
  7. Proceed to Phase 2

IMPORTANT: All subsequent work happens in the worktree directory, not the main repo.
This ensures parallel agents never conflict.

WHY WORKTREES?
- Enables multiple agents to work on different PRs/features simultaneously
- Each agent gets isolated directory → no file conflicts
- Shared .git → all commits/branches synchronized
- Example: Agent 1 fixes PR #4 in ../icpi-pr-4/ while Agent 2 builds feature in ../icpi-feature/

## Phase 2: Iteration Loop (Max 5 iterations)

FOR iteration in 1..5:
  1. Fetch latest PR review:
     gh pr view [PR_NUMBER] --json comments --jq '.comments[-1].body'

  2. Analyze review for P0 (blocking) issues:
     - Count critical issues
     - Extract specific problems

  3. IF P0 issues found:
     a. Report status:
        "📊 Iteration [N]/5: Found [X] P0 issues"

     b. Spawn pr-review-resolver subagent:
        Task: "Analyze [PR_URL] and fix all P0 issues.
               Working directory: [WORKING_DIR]
               Branch: [BRANCH_NAME]
               Implement fixes, build, test, and push."

     c. Wait for subagent completion

     d. Wait 4 minutes for GitHub Actions review:
        sleep 240

     e. Continue to next iteration

  4. IF P0 issues = 0:
     Report: "✅ SUCCESS: PR approved after [N] iterations"
     Ask: "Ready to merge? (yes/no/wait)"
     EXIT

  5. IF iteration = 5 AND P0 issues > 0:
     Report: "⚠️ ESCALATE: 5 iterations complete, [X] P0 issues remain"
     List remaining issues
     Ask: "Continue manually or abandon?"
     EXIT

## Phase 3: Completion

When approved:
- Summary of all iterations
- Total time elapsed
- Issues resolved count
- Merge recommendation

SETTINGS:
- Max iterations: 5
- Review wait time: 240 seconds (4 minutes)
- Auto-build: Yes, after each fix
- Auto-test: Yes, before each push
- Auto-deploy: No (require explicit confirmation)

OUTPUT FORMAT (after each iteration):

📊 Iteration [N]/5 - [Branch: branch-name]

Review Analysis:
- P0 issues: [X]
- P1 issues: [Y]
- P2 issues: [Z]

Fixes Applied:
- ✅ Fix 1: [Description]
- ✅ Fix 2: [Description]

Build Status: [✅ Success | ❌ Failed]
Push Status: [✅ Pushed commit abc1234 | ❌ Failed]

Next Action: [Waiting for review | Iteration N+1 | Complete | Escalate]

START NOW.
```

---

## 🎯 Quick Start Examples

### Example 1: Fix Existing PR #4
```
You are an autonomous PR orchestrator.

INPUT: https://github.com/AlexandriaDAO/daopad/pull/4
Working Directory: /home/theseus/alexandria/daopad/src/icpi
Base Branch: master

START NOW.
```

### Example 2: New Feature from Scratch
```
You are an autonomous PR orchestrator.

INPUT: Add comprehensive integration tests for ICPI minting
Working Directory: /home/theseus/alexandria/daopad/src/icpi
Base Branch: master

START NOW.
```

### Example 3: Fix Known Issue
```
You are an autonomous PR orchestrator.

INPUT: Fix the burning exploit where users can burn ICPI without transferring tokens
Working Directory: /home/theseus/alexandria/daopad/src/icpi
Base Branch: icpi-to-icpx-refactor-v2

START NOW.
```

---

## 🔀 Parallel Work with Git Worktrees

### Why Worktrees?

**Problem with traditional git**:
- `git checkout` changes files for ALL terminals in the same directory
- Terminal 1 does `git checkout feature-a` → Terminal 2's files change too
- **Conflict**: Both terminals can't work on different branches simultaneously

**Solution - Git Worktrees**:
- Each branch gets its own directory
- Terminals work in different directories → No conflicts
- All share same .git database → Commits, branches, remotes unified

### Worktree Structure Example

```
/home/user/alexandria/daopad/src/icpi/          ← Main (master branch)
  .git/                                          ← Shared git database
  src/icpi_backend/

/home/user/alexandria/daopad/src/icpi-pr-4/     ← Worktree (PR #4 branch)
  .git → ../icpi/.git                            ← Linked to shared database
  src/icpi_backend/                              ← Independent files

/home/user/alexandria/daopad/src/icpi-auth/     ← Worktree (feature/auth)
  .git → ../icpi/.git                            ← Linked to shared database
  src/icpi_backend/                              ← Independent files
```

**Result**:
- Terminal 1 works in `icpi-pr-4/` → Never touches Terminal 2's files
- Terminal 2 works in `icpi-auth/` → Never touches Terminal 1's files
- Both can build, test, push independently
- Both share commits/branches via shared .git

### Safe Parallel Pattern ✅

**Terminal 1** (Fix existing PR #4):
```bash
cd /home/theseus/alexandria/daopad/src/icpi
git worktree add ../icpi-pr-4 icpi-to-icpx-refactor-v2
cd ../icpi-pr-4

claude
> Iterate on https://github.com/AlexandriaDAO/daopad/pull/4 until approved.
```

**Terminal 2** (New feature in parallel):
```bash
cd /home/theseus/alexandria/daopad/src/icpi
git worktree add -b feature/ui-dashboard ../icpi-dashboard master
cd ../icpi-dashboard

claude
> Implement user dashboard UI with portfolio charts and create PR.
```

**Both agents run simultaneously with ZERO conflicts!**

### Real-World Parallel Example

**Scenario**: You want to:
1. Fix PR #4 (burning/minting issues)
2. Add new rebalancing feature
3. Fix security audit findings

All at the same time, all verified by GitHub Actions!

**Setup**:
```bash
# Main repo stays on master
cd /home/theseus/alexandria/daopad/src/icpi
git checkout master

# Terminal 1: PR #4
git worktree add ../icpi-pr-4 icpi-to-icpx-refactor-v2
cd ../icpi-pr-4
claude → "Iterate on https://github.com/AlexandriaDAO/daopad/pull/4"

# Terminal 2: Rebalancing feature
cd /home/theseus/alexandria/daopad/src/icpi
git worktree add -b feature/rebalancing ../icpi-rebalance master
cd ../icpi-rebalance
claude → "Implement automated rebalancing and create PR"

# Terminal 3: Security fixes
cd /home/theseus/alexandria/daopad/src/icpi
git worktree add -b fix/security-audit ../icpi-security master
cd ../icpi-security
claude → "Fix all findings from security audit in issue #42"
```

**Result**:
```
Main repo:    master        (untouched)
icpi-pr-4/:   PR #4 branch  (iterating to approval)
icpi-rebalance/: feature/rebalancing (implementing + creating PR)
icpi-security/: fix/security (fixing + creating PR)
```

All three agents:
- Work independently ✅
- Push to different branches ✅
- Create separate PRs ✅
- Get separate GitHub Action reviews ✅
- Iterate independently until approved ✅

### Cleanup After Merge

```bash
# When PR #4 is merged and branch deleted
cd /home/theseus/alexandria/daopad/src/icpi
git worktree remove ../icpi-pr-4
git branch -d icpi-to-icpx-refactor-v2

# Continue using main repo as normal
```

### Worktree Commands Reference

```bash
# List all worktrees
git worktree list

# Create worktree for existing branch
git worktree add ../path branch-name

# Create worktree with NEW branch from base
git worktree add -b new-branch-name ../path base-branch

# Remove worktree (after branch merged)
git worktree remove ../path

# Prune deleted worktrees
git worktree prune
```

---

## 🛠️ Customization Variables

Replace these in the template:

| Variable | Example | Purpose |
|----------|---------|---------|
| `[INPUT]` | PR URL or feature description | What to work on |
| `Working Directory` | `/home/user/project` | Where the code lives |
| `Base Branch` | `main` or `master` | Branch to create features from |
| `Max iterations` | `5` | How many review cycles before escalating |
| `Review wait time` | `240` seconds | How long to wait for GitHub Actions |

---

## 📊 Success Metrics

### Good Outcome ✅
- Converges in 2-4 iterations
- Each iteration reduces P0 count
- Final review shows 0 P0 issues
- Total time: 30-60 minutes

### Warning Signs ⚠️
- Same issues repeat across iterations (agent not learning)
- P0 count increases (fixes introducing bugs)
- Takes >5 iterations (diminishing returns)
- Each iteration takes >10 minutes (inefficient)

### Escalation Triggers 🚨
- Iteration 5 still has P0 issues → Manual review needed
- Issues diverging (getting worse) → Architecture problem
- Agent confused about scope → Clarify requirements
- Build failures recurring → Technical debt cleanup needed

---

## 🔧 Maintenance

### When to Update This Prompt
- GitHub Actions review time changes (currently ~4 minutes)
- pr-review-resolver agent capabilities expand
- New review patterns emerge
- Integration with other tools (CI/CD, deployment)

### Version History
- v1.0 (2025-10-08): Initial autonomous orchestrator
  - Hardcoded 15 min waits (too long)
  - Single PR URL only

- v2.0 (2025-10-08): Generic orchestrator
  - 4 min waits (optimal)
  - Feature requests + PR URLs + issues
  - Automatic branch creation
  - Parallel work support

---

**Last Updated**: 2025-10-08
**Tested With**: PR #4 (ICPI backend refactor)
**Success Rate**: 3 iterations converging toward approval


---

## 🔧 Pattern: Checkpoint Merges (Multi-Phase Projects)

**Use Case:** Large remediation plans with 5+ independent phases

**DON'T:** Create one massive PR for all 50 fixes  
**DO:** Create PR per logical phase, merge as checkpoints

**Example: Security remediation with Phases 1-5**
```bash
# Phase 1: Critical fixes
git worktree add ../project-phase1 -b fix/phase1-critical main
# ... implement, test, PR, merge

# Phase 2: Admin controls (can start while Phase 1 in review)
git worktree add ../project-phase2 -b fix/phase2-admin main
# ... implement, test, PR, merge

# Phase 3: Medium fixes
git worktree add ../project-phase3 -b fix/phase3-medium main
# ... implement, test, PR, merge
```

**Benefits:**
- ✅ Smaller, reviewable PRs (200-500 lines vs 2000+)
- ✅ Can merge progress even if later phases blocked
- ✅ Each merge creates safe rollback point
- ✅ Reduces rebase complexity (smaller diffs)
- ✅ Enables true parallel work (different agents, different phases)

**When NOT to use:**
- Changes are tightly coupled (must go together)
- Project has <3 logical phases
- Each phase is tiny (<50 lines)

---

## 🔀 Handling Merge Conflicts During Rebase

**Scenario:** Your PR branch falls behind main while in review

**Common Cause:** Another PR merged first, your branch needs updating

**Solution:**

```bash
# 1. Fetch and rebase
cd /path/to/worktree
git fetch origin
git rebase origin/main

# 2. If conflicts occur (you'll see this):
# CONFLICT (content): Merge conflict in src/foo.rs
# Auto-merging src/bar.rs
```

**Conflict Resolution Strategy:**

```rust
// File with conflict markers:
<<<<<<< HEAD
// This is current main (already merged code)
fn old_function() { ... }
=======
// This is YOUR code (new feature)
fn new_function() { ... }
>>>>>>> abc1234 (Your commit message)

// Resolution steps:
// 1. Understand BOTH sides
// 2. Usually keep YOUR new code
// 3. BUT integrate it with upstream structure changes
// 4. Check for renamed constants/types
```

**After resolving:**
```bash
# 3. Mark resolved
git add src/foo.rs src/bar.rs

# 4. Continue rebase
git rebase --continue

# 5. Force push (--force-with-lease is safe)
git push origin feature-branch --force-with-lease

# 6. Redeploy to test
./deploy.sh --network ic

# 7. Comment on PR that rebase is done
gh pr comment <NUM> --body "Rebased on latest main, re-tested on mainnet"
```

**Common Conflict Patterns:**

1. **Constant Renamed:**
```
error: cannot find value `OLD_CONSTANT` in module `crate::constants`
```
**Fix:** `rg "CONSTANT" src/ -A 2` to find new name

2. **Function Signature Changed:**
```
error: this function takes 2 arguments but 3 were supplied
```
**Fix:** Read the updated function to see new signature

3. **Import Path Changed:**
```
error: unresolved import `crate::old_module::Foo`
```
**Fix:** `rg "struct Foo" src/` to find new location

---

## 📋 Review Wait Strategy

**DON'T:** Block on review in a loop

```bash
# ❌ BAD: Blocks for 4 minutes doing nothing
sleep 240
gh pr view <NUM> --json comments
```

**DO:** Check status non-blockingly

```bash
# ✅ GOOD: Quick status check, then continue working
gh pr checks <PR_NUM>
# Output:
# claude-review  pending  1m30s  https://...
# (Still running, continue other work)

# OR:
# claude-review  pass     3m20s  https://...
# (Complete, read the review)
```

**Pattern for Sequential PRs:**

```bash
# Create PR #1
gh pr create --title "Phase 2" --body "..."
PR_1=9

# Don't wait - start Phase 3 immediately
git worktree add ../project-phase3 -b fix/phase3 main
cd ../project-phase3
# ... implement Phase 3

# Create PR #2
gh pr create --title "Phase 3" --body "..."
PR_2=12

# Now check if either review is done
gh pr checks $PR_1  # pending or complete?
gh pr checks $PR_2  # pending or complete?

# Fix whichever has feedback, continue others
```

**Pattern for Independent PRs:**

```bash
# Create PR A (admin controls)
# Create PR B (burn limits)
# Create PR C (tests)

# All 3 can be in review simultaneously
# Fix issues as they arise
# Merge in any order (if truly independent)
```

**When to Actually Wait:**

- User explicitly asks "wait for review"
- Next phase depends on this PR's feedback
- You have nothing else productive to do

Otherwise: continue working, check status when convenient.

---

## 🚨 Handling Deployment Breaking Changes

**Scenario:** Candid interface compatibility check fails

```
WARNING! Candid interface compatibility check failed for canister 'icpi_backend'.
You are making a BREAKING change.

Method debug_rebalancer is only in the expected type
Do you want to proceed? yes/No
```

**What This Means:**

- **Removed method:** Clients expecting it will break
- **Added method:** Safe (new feature)
- **Changed signature:** Breaking (clients use old signature)

**Decision Tree:**

1. **Is removed method used by frontend?**
   ```bash
   rg "debug_rebalancer" src/icpi_frontend/
   ```
   - **No results:** Safe to remove → `yes`
   - **Has results:** Update frontend first

2. **Is changed signature backward compatible?**
   - **Added optional param:** Usually safe
   - **Changed return type:** Breaking, needs frontend update

3. **Auto-accept if safe:**
   ```bash
   yes | ./deploy.sh --network ic
   ```

4. **Document in PR:**
   ```markdown
   ## Breaking Changes
   - Removed `debug_rebalancer` (replaced with `get_rebalancer_status`)
   - Frontend does not use this method ✅
   ```

---

## 🔄 Handling Cross-PR Dependencies

**Scenario:** PR B needs types/functions from PR A

**Option 1: Sequential (A must merge first)**
```bash
# Implement and merge PR A
git worktree add ../project-a -b fix/feature-a main
# ... implement, PR, merge

# THEN implement PR B (which uses A's code)
git pull origin main  # Get merged PR A
git worktree add ../project-b -b fix/feature-b main
# ... implement using A's types/functions
```

**Option 2: Stack PRs (B builds on A)**
```bash
# Implement PR A
git worktree add ../project-a -b fix/feature-a main
# ... implement, PR (don't merge yet)

# Implement PR B based on A's branch
git worktree add ../project-b -b fix/feature-b fix/feature-a
# ... implement using A's code
# Create PR B with base: fix/feature-a (not main)

# Merge order: A first, then B
```

**Option 3: Duplicate Code Temporarily**
```bash
# If A is still in review but B is urgent:
# Copy the types/functions you need into PR B
# Add comment: "TODO: Remove after PR A merges"
# After A merges: remove duplicates in follow-up commit
```

---

## 📊 Iteration Metrics & When to Escalate

**Good Iteration:**
- Review comes back within 3-5 minutes
- 0-2 P0 (blocking) issues found
- Fixes take <30 minutes
- Converges in 1-2 iterations
- **Action:** Continue to next phase

**Concerning Pattern:**
- Review takes >10 minutes (check if CI is stuck)
- Same issues repeat across iterations (not learning)
- P0 count increases (fixes introducing bugs)
- Fixes take >60 minutes each time
- **Action:** Ask user for guidance

**Escalation Triggers:**
- 3+ iterations on same PR with no progress
- Review identifies architectural issues (not small fixes)
- Build failures that persist after multiple fix attempts
- Test failures on mainnet that can't be reproduced
- **Action:** Report to user, propose alternative approach

---

## 🎯 Updated Quick Start Examples

### Example 1: Continue Multi-Phase Plan (Checkpoint Pattern)

```
You are continuing ICPI security remediation. Phases 1-3 are merged.

Implement M-4 (concurrent operation guards) from SECURITY_REMEDIATION_PLAN.md.

Working Directory: /home/theseus/alexandria/basket/
Current Branch: main (clean)

Use checkpoint workflow:
1. Create worktree for M-4
2. Implement, build, test
3. Deploy to mainnet
4. Create PR
5. Fix review issues if any
6. Merge when approved
7. Move to next fix

START NOW.
```

### Example 2: Fix Multiple PRs in Review

```
You have 3 PRs in review: #9, #10, #11

Check review status for all 3:
- If any have feedback: fix those first
- If all pending: start next phase
- If all approved: merge all, update main

Prioritize critical feedback over minor suggestions.

Working Directory: /home/theseus/alexandria/basket/

START NOW.
```

---

## 🛡️ Safety Checks Before Merge

**Always verify before merging:**

```bash
# 1. All CI checks passed
gh pr checks <PR_NUM>
# Should show: pass (not pending or fail)

# 2. Conflicts resolved
gh pr view <PR_NUM> --json mergeable
# Should show: "MERGEABLE" (not "CONFLICTING")

# 3. Deployed to mainnet successfully
# (Check deploy logs for "Deployment successful")

# 4. Manual smoke test passed
# (Test key functions on mainnet after deploy)

# If all ✅: safe to merge
gh pr merge <PR_NUM> --squash --delete-branch
```

---

**Version:** 2.1 (Updated October 8, 2025 with checkpoint pattern)
**Tested With:** ICPI Security Remediation (3 phases, 3 PRs, all merged)
**Success Rate:** 100% (all PRs converged in 1-2 iterations)
