# Instructions for Next Agent

## Your Task

Iterate on PR #7 until approved, following the autonomous PR orchestrator methodology.

**PR URL**: https://github.com/AlexandriaDAO/basket/pull/7
**Branch**: `fix/frontend-loading-missing-api-methods`
**Working Directory**: `/home/theseus/alexandria/basket`
**Network**: Always mainnet (`--network ic`)

---

## Context

### What's Been Done
1. ✅ Fixed blank skeleton screen issue (backend missing API methods)
2. ✅ Added `get_index_state_cached` and 15+ missing endpoints
3. ✅ Deployed to mainnet and working
4. ✅ Attempted ic-use-actor refactor (failed, rolled back)
5. ⏳ **PR #7 awaiting review feedback**

### Current State
- Site is functional: https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io
- Backend deployed with all API methods
- Frontend using original (working) code
- Some performance issues remain (20s delays, missing data on first load)

### Branch Status
```bash
git branch
# * fix/frontend-loading-missing-api-methods

git log --oneline -5
# ba5987c - Docs: Document ic-use-internet-identity failure and alternatives
# 4614faa - ROLLBACK: Revert to original App.tsx
# 55e2da6 - Fix login: Use window.location.hostname
# ... (see git log for full history)
```

---

## Your Workflow

### Step 1: Check PR Review
```bash
cd /home/theseus/alexandria/basket
git checkout fix/frontend-loading-missing-api-methods
git pull origin fix/frontend-loading-missing-api-methods

# View PR comments
gh pr view 7 --comments
```

### Step 2: Address Feedback
Read Claude Code's review comments and fix any P0 (blocking) issues.

**Common areas to check**:
- Code quality issues
- Type safety
- Error handling
- Security concerns
- Performance issues

### Step 3: Build and Test
```bash
# Build frontend
cd src/icpi_frontend
npm run prebuild
npm run build
cd ../..

# If backend changes needed
cargo build --target wasm32-unknown-unknown --release -p icpi_backend
```

### Step 4: Deploy to Mainnet
**IMPORTANT**: Always deploy to mainnet, never local.

```bash
# Deploy everything
dfx deploy --network ic

# Or deploy specific canister
dfx deploy --network ic icpi_backend
dfx deploy --network ic icpi_frontend
```

### Step 5: Test on Mainnet
Visit: https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io

Verify:
- [ ] Site loads without errors
- [ ] Login works
- [ ] No console errors
- [ ] Fixed issues don't regress

### Step 6: Commit and Push

**CRITICAL**: Always commit after fixing issues and deploying.

```bash
# Stage all changes
git add -A

# Commit with clear message
git commit -m "Address PR review: <description of fixes>

<detailed explanation of what was fixed>

## Testing
✅ Built successfully
✅ Deployed to mainnet: <canister IDs>
✅ Tested at https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io
✅ Verified: <specific checks>

## Changes Made
- <file>: <what changed>
- <file>: <what changed>

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
"

# Push to branch (NOT main)
git push origin fix/frontend-loading-missing-api-methods

# Verify push succeeded
git log origin/fix/frontend-loading-missing-api-methods --oneline -1
```

**IMPORTANT Git Notes**:
- ✅ Push to `fix/frontend-loading-missing-api-methods` branch
- ❌ Never push to `main` directly
- ✅ Always pull before starting: `git pull origin fix/frontend-loading-missing-api-methods`
- ✅ Commit after EACH fix iteration, not in batches

### Step 7: Wait for Next Review
GitHub Actions will trigger another review (~4 minutes).

**Repeat Steps 1-6** until PR is approved (usually 2-4 iterations).

---

## Important Notes

### Always Deploy to Mainnet
```bash
# ✅ Correct
dfx deploy --network ic icpi_frontend

# ❌ Wrong - never use local
dfx deploy icpi_frontend
```

### Always Test on Mainnet After Deploy
After any change, verify at:
- Frontend: https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io
- Backend Candid UI: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=ev6xm-haaaa-aaaap-qqcza-cai

### Canister IDs (Mainnet)
- icpi_frontend: `qhlmp-5aaaa-aaaam-qd4jq-cai`
- icpi_backend: `ev6xm-haaaa-aaaap-qqcza-cai`
- ICPI token: `l6lep-niaaa-aaaap-qqeda-cai`

### Git Workflow
```bash
# Check current branch
git branch
# Should show: * fix/frontend-loading-missing-api-methods

# Pull latest before starting
git pull origin fix/frontend-loading-missing-api-methods

# After fixes, push
git push origin fix/frontend-loading-missing-api-methods
```

### Files to Know About
- `FRONTEND_REFACTOR_STATUS.md` - Full context of what was attempted
- `FRONTEND_REFACTOR_BLOCKED.md` - Why ic-use-actor refactor failed
- `QUICK_FIX_BUILD.md` - Build troubleshooting
- `src/icpi_frontend/src/App.tsx` - Main app (original working version)
- `src/icpi_frontend/src/hooks/useICPI.ts` - Data hooks (original working version)
- `src/icpi_backend/src/lib.rs` - Backend API (has fixes)

---

## If You Get Stuck

### Build Fails
```bash
# Re-install dependencies
npm install tailwindcss tailwindcss-animate @tailwindcss/typography --workspace-root --legacy-peer-deps
cd src/icpi_frontend && npm install --legacy-peer-deps
```

### Deploy Fails
```bash
# Check dfx identity
dfx identity whoami

# Check you're on mainnet
dfx ping ic
```

### Need to Revert a Commit
```bash
git revert HEAD
git push origin fix/frontend-loading-missing-api-methods
```

---

## Success Criteria

PR is approved when Claude Code review shows:
- ✅ 0 P0 (blocking) issues
- ✅ Build succeeds
- ✅ Deploys successfully
- ✅ Site functional on mainnet

Then merge to main:
```bash
git checkout main
git merge fix/frontend-loading-missing-api-methods
git push origin main
```

---

**START**: Begin by running `gh pr view 7 --comments` to see what needs to be fixed.
