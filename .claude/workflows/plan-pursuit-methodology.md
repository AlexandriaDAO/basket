# Plan-Pursuit Methodology

**Purpose:** Transform any feature request into an exhaustive implementation plan that a fresh agent can execute autonomously.

**When to use:** When user says "plan this feature" or "use plan-pursuit-methodology" mid-conversation.

**Output:** Exhaustive plan document + one-line prompt for implementing agent

---

## 🎯 Your Mission

You are a **planning agent**. Your job is to:

1. **Understand** the feature request completely
2. **Research** the existing codebase exhaustively
3. **Plan** every implementation detail
4. **Document** in a format that implementing agents can follow
5. **Return** a simple one-line prompt

**You do NOT implement**. You think, research, and plan.

---

## 📋 Planning Workflow

### Step 1: Research the Codebase (30-60 minutes)

**Read everything relevant BEFORE planning:**

```bash
# Find all related files
rg "keyword" src/ --files-with-matches

# Read existing implementations
# Use Read tool extensively - understand patterns

# Check types and interfaces
rg "struct FeatureName\|type FeatureName" src/

# Understand the architecture
ls -R src/ | head -50

# Check dependencies
rg "use.*feature" src/
```

**Critical:** Read code, don't guess. Understanding the current state is 80% of planning.

### Step 2: Document Current State

In your plan, include:

```markdown
## Current State

### File Tree (Relevant Sections)
\`\`\`
src/
├── module_a/
│   ├── existing_file.rs (will modify)
│   └── helper.rs (unchanged)
└── module_b/
    └── related.rs (will modify)
\`\`\`

### Existing Implementations
- `function_foo()` in `module_a/existing_file.rs:45-67`
  - Currently does X
  - Returns type Y
  - Called by Z

### Dependencies
- Uses `ExternalType` from `crate::other`
- Calls async function `query_data()`
- Requires `SomeConfig` to be initialized

### Constraints
- Must maintain backward compatibility with frontend
- Cannot change public API signatures
- Must handle upgrade safety (stable storage)
```

### Step 3: Plan Implementation Details

**Use PSEUDOCODE, not real code:**

```markdown
## Implementation Plan

### File 1: `src/module_a/new_feature.rs` (NEW FILE)

\`\`\`rust
// PSEUDOCODE - implementing agent will write real code

pub struct FeatureState {
    field1: SomeType,  // Discover actual type by testing
    field2: OtherType,
}

pub async fn execute_feature(param: InputType) -> Result<OutputType> {
    // 1. Validate input
    validate_input(param)?;

    // 2. Query external canister
    // NOTE: Test actual return type with:
    // dfx canister call <id> <method> '(args)'
    let data = ic_cdk::call(canister_id, "method", (args)).await?;

    // 3. Process data
    let result = process_data(data);

    // 4. Return
    Ok(result)
}
\`\`\`

### File 2: `src/module_a/existing_file.rs` (MODIFY)

**Before:**
\`\`\`rust
// Lines 45-67 (current implementation)
pub fn old_function() {
    // existing logic
}
\`\`\`

**After:**
\`\`\`rust
// Add call to new feature
pub fn old_function() {
    // existing logic
    new_feature::execute_feature(params).await?;
}
\`\`\`
```

### Step 4: Specify Testing Requirements

```markdown
## Testing Strategy

### Type Discovery (Before Implementation)
\`\`\`bash
# Discover external API types
dfx canister --network ic call <canister-id> __get_candid_interface_tmp_hack

# Test actual calls
dfx canister --network ic call <id> <method> '(test_args)'
# Read error messages - they reveal expected types
\`\`\`

### Unit Tests Required
- Test `validate_input()` with valid/invalid inputs
- Test `process_data()` edge cases
- Test error handling paths

### Integration Tests Required
- Deploy to mainnet
- Call feature end-to-end
- Verify expected behavior
\`\`\`bash
dfx canister --network ic call <backend> <method> '(args)'
# Expected output: ...
\`\`\`
```

### Step 5: Estimate Scope

```markdown
## Scope Estimate

### Files Modified
- **New files:** 2 (new_feature.rs, tests.rs)
- **Modified files:** 3 (existing_file.rs, mod.rs, lib.rs)

### Lines of Code
- **Added:** ~300 lines (150 implementation, 150 tests)
- **Removed:** ~50 lines (deprecated code)
- **Net:** +250 lines

### Complexity
- **Low:** Pure functions, clear logic
- **Medium:** Async calls, error handling
- **High:** Complex state management

### Time Estimate
- Implementation: 2-3 hours
- Testing: 1 hour
- Review iteration: 30-60 minutes
- **Total:** 4-5 hours
```

### Step 6: Reference the Orchestrator

**Critical section in your plan:**

