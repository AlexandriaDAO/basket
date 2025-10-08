# ICPI Implementation Guide - START HERE

**Purpose**: This document tells you EXACTLY which guides to follow based on your situation.
**Date**: 2025-10-07
**Read this first**: Takes 2 minutes, saves hours of confusion

---

## Quick Decision Tree

```
┌─────────────────────────────────────────────────────────┐
│ What is the current state of the ICPI codebase?        │
└──────────────────────┬──────────────────────────────────┘
                       │
        ┌──────────────▼───────────────┐
        │ Do numbered directories      │
        │ (1_CRITICAL_OPERATIONS,      │
        │  2_CRITICAL_DATA, etc.)      │
        │ exist in src/icpi_backend/   │
        │ src/?                        │
        └──────┬──────────────┬────────┘
               │              │
            YES│              │NO
               │              │
    ┌──────────▼─────┐   ┌───▼────────────────────┐
    │ Use PR4 GUIDE  │   │ Use V3 GUIDE (Partial) │
    │ ✅ 100% Ready  │   │ ⏳ Foundation Ready    │
    └────────────────┘   └────────────────────────┘
```

---

## Scenario 1: You Have Numbered Directories (1-6)

**Your Situation**:
```bash
$ ls src/icpi_backend/src/
1_CRITICAL_OPERATIONS/
2_CRITICAL_DATA/
3_KONG_LIQUIDITY/
4_TRADING_EXECUTION/
5_INFORMATIONAL/
6_INFRASTRUCTURE/
```

**What To Do**: Use **ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md**

**Why**: The code is already refactored into zones, but has incomplete implementations (stubs). The PR4 guide fixes these stubs.

**This Guide Provides**:
- ✅ All 8 stub functions identified with exact file/line numbers
- ✅ Complete working code to replace each stub
- ✅ Execution order (6 stages) with dependency graph
- ✅ ICRC signature fixes
- ✅ Security enhancements
- ✅ Complete test suite (15+ tests fully written)
- ✅ Verification commands for every fix
- ✅ Pre-merge checklist

**Estimated Time**: 1-3 weeks depending on testing rigor

**Success Criteria**:
- 0 functions returning `Ok(Nat::from(0u64))` as stubs
- All ICRC calls use correct signatures
- 40+ tests passing
- Testnet deployment successful

---

## Scenario 2: You Have Flat Legacy Structure

**Your Situation**:
```bash
$ ls src/icpi_backend/src/
minting.rs
burning.rs
rebalancer.rs
kong_locker.rs
kongswap.rs
index_state.rs
... (no numbered directories)
```

**What To Do**: **Two-Phase Approach**

### Phase A: Use V3 Guide (Partial) for Foundation

**Document**: ICPI_REFACTORING_IMPLEMENTATION_GUIDE_V3.md

**Use These Sections** (100% complete):
1. ✅ **Pre-Flight Safety** (Steps 0.1-0.5) - Create backups
2. ✅ **Emergency Infrastructure Fix** (if needed) - Fix compilation
3. ✅ **Phase 0**: Preparation & baseline
4. ✅ **Phase 1**: Create numbered directory structure
5. ✅ **Phase 2**: Implement error type system
6. ✅ **Phase 3**: Extract pure functions
7. ✅ **Phase 4**: Migrate infrastructure

**After Completing V3 Phases 0-4, You'll Have**:
- Numbered directories (1-6) created
- Complete error type system
- Complete pure math module
- Constants and infrastructure

### Phase B: Use PR4 Guide for Implementations

**Document**: ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md

**After V3 Phases 0-4, switch to PR4 guide** to:
1. Fix all stub implementations
2. Correct ICRC signatures
3. Add security layers
4. Write complete tests
5. Deploy

**Why This Works**: V3 gives you the architecture, PR4 gives you the implementations.

---

## Scenario 3: You Have Mixed State (Partial Refactoring)

**Your Situation**:
```bash
$ ls src/icpi_backend/src/
1_CRITICAL_OPERATIONS/  # Some numbered dirs exist
2_CRITICAL_DATA/
minting.rs              # But legacy files still there
burning.rs
legacy/                 # And legacy folder exists
```

**What To Do**: **Hybrid Approach**

1. **Read** ICPI_DOCUMENTATION_ANALYSIS.md first
   - Understand current state
   - See what's duplicated
   - See what's missing

2. **Decision**:
   - If numbered zones are mostly complete → Use PR4 guide
   - If numbered zones are empty stubs → Complete V3 Phases 0-4, then use PR4 guide
   - If confused → Read the analysis doc gap analysis section

