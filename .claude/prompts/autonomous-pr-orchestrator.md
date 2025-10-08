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