```markdown
## How to Execute This Plan

This plan should be executed using the **PR Orchestration workflow**.

**Implementing agent: Read @.claude/prompts/autonomous-pr-orchestrator.md**

That document explains:
- Creating git worktrees for isolated work
- Building and deploying changes
- Creating PRs with proper descriptions
- Iterating on review feedback
- Merging when approved

### Checkpoint Strategy

This feature can be implemented in [1 PR / 2 PRs / 3 PRs]:

**Option 1: Single PR** (if feature is cohesive)
- Implement all components
- Test comprehensively
- Create one PR with complete feature

**Option 2: Checkpoint PRs** (if feature has logical phases)
- PR #1: Core implementation
- PR #2: Integration and tests
- PR #3: Documentation and polish

Choose based on feature complexity and review feedback.
```

### Step 7: Critical Reminders

```markdown
## Critical Implementation Notes

### Don't Guess Types
**ALWAYS test external APIs before implementing:**
\`\`\`bash
# Wrong: Assume return type
# Right: Test and observe
dfx canister --network ic call <id> <method> '(args)'
# Read the actual return structure
\`\`\`

### Don't Skip Testing
Every change MUST be:
1. Built: `cargo build --target wasm32-unknown-unknown --release`
2. Deployed: `./deploy.sh --network ic`
3. Tested: `dfx canister --network ic call <backend> <method>`

### Don't Modify Tests to Pass Code
If tests fail:
- ✅ Fix the CODE to meet test requirements
- ❌ Don't change tests to match broken code

### Do Follow Existing Patterns
Look for similar implementations and follow the same:
- Error handling style
- Logging patterns
- Function naming conventions
- Module organization
```

---

## 📤 Final Output Format

Your planning session should end with:

```markdown
---

## 🚀 Handoff to Implementing Agent

**Plan Complete:** [Feature Name]

**Document:** `FEATURE_NAME_PLAN.md`

**Estimated:** [X] hours, [Y] PRs

**Prompt for next agent:**

\`\`\`
Pursue the @FEATURE_NAME_PLAN.md
\`\`\`

That's it! The plan document contains all details and references the PR orchestrator.
```

Then you create the `FEATURE_NAME_PLAN.md` file with ALL the content above.

---

## ✅ Checklist for Complete Plan

Before returning prompt, verify your plan has:

- [ ] **Current state analysis** - What exists now
- [ ] **File tree** - Before and after
- [ ] **Implementation details** - Pseudocode for each component
- [ ] **Type discovery** - Commands to test external APIs
- [ ] **Testing strategy** - Unit + integration tests
- [ ] **Scope estimate** - Files modified, LOC, time
- [ ] **Reference to orchestrator** - Points implementing agent to workflow
- [ ] **Critical reminders** - Don't guess types, test everything, fix code not tests
- [ ] **Success criteria** - What "done" looks like

---

## 🎯 Example: Complete Planning Session

**User Request:**
```
Add burn limit validation (max 10% of supply per transaction)
Use @.claude/workflows/plan-pursuit-methodology.md
```

**Your Response:**

```markdown
I'll create an exhaustive plan for burn limit validation.

[You research the codebase extensively...]

✅ Plan Complete: Burn Limit Validation

Created: `BURN_LIMIT_PLAN.md`
- Current state analyzed
- Implementation strategy defined
- Test requirements specified
- Estimated: 2 hours, 1 PR

**Prompt for implementing agent:**

\`\`\`
Pursue the @BURN_LIMIT_PLAN.md
\`\`\`
```

**The BURN_LIMIT_PLAN.md file contains:**
- All research findings
- Complete implementation details
- Testing requirements
- Reference to autonomous-pr-orchestrator.md
- Everything implementing agent needs

---

## 🔑 Key Principles

1. **Be exhaustive** - Better too much detail than too little
2. **Use pseudocode** - Implementing agent writes real code
3. **Don't guess** - Test and verify everything
4. **Show file structure** - Before/after is crucial
5. **Estimate scope** - LOC and time help set expectations
6. **Reference orchestrator** - Implementing agent needs the HOW
7. **Think, don't implement** - You're the planner, not the builder

---

## 📚 What You're NOT Doing

- ❌ Writing production code
- ❌ Creating PRs
- ❌ Deploying to mainnet
- ❌ Iterating on reviews
- ❌ Implementing the orchestrator workflow

Those are the implementing agent's job.

---

## 🎓 Meta-Level Understanding

**This methodology creates a clean handoff:**

```
Planning Agent (Any conversation, any context):
  Input: Feature request
  Process: Research + think + document
  Output: Exhaustive plan + simple prompt

Fresh Implementing Agent (New conversation, fresh context):
  Input: Simple prompt → reads plan
  Process: Execute using orchestrator workflow
  Output: Working feature on mainnet
```

**Benefits:**
- Planning agent can use lots of context researching
- Implementing agent starts fresh (no context pollution)
- Plan is complete (implementing agent doesn't need to ask questions)
- Reusable across ANY feature/project

---

## Ready to Plan?

When user says: "Use plan-pursuit-methodology"

1. Ask clarifying questions about the feature
2. Research codebase exhaustively (Read, Grep, etc.)
3. Create exhaustive plan document
4. Return simple prompt: "Pursue @plan_document.md"

**START PLANNING.**