3. **Clean up duplicates** using V3 Phase 6 (legacy cleanup)

---

## How to Check Which Scenario You're In

### Run This Command:

```bash
cd /home/theseus/alexandria/daopad/src/icpi

# Check for numbered directories
if [ -d "src/icpi_backend/src/1_CRITICAL_OPERATIONS" ]; then
    echo "✅ SCENARIO 1: Have numbered directories"
    echo "→ Use: ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md"
    echo ""
    echo "Next steps:"
    echo "1. Read PR4 guide Issue #0 (stub inventory)"
    echo "2. Read PR4 guide Issue #1.5 (execution order)"
    echo "3. Start fixing stubs in dependency order"
else
    echo "⏳ SCENARIO 2: Flat legacy structure"
    echo "→ Use: ICPI_REFACTORING_IMPLEMENTATION_GUIDE_V3.md (Phases 0-4)"
    echo "→ Then: ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md"
    echo ""
    echo "Next steps:"
    echo "1. Read V3 guide Pre-Flight Safety"
    echo "2. Create backups (Step 0.3)"
    echo "3. Execute V3 Phases 0-4"
    echo "4. Switch to PR4 guide for implementations"
fi

# Check for partial refactoring
if [ -d "src/icpi_backend/src/1_CRITICAL_OPERATIONS" ] && [ -f "src/icpi_backend/src/minting.rs" ]; then
    echo ""
    echo "⚠️ WARNING: SCENARIO 3 detected (mixed state)"
    echo "You have both numbered directories AND legacy files"
    echo "→ Use: Hybrid approach (see Scenario 3 above)"
fi
```

---

## Document Quick Reference

| Document | Lines | Purpose | When to Use | Completeness |
|----------|-------|---------|-------------|--------------|
| **ICPI_DOCUMENTATION_ANALYSIS.md** | 600+ | Research findings | For understanding current state | ✅ 100% |
| **ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md** | 2,671 | Fix broken implementations | When you have numbered zones | ✅ 100% |
| **ICPI_REFACTORING_IMPLEMENTATION_GUIDE_V3.md** | 1,759 | Transform to numbered zones | When you have flat legacy structure | ⏳ 39% (Phases 0-4 complete) |
| **DOCUMENTATION_COMPLETION_SUMMARY.md** | Summary | Mission completion status | For understanding what's done | ✅ 100% |
| **THIS FILE (START_HERE.md)** | Guide | Navigation | Read this first! | ✅ 100% |

---

## Execution Paths

### Path 1: Fix Current Refactored Code (FASTEST)

**If you have numbered directories:**

```
1. Read ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md
   ├─ Section: Issue #0 (stub inventory)
   ├─ Section: Issue #1.5 (execution order)
   └─ Section: Issue #1 (all fixes)

2. Execute fixes in this order:
   Stage 1: Core queries (stubs 1-3)
   Stage 2: Portfolio calculations (stubs 4-5)
   Stage 3: Rebalancing (stubs 6-8) [optional]
   Stage 4: ICRC signatures
   Stage 5: Security
   Stage 6: Tests

3. Follow Pre-Merge Checklist

4. Deploy to testnet → mainnet
```

**Time**: 1-3 weeks
**Success Rate**: High (everything is documented)

---

### Path 2: Complete Architectural Refactoring (COMPREHENSIVE)

**If you have flat legacy structure:**

```
PART A: Architecture Foundation (V3 Guide)
├─ 1. Read ICPI_REFACTORING_IMPLEMENTATION_GUIDE_V3.md
├─ 2. Execute Pre-Flight Safety (backups!)
├─ 3. Execute Phase 0 (preparation)
├─ 4. Execute Phase 1 (create directories)
├─ 5. Execute Phase 2 (error types)
├─ 6. Execute Phase 3 (pure functions)
└─ 7. Execute Phase 4 (infrastructure)

PART B: Implementation (PR4 Guide)
├─ 8. Switch to ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md
├─ 9. Execute Issue #1 (fix all stubs)
├─ 10. Execute Issue #2 (ICRC signatures)
├─ 11. Execute Issue #3 (security)
├─ 12. Execute Issue #4 (tests)
└─ 13. Deploy

NOTE: V3 Phase 5-8 are incomplete. Use PR4 guide's complete
code examples as templates for migrating remaining modules.
```

**Time**: 3-5 weeks
**Success Rate**: Medium (requires some extrapolation for V3 Phase 5)

---

## What If I Get Stuck?

### During V3 Phases 0-4
**These are 100% complete** - if stuck:
1. Re-read the step carefully
2. Run the verification commands
3. Check error messages match expected output

### During V3 Phase 5 (Partial)
**Phase 5 has minting structure but not other modules**:
1. Follow the minting pattern shown
2. Reference PR4 guide for complete implementation code
3. Adapt PR4 fixes to new module locations

### During PR4 Execution
**This is 100% complete** - if stuck:
1. Check you followed execution order (Issue #1.5)
2. Verify dependencies were fixed first
3. Run verification commands
4. Check expected output

---

## Critical Success Factors

### ✅ DO These Things

1. **Create backups** before ANY changes (git commit + tag)
2. **Follow execution order** in PR4 guide (dependencies matter)
3. **Run verification commands** after each fix
4. **Test on testnet** before mainnet
5. **Read Issue #0** in PR4 guide (stub inventory) before starting
6. **Check compilation** after every file edit

### ❌ DON'T Do These Things

1. ❌ Skip the Pre-Flight Safety steps (you'll regret it)
2. ❌ Fix stubs in random order (dependencies will break)
3. ❌ Skip verification commands (you won't know if it worked)
4. ❌ Deploy to mainnet without testnet testing
5. ❌ Assume "similar to" means you understand the pattern
6. ❌ Use caching for critical financial operations

---

## Summary Command

**Run this to see your status:**

```bash
cd /home/theseus/alexandria/daopad/src/icpi

echo "=========================================="
echo "ICPI CODEBASE STATUS CHECK"
echo "=========================================="
echo ""

# Check structure
if [ -d "src/icpi_backend/src/1_CRITICAL_OPERATIONS" ]; then
    echo "✅ Structure: Numbered zones exist"
    echo "→ Guide: ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md"
    echo ""

    # Count stubs
    STUBS=$(grep -r "Ok(Nat::from(0u64))" src/icpi_backend/src/[1-2]_* --include="*.rs" | grep -v test | wc -l)
    echo "Stubs remaining: $STUBS"
    if [ $STUBS -eq 0 ]; then
        echo "✅ No stubs - implementations complete!"
    else
        echo "⏳ Need to fix $STUBS stubs - see PR4 guide Issue #0"
    fi

    # Count TODOs
    TODOS=$(grep -r "TODO:" src/icpi_backend/src/[1-6]_* --include="*.rs" | wc -l)
    echo "TODOs remaining: $TODOS"

    # Check ICRC errors
    ICRC1_TF=$(grep -r "icrc1_transfer_from" src/icpi_backend/src/1_CRITICAL_OPERATIONS --include="*.rs" | wc -l)
    if [ $ICRC1_TF -gt 0 ]; then
        echo "⚠️ ICRC signature errors: $ICRC1_TF (should use icrc2_transfer_from)"
        echo "→ See PR4 guide Issue #2"
    fi
else
    echo "⏳ Structure: Flat legacy structure"
    echo "→ Guide: ICPI_REFACTORING_IMPLEMENTATION_GUIDE_V3.md (Phases 0-4)"
    echo "→ Then:  ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md"
    echo ""
    echo "Root-level files found:"
    ls src/icpi_backend/src/*.rs 2>/dev/null | wc -l
fi

echo ""
echo "=========================================="
echo "Available Guides:"
echo "=========================================="
echo "1. ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md (2,671 lines) ✅ 100% Complete"
echo "2. ICPI_REFACTORING_IMPLEMENTATION_GUIDE_V3.md (1,759 lines) ⏳ Phases 0-4 Complete"
echo "3. ICPI_DOCUMENTATION_ANALYSIS.md (600+ lines) ✅ Research Complete"
echo ""
echo "Run: cat START_HERE.md | less"
echo "For: Complete navigation guide"
echo "=========================================="
```

---

## Step-by-Step Instructions for Fresh Agents

### You Are a Fresh Agent With Zero Context

**Step 1**: Run the status check above to see your situation

**Step 2**: Based on output, follow your path:

#### If You See: "✅ Structure: Numbered zones exist"

```
→ Open: ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md

→ Read these sections IN ORDER:
  1. How to Use This Guide
  2. Pre-Flight Safety Checklist
  3. Issue #0: Complete Stub Inventory
  4. Issue #1.5: Execution Order & Dependencies
  5. Issue #1: Fix All Stub Implementations
     - Apply fixes in Stage 1 → Stage 6 order
     - Run verification after each fix
  6. Issue #2: ICRC Transfer Signature Fixes
  7. Issue #3: Security Vulnerabilities
  8. Issue #4: Complete Test Coverage
  9. Pre-Merge Checklist

→ Execute mechanically - no questions needed
→ All code is copy-paste ready
→ All verification commands have expected output
```

#### If You See: "⏳ Structure: Flat legacy structure"

```
→ Open: ICPI_REFACTORING_IMPLEMENTATION_GUIDE_V3.md

→ Execute ONLY these phases (100% complete):
  1. Pre-Flight Safety & Assessment
  2. Emergency Infrastructure Fix (if needed)
  3. Phase 0: Preparation & Baseline
  4. Phase 1: Create Numbered Directory Structure
  5. Phase 2: Implement Error Type System
  6. Phase 3: Extract Pure Functions
  7. Phase 4: Migrate Infrastructure

→ After Phase 4 complete:
  - You now have numbered directories
  - You now have infrastructure foundation
  - Switch to ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md
  - Follow instructions from "Scenario 1" above

→ For Phase 5+ (module migration):
  - V3 guide shows minting pattern
  - Use PR4 guide's complete code as examples
  - Adapt to your module structure
```

---

## Common Questions

### Q: Which guide should I read first?

**A**: Run the status check command above. It will tell you exactly which guide to use.

### Q: Can I skip the V3 guide and just use PR4?

**A**: Only if you already have numbered directories (1-6). Check with:
```bash
ls src/icpi_backend/src/1_CRITICAL_OPERATIONS 2>/dev/null && echo "Yes, use PR4" || echo "No, need V3 first"
```

### Q: What if both guides seem relevant?

**A**: You probably have partial refactoring (Scenario 3):
1. Read ICPI_DOCUMENTATION_ANALYSIS.md (understand current state)
2. Decide if it's easier to:
   - Complete refactoring (V3) then fix stubs (PR4)
   - OR roll back to legacy and start V3 from scratch

### Q: The V3 guide says Phase 5 is incomplete. What do I do?

**A**: V3 Phase 5 shows the minting module pattern. For other modules:
1. Use the same pattern structure
2. Reference PR4 guide for the complete implementation code
3. Adapt to your specific module (burning, rebalancing, etc.)

### Q: How long will this take?

**A**:
- PR4 guide only: 1-3 weeks
- V3 + PR4 full refactor: 3-5 weeks
- V3 Phases 0-4 only: 2-3 days

### Q: Can I deploy after just fixing stubs?

**A**: Check PR4 guide "Pre-Merge Checklist" section:
- Must fix all critical stubs (Stage 1-2)
- Must fix ICRC signatures (Stage 4)
- Must add security (Stage 5)
- Must test on testnet first
- Then can deploy to mainnet

---

## Emergency Contact Information

### If You Find Issues With These Guides

**Problem**: Guide has errors, unclear instructions, or missing information

**Solution**:
1. Document the specific issue (which guide, which section, what's wrong)
2. Check DOCUMENTATION_COMPLETION_SUMMARY.md for known gaps
3. Reference ICPI_DOCUMENTATION_ANALYSIS.md for codebase details

### If Code Fails After Following Guide

**First**: Check you followed execution order correctly

**Second**: Run verification commands exactly as written

**Third**: Check expected output matches actual output

**Fourth**: Consult the troubleshooting section in the guide you're using

---

## Final Checklist Before Starting

- [ ] I ran the status check command
- [ ] I know which scenario I'm in (1, 2, or 3)
- [ ] I know which guide(s) to use
- [ ] I've read the "How to Use This Guide" section
- [ ] I've created backups (git commit + tag)
- [ ] I have the progress tracker ready
- [ ] I understand this will take 1-5 weeks
- [ ] I'm ready to follow instructions mechanically

---

## TL;DR (Too Long; Didn't Read)

**Have numbered directories?**
→ Use **ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md**
→ 100% ready, just follow it

**Have flat legacy files?**
→ Use **ICPI_REFACTORING_IMPLEMENTATION_GUIDE_V3.md** (Phases 0-4 only)
→ Then switch to **ICPI_PR4_ISSUES_FIX_GUIDE_COMPLETE.md**

**Not sure?**
→ Run the status check command at the top
→ It will tell you exactly what to do

---

**Remember**: The PR4 guide is 100% mechanically executable. The V3 guide's Phases 0-4 are complete. Together they provide a complete path from any starting state to working ICPI implementation.

**Good luck!**
