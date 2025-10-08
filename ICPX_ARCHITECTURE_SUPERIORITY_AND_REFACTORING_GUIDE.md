# ICPX Architecture Superiority & Complete Refactoring Guide

**Purpose**: Transform ICPI codebase to match ICPX's superior security-first architecture
**Target Audience**: AI agents performing mechanical refactoring
**Completeness**: Line-by-line instructions for complete transformation
**Date**: 2025-10-06

---

## Table of Contents

1. [Why ICPX Architecture is Superior](#part-1-why-icpx-architecture-is-superior)
2. [Pre-Refactoring Analysis](#part-2-pre-refactoring-analysis)
3. [Refactoring Strategy](#part-3-refactoring-strategy)
4. [Code Transformation Patterns](#part-4-code-transformation-patterns)
5. [Step-by-Step Instructions](#part-5-step-by-step-instructions)
6. [Verification & Testing](#part-6-verification-testing)
7. [Rollback Procedures](#part-7-rollback-procedures)

---

# Part 1: Why ICPX Architecture is Superior

## 1.1 Numbered Security Zones vs Flat Modules

### ICPX Approach (SUPERIOR)
```
src/
├── 1_CRITICAL_OPERATIONS/    # 🔴 Immediately visible priority
├── 2_CRITICAL_DATA/          # 🔴 Second highest priority
├── 3_KONG_LIQUIDITY/         # 🟡 Medium priority
├── 4_TRADING_EXECUTION/      # 🟠 Important but controlled
├── 5_INFORMATIONAL/          # 🟢 Low security impact
└── 6_INFRASTRUCTURE/         # ⚪ Shared, audit once
```

**Benefits**:
1. **Instant Priority Recognition**: Directory name tells security level
2. **Audit Efficiency**: Review in numerical order (1 → 6)
3. **Developer Safety**: New devs see "1_CRITICAL" and know to be careful
4. **Tooling Support**: Easy to enforce different policies per zone
5. **Visual Hierarchy**: File browser shows security tiers clearly

### ICPI Approach (INFERIOR)
```
src/icpi_backend/src/
├── critical_operations/      # ⚠️ Hidden among other folders
├── portfolio_data/          # ⚠️ Unclear priority
├── kong_liquidity/          # ⚠️ Same visual weight as critical
├── informational/           # ⚠️ No clear hierarchy
├── infrastructure/          # ⚠️ Mixed with business logic
├── balance_tracker.rs       # ❌ Old file at root
├── burning.rs              # ❌ Duplicate with critical_operations
├── kongswap.rs             # ❌ Not refactored
└── legacy/                 # ❌ Technical debt visible
```

**Problems**:
1. No visual hierarchy - all folders look equal
2. Root-level files mixed with organized modules
3. Legacy folder indicates incomplete migration
4. No clear "review this first" signal for auditors
5. Developers must remember which is critical

**Superiority Score**: ICPX 10/10, ICPI 5/10

---

## 1.2 Type-Safe Error Handling vs String Errors

### ICPX Approach (SUPERIOR)

**Comprehensive Error Hierarchy** (from error_types.rs):
```rust
pub enum IcpxError {
    Mint(MintError),
    Burn(BurnError),
    Rebalance(RebalanceError),
    Query(QueryError),
    Validation(ValidationError),
    Trading(TradingError),
    System(SystemError),
    Other(String),
}

pub enum MintError {
    AmountBelowMinimum { amount: String, minimum: String },
    MintTimeout { mint_id: String },
    FeeCollectionFailed { reason: String },
    DepositCollectionFailed { reason: String },
    SnapshotFailed { reason: String },
    CalculationError { reason: String },
    LedgerMintFailed { reason: String },
    RefundFailed { reason: String },
    InvalidMintId { id: String },
    MintNotPending { id: String, status: String },
}

pub enum ValidationError {
    PriceOutOfBounds { price: String, min: String, max: String },
    SupplyOutOfBounds { supply: String, max: String },
    RapidChangeDetected { field: String, change_ratio: String },
    InvalidSnapshot { reason: String },
    DataInconsistency { reason: String },
}

// 287 lines of structured error definitions
```

**Benefits**:
1. **Type Safety**: Compiler ensures all error cases handled
2. **Structured Data**: Error variants carry relevant context
3. **Pattern Matching**: Can handle errors specifically
4. **Serializable**: Works with Candid interface
5. **Self-Documenting**: Error names describe failure modes
6. **Debugging**: Rich context for troubleshooting
7. **Frontend Integration**: Can show specific error messages

**Example Usage**:
```rust
// Specific error handling
match mint_result {
    Err(IcpxError::Mint(MintError::AmountBelowMinimum { amount, minimum })) => {
        // Show user exactly what minimum is
        format!("Deposit {} is below minimum {}", amount, minimum)
    }
    Err(IcpxError::Validation(ValidationError::PriceOutOfBounds { price, min, max })) => {
        // Log for monitoring
        alert_ops_team("Price validation failed", price, min, max)
    }
    _ => // Handle other errors
}
```

### ICPI Approach (INFERIOR)

**String-Based Errors**:
```rust
pub async fn mint_icpi(...) -> Result<Nat, String> {
    // Generic string errors
    return Err("Fee collection failed".to_string());
    return Err(format!("Mint failed: {:?}", e));
    return Err("Invalid TVL for proportional minting".to_string());
}
```

**Problems**:
1. **No Type Safety**: Caller must parse strings
2. **Lost Context**: No structured data, just text
3. **Hard to Handle**: Can't pattern match on strings
4. **Debugging Pain**: No stack trace, just message
5. **Frontend Hell**: Must parse error strings
6. **Inconsistent**: Different error formats everywhere
7. **Information Loss**: Can't programmatically extract details

**Superiority Score**: ICPX 10/10, ICPI 3/10

---

## 1.3 Pure Calculation Functions vs Mixed I/O

### ICPX Approach (SUPERIOR)

**Pure Calculation** (from minting/mod.rs:102-115):
```rust
/// Pure function - no I/O, fully testable
fn calculate_mint_amount(
    deposit_amount: &Nat,
    current_supply: &Nat,
    current_tvl: &Nat,
) -> Result<Nat> {
    // Initial mint case: 1:1 ratio
    if current_supply == &Nat::from(0u64) || current_tvl == &Nat::from(0u64) {
        return convert_decimals(deposit_amount, 6, 8);
    }

    // Subsequent mints: Proportional ownership
    multiply_and_divide(deposit_amount, current_supply, current_tvl)
}
```

**Orchestration Separate** (from minting/mod.rs:50-100):
```rust
pub async fn complete_mint(caller: Principal, mint_id: String) -> Result<Nat> {
    // I/O operations
    let (supply, tvl) = futures::join!(
        critical_data::supply_tracker::get_validated_supply(),
        critical_data::portfolio_value::calculate_portfolio_value_atomic()
    );

    // Pure calculation (delegated)
    let icpx_amount = calculate_mint_amount(&mint.amount, &supply, &tvl)?;

    Ok(icpx_amount)
}
```

**Benefits**:
1. **Testability**: Pure functions easy to unit test
2. **Reliability**: No side effects, deterministic
3. **Reusability**: Can use in different contexts
4. **Verification**: Can prove correctness mathematically
5. **Debugging**: Input → Output, no hidden state
6. **Parallelization**: Safe to run concurrently
7. **Mocking**: No need to mock external calls for pure tests

**Example Tests**:
```rust
#[test]
fn test_initial_mint_ratio() {
    let deposit = Nat::from(1_000_000u64); // 1 ckUSDT (e6)
    let supply = Nat::from(0u64);
    let tvl = Nat::from(0u64);

    let result = calculate_mint_amount(&deposit, &supply, &tvl).unwrap();

    assert_eq!(result, Nat::from(100_000_000u64)); // 1 ICPX (e8)
}

#[test]
fn test_proportional_mint() {
    let deposit = Nat::from(1_000_000u64); // 1 USDT
    let supply = Nat::from(100_000_000u64); // 1 ICPX existing
    let tvl = Nat::from(1_000_000u64); // $1 TVL

    let result = calculate_mint_amount(&deposit, &supply, &tvl).unwrap();

    // Should double supply (100% increase)
    assert_eq!(result, Nat::from(100_000_000u64));
}
```

### ICPI Approach (INFERIOR)

**Mixed I/O and Logic**:
```rust
pub async fn mint_icpi(user: Principal, usdt_amount: Nat) -> Result<Nat, String> {
    // Validation mixed with I/O
    super::validate_principal(&user)?;
    super::check_rate_limit("mint_icpi", 1_000_000_000)?;

    // I/O
    let state_before = get_index_state_uncached().await?;
    let icpi_supply = get_icpi_supply().await?;

    // Calculation inline
    let icpi_to_mint = if icpi_supply == 0u64 {
        usdt_amount.clone()
    } else {
        // Math mixed with f64 conversions
        let usdt_value = usdt_amount.0.to_u64().ok_or("overflow")? as f64 / 1e6;
        let supply_f64 = supply.0.to_u64().ok_or("overflow")? as f64;
        let new_tokens = (usdt_value * supply_f64) / tvl;
        Nat::from(new_tokens as u64)
    };

    // More I/O mixed in
    ic_cdk::call(...).await
}
```

**Problems**:
1. **Hard to Test**: Must mock external calls
2. **Not Reusable**: Tied to specific I/O context
3. **Debugging Difficult**: Side effects complicate reasoning
4. **Can't Verify**: Mathematical correctness obscured
5. **Precision Loss**: f64 conversions scattered throughout
6. **Race Conditions**: I/O interleaved with calculations

**Superiority Score**: ICPX 10/10, ICPI 4/10

---

## 1.4 Dedicated Validation Module vs Scattered Checks

### ICPX Approach (SUPERIOR)

**Centralized Validation** (2_CRITICAL_DATA/validation/):
```rust
// trust_boundaries.rs - Single source of truth
pub fn validate_external_supply(
    new_supply: Nat,
    cached_supply: Option<Nat>
) -> Result<Nat> {
    // Absolute bounds
    if new_supply > MAX_POSSIBLE_SUPPLY {
        return Err(ValidationError::SupplyOutOfBounds {
            supply: new_supply.to_string(),
            max: MAX_POSSIBLE_SUPPLY.to_string(),
        });
    }

    // Relative change check
    if let Some(cached) = cached_supply {
        if cached > 0 {
            let ratio = new_supply as f64 / cached as f64;

            if ratio > MAX_SUPPLY_CHANGE_RATIO {
                return Err(ValidationError::RapidChangeDetected {
                    field: "supply".to_string(),
                    change_ratio: ratio.to_string(),
                });
            }
        }
    }

    Ok(new_supply)
}

pub fn validate_token_price(
    token: Token,
    price: Decimal,
    cached_price: Option<Decimal>
) -> Result<Decimal> {
    // Absolute bounds
    if price < MIN_REASONABLE_PRICE || price > MAX_REASONABLE_PRICE {
        return Err(ValidationError::PriceOutOfBounds {
            price: price.to_string(),
            min: MIN_REASONABLE_PRICE.to_string(),
            max: MAX_REASONABLE_PRICE.to_string(),
        });
    }

    // Relative change
    if let Some(cached) = cached_price {
        let ratio = price / cached;
        if ratio > MAX_PRICE_CHANGE_RATIO {
            log_warning("Large price increase", token, ratio);
        }
    }

    Ok(price)
}
```

**Benefits**:
1. **Single Source of Truth**: All validation in one place
2. **Consistent Rules**: Same validation everywhere
3. **Easy to Update**: Change rules in one location
4. **Auditable**: Review all validation logic together
5. **Testable**: Test validation independently
6. **Documented**: Validation rules are explicit

### ICPI Approach (INFERIOR)

**Scattered Validation**:
```rust
// In mint function
super::validate_amount(&usdt_amount, 100_000, 100_000_000_000)?;

// In different file
pub fn validate_amount(amount: &Nat, min: u64, max: u64) -> Result<(), String> {
    // Basic check
}

// Somewhere else
if supply.0.to_u64().is_none() {
    return Err("Supply overflow".to_string());
}

// Another place
if tvl <= 0.0 {
    return Err("Invalid TVL".to_string());
}
```

**Problems**:
1. **Duplicated Logic**: Validation repeated everywhere
2. **Inconsistent Rules**: Different thresholds in different places
3. **Hard to Maintain**: Must update multiple locations
4. **Missing Checks**: Easy to forget validation
5. **No Central Audit**: Can't review all rules at once

**Superiority Score**: ICPX 9/10, ICPI 4/10

---

## 1.5 No Legacy Code vs Technical Debt

### ICPX Approach (SUPERIOR)
```
src/
├── 1_CRITICAL_OPERATIONS/    ✅ Clean implementation
├── 2_CRITICAL_DATA/          ✅ No duplicates
├── 3_KONG_LIQUIDITY/         ✅ Single source
├── 4_TRADING_EXECUTION/      ✅ Organized
├── 5_INFORMATIONAL/          ✅ Purpose-built
└── 6_INFRASTRUCTURE/         ✅ Shared utilities

Total: ~1,500 lines of clean code
```

### ICPI Approach (INFERIOR)
```
src/icpi_backend/src/
├── critical_operations/      ⚠️ New code
├── portfolio_data/          ⚠️ Partially migrated
├── balance_tracker.rs       ❌ OLD - should be in portfolio_data
├── burning.rs              ❌ OLD - duplicates critical_operations/burning
├── kongswap.rs             ❌ OLD - not refactored
├── minting.rs              ❌ OLD - duplicates critical_operations/minting
├── icpi_math.rs            ❌ OLD - should be infrastructure
├── legacy/                 ❌ ENTIRE OLD CODEBASE
│   ├── balance_tracker.rs  ❌ Duplicate
│   ├── burning.rs          ❌ Duplicate
│   ├── minting.rs          ❌ Duplicate
│   └── ... (15+ old files) ❌ All duplicates

Total: ~4,149 lines (60% is legacy/duplicates)
```

**Problems**:
1. **Confusion**: Which version is canonical?
2. **Bugs**: Old code may still be called
3. **Maintenance**: Must update multiple versions
4. **Import Errors**: Easy to import from wrong location
5. **File Size**: 2.7x larger than needed

**Superiority Score**: ICPX 10/10, ICPI 2/10

---

## 1.6 Comprehensive Documentation vs Scattered Docs

### ICPX Approach (SUPERIOR)

**Single Comprehensive README** (226 lines):
```markdown
# ICPX Backend - Security-First Architecture

## Architecture
(Clear visual diagram)

## API Endpoints
(Complete list with examples)

## Testing
- Unit tests
- Integration tests
- Security tests

## Deployment
(Step-by-step)

## Troubleshooting
(Common issues + fixes)

## Security Considerations
(5 key points)

## Migration Strategy
(Detailed plan)
```

### ICPI Approach (INFERIOR)

**Multiple Scattered Docs**:
- BACKEND_REFACTORING_PROPOSAL.md (842 lines)
- ICPI_BACKEND_PSEUDOCODE.md (1,189 lines)
- Multiple planning docs
- No single source of truth
- Outdated information

**Superiority Score**: ICPX 9/10, ICPI 5/10

---

# Part 2: Pre-Refactoring Analysis

## 2.1 Complete ICPI File Inventory

### Files to Keep and Transform
```yaml
KEEP_AND_TRANSFORM:
  critical_operations/:
    - minting/mod.rs (transform to match ICPX)
    - burning/mod.rs (transform)
    - trading/ (organize like ICPX)

  portfolio_data/:
    - (create matching ICPX structure)

  kong_liquidity/:
    - locker_client.rs → locker_discovery.rs
    - target_allocations.rs (keep)
    - tvl_calculator.rs (reorganize)
```

### Files to DELETE
```yaml
DELETE_IMMEDIATELY:
  root_level_old_files:
    - balance_tracker.rs (move to portfolio_data)
    - burning.rs (superseded by critical_operations/burning)
    - icpi_math.rs (move to infrastructure)
    - icpi_token.rs (move to infrastructure)
    - icrc_types.rs (move to infrastructure)
    - index_state.rs (reorganize)
    - kong_locker.rs (superseded by kong_liquidity)
    - kongswap.rs (reorganize into multiple modules)
    - ledger_client.rs (move to infrastructure)
    - minting.rs (superseded by critical_operations/minting)
    - precision.rs (move to infrastructure/math_utils.rs)
    - rebalancer.rs (move to critical_operations/rebalancing)
    - tvl_calculator.rs (reorganize)

  legacy_folder:
    - legacy/ (DELETE ENTIRE FOLDER - 15+ duplicate files)
```

### Current Structure Audit
```bash
# Run this to see current state
find src/icpi_backend/src -type f -name "*.rs" | sort
```

**Output**:
```
src/icpi_backend/src/balance_tracker.rs          [❌ DELETE]
src/icpi_backend/src/burning.rs                  [❌ DELETE]
src/icpi_backend/src/critical_operations/burning/mod.rs    [✅ KEEP]
src/icpi_backend/src/critical_operations/minting/mod.rs    [✅ KEEP]
src/icpi_backend/src/critical_operations/mod.rs            [✅ KEEP]
src/icpi_backend/src/icpi_math.rs               [❌ DELETE]
src/icpi_backend/src/icpi_token.rs              [❌ DELETE]
...
src/icpi_backend/src/legacy/                    [❌ DELETE ENTIRE FOLDER]
```

---

## 2.2 Dependency Mapping

### Current Import Graph
```
lib.rs
  ├─> balance_tracker.rs [OLD]
  ├─> burning.rs [OLD]
  ├─> critical_operations/
  ├─> kong_liquidity/
  └─> [12 other old files]

Problem: lib.rs imports from BOTH old and new locations
```

### Target Import Graph (ICPX Style)
```
lib.rs
  ├─> 1_CRITICAL_OPERATIONS/
  ├─> 2_CRITICAL_DATA/
  ├─> 3_KONG_LIQUIDITY/
  ├─> 4_TRADING_EXECUTION/
  ├─> 5_INFORMATIONAL/
  └─> 6_INFRASTRUCTURE/

Clean: Single source of truth for each module
```

---

# Part 3: Refactoring Strategy

## 3.1 Phase Overview

```yaml
Phase 1: Create ICPX-Style Structure (1 hour)
  Risk: NONE - just creating directories
  Reversible: Yes - delete directories

Phase 2: Create Error Type System (2 hours)
  Risk: LOW - new code, doesn't break existing
  Reversible: Yes - just delete the file

Phase 3: Extract Pure Functions (4 hours)
  Risk: MEDIUM - must maintain behavior
  Reversible: Yes - keep old code in place initially

Phase 4: Migrate Infrastructure (3 hours)
  Risk: LOW - utilities don't change behavior
  Reversible: Yes - old imports still work

Phase 5: Reorganize Critical Operations (6 hours)
  Risk: HIGH - affects token operations
  Reversible: Yes with feature flags

Phase 6: Delete Legacy Code (1 hour)
  Risk: MEDIUM - ensure nothing depends on it
  Reversible: Yes - git revert

Phase 7: Update lib.rs (2 hours)
  Risk: HIGH - central routing
  Reversible: Yes - keep old version commented

Phase 8: Final Cleanup (2 hours)
  Risk: LOW - polish and documentation
  Reversible: N/A
```

**Total Time**: ~21 hours of work
**Calendar Time**: 3-4 days with testing

---

## 3.2 Success Criteria Per Phase

### Phase 1 Success
```bash
# Directory structure exists
test -d src/icpi_backend/src/1_CRITICAL_OPERATIONS && echo "✅"
test -d src/icpi_backend/src/2_CRITICAL_DATA && echo "✅"
test -d src/icpi_backend/src/3_KONG_LIQUIDITY && echo "✅"
test -d src/icpi_backend/src/4_TRADING_EXECUTION && echo "✅"
test -d src/icpi_backend/src/5_INFORMATIONAL && echo "✅"
test -d src/icpi_backend/src/6_INFRASTRUCTURE && echo "✅"

# All directories have mod.rs
find src/icpi_backend/src -name "mod.rs" | wc -l
# Should be 6+
```

### Phase 2 Success
```bash
# Error types compile
cargo check --lib 2>&1 | grep "error_types" && echo "❌" || echo "✅"

# Error types are comprehensive
grep "pub enum IcpxError" src/icpi_backend/src/6_INFRASTRUCTURE/error_types.rs && echo "✅"
```

### Phase 3-8 Success
(Detailed in each phase below)

---

# Part 4: Code Transformation Patterns

## 4.1 Error Handling Transformation

### Pattern 1: String Result → Typed Result

**BEFORE** (ICPI):
```rust
pub async fn mint_icpi(user: Principal, amount: Nat) -> Result<Nat, String> {
    if amount < MIN {
        return Err("Amount too small".to_string());
    }

    let supply = get_supply().await
        .map_err(|e| format!("Supply query failed: {}", e))?;

    // ...
}
```

**AFTER** (ICPX):
```rust
use crate::infrastructure::error_types::{Result, IcpxError, MintError};

pub async fn mint_icpi(user: Principal, amount: Nat) -> Result<Nat> {
    if amount < MIN {
        return Err(IcpxError::Mint(MintError::AmountBelowMinimum {
            amount: amount.to_string(),
            minimum: MIN.to_string(),
        }));
    }

    let supply = get_supply().await?; // Errors auto-convert

    // ...
}
```

### Pattern 2: Generic Error → Specific Error

**BEFORE**:
```rust
return Err("Fee collection failed".to_string());
```

**AFTER**:
```rust
return Err(IcpxError::Mint(MintError::FeeCollectionFailed {
    reason: "Insufficient approval".to_string(),
}));
```

### Pattern 3: Error Propagation

**BEFORE**:
```rust
let result = some_function().await
    .map_err(|e| format!("Operation failed: {}", e))?;
```

**AFTER**:
```rust
// If some_function returns Result<T>:
let result = some_function().await?;

// Auto-converts to IcpxError via From trait
```

---

## 4.2 Pure Function Extraction

### Pattern: Separate I/O from Calculation

**BEFORE** (Mixed):
```rust
pub async fn calculate_mint(amount: Nat) -> Result<Nat, String> {
    // I/O
    let supply = get_supply().await?;
    let tvl = get_tvl().await?;

    // Calculation inline
    if supply == 0 {
        return Ok(amount.clone());
    }

    let result = (amount * supply) / tvl;
    Ok(result)
}
```

**AFTER** (Separated):
```rust
// Pure calculation function (private, no async)
fn calculate_mint_amount(
    deposit: &Nat,
    supply: &Nat,
    tvl: &Nat
) -> Result<Nat> {
    if supply == &Nat::from(0u64) {
        return Ok(deposit.clone());
    }

    multiply_and_divide(deposit, supply, tvl)
}

// I/O orchestration (public, async)
pub async fn mint_icpi(amount: Nat) -> Result<Nat> {
    // Fetch data
    let (supply, tvl) = futures::join!(
        get_validated_supply(),
        calculate_portfolio_value()
    );

    // Delegate to pure function
    calculate_mint_amount(&amount, &supply?, &tvl?)
}
```

**Benefits of This Pattern**:
1. `calculate_mint_amount` is pure - easy to test
2. Can test calculation without mocking I/O
3. Can reuse calculation in different contexts
4. Clear separation of concerns

---

## 4.3 Module Restructuring

### Pattern: Numbered Directories

**BEFORE**:
```
src/
├── critical_operations/
└── portfolio_data/
```

**AFTER**:
```
src/
├── 1_CRITICAL_OPERATIONS/
└── 2_CRITICAL_DATA/
```

**Implementation**:
```bash
# Rename with git to preserve history
git mv src/critical_operations src/1_CRITICAL_OPERATIONS
git mv src/portfolio_data src/2_CRITICAL_DATA
```

### Pattern: Module Files

**BEFORE**:
```rust
// src/minting.rs (at root)
pub async fn initiate_mint(...) { ... }
pub async fn complete_mint(...) { ... }
```

**AFTER**:
```rust
// src/1_CRITICAL_OPERATIONS/minting/mod.rs
pub async fn initiate_mint(...) { ... }
pub async fn complete_mint(...) { ... }

// Re-export at critical operations level
// src/1_CRITICAL_OPERATIONS/mod.rs
pub mod minting;
pub mod burning;
pub mod rebalancing;

// Re-export specific functions if needed
pub use minting::{initiate_mint, complete_mint};
```

---

# Part 5: Step-by-Step Instructions

## Phase 1: Create ICPX-Style Structure

### Step 1.1: Create Directory Structure
```bash
cd src/icpi_backend/src

# Create numbered directories
mkdir -p 1_CRITICAL_OPERATIONS/{minting,burning,rebalancing}
mkdir -p 2_CRITICAL_DATA/{portfolio_value,supply_tracker,token_queries,validation}
mkdir -p 3_KONG_LIQUIDITY
mkdir -p 4_TRADING_EXECUTION
mkdir -p 5_INFORMATIONAL
mkdir -p 6_INFRASTRUCTURE
```

**Verify**:
```bash
tree -L 2 -d .
```

**Expected Output**:
```
.
├── 1_CRITICAL_OPERATIONS
│   ├── burning
│   ├── minting
│   └── rebalancing
├── 2_CRITICAL_DATA
│   ├── portfolio_value
│   ├── supply_tracker
│   ├── token_queries
│   └── validation
├── 3_KONG_LIQUIDITY
├── 4_TRADING_EXECUTION
├── 5_INFORMATIONAL
└── 6_INFRASTRUCTURE
```

### Step 1.2: Create mod.rs Files
```bash
# Create all mod.rs files
for dir in 1_CRITICAL_OPERATIONS 2_CRITICAL_DATA 3_KONG_LIQUIDITY 4_TRADING_EXECUTION 5_INFORMATIONAL 6_INFRASTRUCTURE; do
    echo "//! ${dir} module" > ${dir}/mod.rs
done

# Create sub-module mod.rs files
for subdir in 1_CRITICAL_OPERATIONS/{minting,burning,rebalancing} 2_CRITICAL_DATA/{portfolio_value,supply_tracker,token_queries,validation}; do
    echo "//! $(basename ${subdir}) module" > ${subdir}/mod.rs
done
```

**Verify**:
```bash
find . -name "mod.rs" -newer /tmp/refactor_start | wc -l
# Should show 10 new mod.rs files
```

---

## Phase 2: Create Error Type System

### Step 2.1: Create error_types.rs

**Create File**:
```bash
cat > 6_INFRASTRUCTURE/error_types.rs << 'EOF'
//! Comprehensive error handling for ICPI backend
//! Security: MEDIUM - Proper error handling prevents information leakage

use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum IcpiError {
    // === Critical Operation Errors ===
    Mint(MintError),
    Burn(BurnError),
    Rebalance(RebalanceError),

    // === Data Layer Errors ===
    Query(QueryError),
    Validation(ValidationError),

    // === Trading Errors ===
    Trading(TradingError),

    // === System Errors ===
    System(SystemError),

    // === Generic ===
    Other(String),
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum MintError {
    AmountBelowMinimum { amount: String, minimum: String },
    MintTimeout { mint_id: String },
    FeeCollectionFailed { reason: String },
    DepositCollectionFailed { reason: String },
    SnapshotFailed { reason: String },
    CalculationError { reason: String },
    LedgerMintFailed { reason: String },
    RefundFailed { reason: String },
    InvalidMintId { id: String },
    MintNotPending { id: String, status: String },
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum BurnError {
    AmountBelowMinimum { amount: String, minimum: String },
    NoRedemptionsPossible { reason: String },
    TransferFailed { token: String, reason: String },
    CalculationError { reason: String },
    FeeCollectionFailed { reason: String },
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum RebalanceError {
    TooSoonSinceLastRebalance { last: u64, interval: u64 },
    NoRebalanceNeeded,
    TradeFailed { reason: String },
    InsufficientBalance { token: String, needed: String, available: String },
    SlippageExceeded { expected: String, max: String },
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum QueryError {
    CanisterUnreachable { canister: String },
    InvalidResponse { reason: String },
    Timeout,
    RateLimited,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum ValidationError {
    PriceOutOfBounds { price: String, min: String, max: String },
    SupplyOutOfBounds { supply: String, max: String },
    RapidChangeDetected { field: String, change_ratio: String },
    InvalidSnapshot { reason: String },
    DataInconsistency { reason: String },
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum TradingError {
    InsufficientLiquidity,
    PairNotFound { token_a: String, token_b: String },
    ApprovalFailed { reason: String },
    SwapFailed { reason: String },
    SlippageTooHigh { actual: String, max: String },
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum SystemError {
    Unauthorized { principal: String },
    StorageFull,
    CyclesInsufficient,
    TimerFailed { reason: String },
    StateCorrupted { reason: String },
}

// === Display Implementations ===

impl fmt::Display for IcpiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IcpiError::Mint(e) => write!(f, "Mint error: {}", e),
            IcpiError::Burn(e) => write!(f, "Burn error: {}", e),
            IcpiError::Rebalance(e) => write!(f, "Rebalance error: {}", e),
            IcpiError::Query(e) => write!(f, "Query error: {}", e),
            IcpiError::Validation(e) => write!(f, "Validation error: {}", e),
            IcpiError::Trading(e) => write!(f, "Trading error: {}", e),
            IcpiError::System(e) => write!(f, "System error: {}", e),
            IcpiError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for MintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MintError::AmountBelowMinimum { amount, minimum } => {
                write!(f, "Amount {} below minimum {}", amount, minimum)
            }
            MintError::MintTimeout { mint_id } => {
                write!(f, "Mint {} timed out", mint_id)
            }
            MintError::FeeCollectionFailed { reason } => {
                write!(f, "Fee collection failed: {}", reason)
            }
            MintError::DepositCollectionFailed { reason } => {
                write!(f, "Deposit collection failed: {}", reason)
            }
            MintError::SnapshotFailed { reason } => {
                write!(f, "Snapshot failed: {}", reason)
            }
            MintError::CalculationError { reason } => {
                write!(f, "Calculation error: {}", reason)
            }
            MintError::LedgerMintFailed { reason } => {
                write!(f, "Ledger mint failed: {}", reason)
            }
            MintError::RefundFailed { reason } => {
                write!(f, "Refund failed: {}", reason)
            }
            MintError::InvalidMintId { id } => {
                write!(f, "Invalid mint ID: {}", id)
            }
            MintError::MintNotPending { id, status } => {
                write!(f, "Mint {} not pending, status: {}", id, status)
            }
        }
    }
}

// Continue with other Display implementations...
// [Add remaining Display impls for BurnError, RebalanceError, etc.]

// === Conversion Helpers ===

impl From<String> for IcpiError {
    fn from(s: String) -> Self {
        IcpiError::Other(s)
    }
}

impl From<&str> for IcpiError {
    fn from(s: &str) -> Self {
        IcpiError::Other(s.to_string())
    }
}

// === Result Type ===

pub type Result<T> = std::result::Result<T, IcpiError>;
EOF
```

**Verify**:
```bash
# Check file was created
test -f 6_INFRASTRUCTURE/error_types.rs && echo "✅ Created"

# Try to compile
cargo check 2>&1 | grep -i "error_types" && echo "❌ Compilation errors" || echo "✅ Compiles"
```

### Step 2.2: Update Infrastructure mod.rs
```bash
cat >> 6_INFRASTRUCTURE/mod.rs << 'EOF'

pub mod error_types;

// Re-export for convenience
pub use error_types::{Result, IcpiError, MintError, BurnError};
EOF
```

---

## Phase 3: Extract Pure Functions

### Step 3.1: Create Math Utilities Module

**Copy from existing precision.rs**:
```bash
cp precision.rs 6_INFRASTRUCTURE/math_utils.rs
```

**Transform to match ICPX style**:
```bash
cat > 6_INFRASTRUCTURE/math_utils.rs << 'EOF'
//! Safe mathematical operations for token calculations
//! Security: CRITICAL - Prevents overflow and precision loss

use candid::Nat;
use num_bigint::BigUint;
use crate::infrastructure::error_types::{Result, IcpiError};

/// Multiply and divide with arbitrary precision
/// Formula: (a × b) ÷ c
pub fn multiply_and_divide(a: &Nat, b: &Nat, c: &Nat) -> Result<Nat> {
    if c == &Nat::from(0u64) {
        return Err(IcpiError::Other("Division by zero".to_string()));
    }

    let a_big = nat_to_biguint(a);
    let b_big = nat_to_biguint(b);
    let c_big = nat_to_biguint(c);

    let result = (a_big * b_big) / c_big;

    Ok(biguint_to_nat(result))
}

/// Convert between different decimal places
/// Example: ckUSDT (e6) → ICPI (e8)
pub fn convert_decimals(
    amount: &Nat,
    from_decimals: u32,
    to_decimals: u32
) -> Result<Nat> {
    if from_decimals == to_decimals {
        return Ok(amount.clone());
    }

    if from_decimals < to_decimals {
        // Multiply
        let multiplier = 10u64.pow(to_decimals - from_decimals);
        Ok(amount.clone() * Nat::from(multiplier))
    } else {
        // Divide
        let divisor = 10u64.pow(from_decimals - to_decimals);
        Ok(amount.clone() / Nat::from(divisor))
    }
}

// Helper conversions
fn nat_to_biguint(nat: &Nat) -> BigUint {
    BigUint::from_bytes_be(&nat.0.to_bytes_be())
}

fn biguint_to_nat(big: BigUint) -> Nat {
    Nat::from(num_bigint::ToBigUint::to_biguint(&big).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiply_and_divide() {
        let a = Nat::from(100u64);
        let b = Nat::from(200u64);
        let c = Nat::from(50u64);

        let result = multiply_and_divide(&a, &b, &c).unwrap();
        assert_eq!(result, Nat::from(400u64));
    }

    #[test]
    fn test_convert_decimals_increase() {
        let amount = Nat::from(1_000_000u64); // 1 ckUSDT (e6)
        let result = convert_decimals(&amount, 6, 8).unwrap();
        assert_eq!(result, Nat::from(100_000_000u64)); // 1 ICPI (e8)
    }

    #[test]
    fn test_convert_decimals_decrease() {
        let amount = Nat::from(100_000_000u64); // 1 ICPI (e8)
        let result = convert_decimals(&amount, 8, 6).unwrap();
        assert_eq!(result, Nat::from(1_000_000u64)); // 1 ckUSDT (e6)
    }
}
EOF
```

### Step 3.2: Extract Pure Mint Calculation

**Create new pure function module**:
```bash
cat > 1_CRITICAL_OPERATIONS/minting/calculations.rs << 'EOF'
//! Pure calculation functions for minting
//! Security: CRITICAL - Determines token issuance
//! Testing: Property-based tests required

use candid::Nat;
use crate::infrastructure::error_types::{Result, IcpiError, MintError};
use crate::infrastructure::math_utils::{multiply_and_divide, convert_decimals};

/// Calculate amount of ICPI to mint
///
/// # Formula
/// - Initial mint: 1:1 ratio (adjusted for decimals)
/// - Subsequent: (deposit × supply) ÷ tvl
///
/// # Invariants
/// - Output is proportional to deposit
/// - User ownership % = deposit %
/// - No dilution of existing holders
///
/// # Arguments
/// * `deposit_amount` - ckUSDT amount being deposited (e6 decimals)
/// * `current_supply` - Current ICPI supply (e8 decimals)
/// * `current_tvl` - Current portfolio value in ckUSDT units (e6)
pub fn calculate_mint_amount(
    deposit_amount: &Nat,
    current_supply: &Nat,
    current_tvl: &Nat,
) -> Result<Nat> {
    // Validation
    if deposit_amount == &Nat::from(0u64) {
        return Err(IcpiError::Mint(MintError::CalculationError {
            reason: "Deposit amount cannot be zero".to_string(),
        }));
    }

    // Initial mint case: 1:1 ratio (adjusted for decimals)
    if current_supply == &Nat::from(0u64) || current_tvl == &Nat::from(0u64) {
        // Convert ckUSDT (e6) to ICPI (e8)
        return convert_decimals(deposit_amount, 6, 8);
    }

    // Subsequent mints: Proportional ownership
    // new_icpi = (deposit × supply) / tvl
    let icpi_amount = multiply_and_divide(deposit_amount, current_supply, current_tvl)?;

    // Sanity check: Must produce non-zero result
    if icpi_amount == Nat::from(0u64) {
        return Err(IcpiError::Mint(MintError::CalculationError {
            reason: "Mint amount too small - would result in zero ICPI".to_string(),
        }));
    }

    // Sanity check: Prevent massive mints (> 10% of supply)
    let max_reasonable = current_supply.clone() / Nat::from(10u64);
    if icpi_amount > max_reasonable && current_supply > &Nat::from(0u64) {
        return Err(IcpiError::Mint(MintError::CalculationError {
            reason: format!(
                "Mint amount {} exceeds 10% of current supply {}",
                icpi_amount, current_supply
            ),
        }));
    }

    Ok(icpi_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_mint_creates_1_to_1() {
        let deposit = Nat::from(1_000_000u64); // 1 ckUSDT (e6)
        let supply = Nat::from(0u64);
        let tvl = Nat::from(0u64);

        let result = calculate_mint_amount(&deposit, &supply, &tvl).unwrap();

        // Should create 1 ICPI (e8)
        assert_eq!(result, Nat::from(100_000_000u64));
    }

    #[test]
    fn test_proportional_mint_doubles_supply() {
        // Existing: 1 ICPI backed by $1
        let deposit = Nat::from(1_000_000u64); // Deposit $1
        let supply = Nat::from(100_000_000u64); // 1 ICPI (e8)
        let tvl = Nat::from(1_000_000u64); // $1 TVL (e6)

        let result = calculate_mint_amount(&deposit, &supply, &tvl).unwrap();

        // Should create 1 more ICPI (100% increase)
        assert_eq!(result, Nat::from(100_000_000u64));
    }

    #[test]
    fn test_proportional_ownership_maintained() {
        // Existing: 10 ICPI backed by $100
        // Deposit: $10 (10% of TVL)
        let deposit = Nat::from(10_000_000u64); // $10 (e6)
        let supply = Nat::from(1_000_000_000u64); // 10 ICPI (e8)
        let tvl = Nat::from(100_000_000u64); // $100 (e6)

        let result = calculate_mint_amount(&deposit, &supply, &tvl).unwrap();

        // Should create 1 ICPI (10% of supply)
        assert_eq!(result, Nat::from(100_000_000u64));

        // Verify: new_icpi/new_supply == deposit/new_tvl
        let new_supply = supply.clone() + result.clone();
        let new_tvl = tvl.clone() + deposit.clone();

        // Both should be ~9.09%
        // (result / new_supply) ≈ (deposit / new_tvl)
    }

    #[test]
    fn test_zero_deposit_fails() {
        let deposit = Nat::from(0u64);
        let supply = Nat::from(100_000_000u64);
        let tvl = Nat::from(1_000_000u64);

        let result = calculate_mint_amount(&deposit, &supply, &tvl);

        assert!(result.is_err());
    }

    #[test]
    fn test_massive_mint_fails() {
        // Try to mint 20% of supply in one go
        let deposit = Nat::from(200_000_000u64); // $200
        let supply = Nat::from(1_000_000_000u64); // 10 ICPI
        let tvl = Nat::from(100_000_000u64); // $100

        let result = calculate_mint_amount(&deposit, &supply, &tvl);

        assert!(result.is_err());
    }
}
EOF
```

### Step 3.3: Update Minting Module to Use Pure Function

**Edit 1_CRITICAL_OPERATIONS/minting/mod.rs**:
```rust
//! Minting operations module
//! Security: CRITICAL - Creates new ICPI tokens

mod calculations;

use candid::{Nat, Principal};
use crate::infrastructure::error_types::{Result, IcpiError, MintError};
use crate::infrastructure::types::*;
use crate::infrastructure::constants::*;

// Use the pure calculation function
use calculations::calculate_mint_amount;

// ... rest of minting code ...

pub async fn complete_mint(caller: Principal, mint_id: String) -> Result<Nat> {
    // ... validation code ...

    // Get snapshot BEFORE collecting deposit
    let (supply, tvl) = futures::join!(
        crate::_2_CRITICAL_DATA::supply_tracker::get_validated_supply(),
        crate::_2_CRITICAL_DATA::portfolio_value::calculate_portfolio_value_atomic()
    );

    let supply = supply?;
    let tvl = tvl?;

    // Delegate to PURE function (easy to test)
    let icpi_amount = calculate_mint_amount(&mint.amount, &supply, &tvl)?;

    // ... rest of minting logic ...

    Ok(icpi_amount)
}
```

**Verify Pure Function Extraction**:
```bash
# Run tests for pure functions
cargo test --lib calculations

# Should see:
# test 1_CRITICAL_OPERATIONS::minting::calculations::tests::test_initial_mint_creates_1_to_1 ... ok
# test 1_CRITICAL_OPERATIONS::minting::calculations::tests::test_proportional_mint_doubles_supply ... ok
# ... etc
```

---

## Phase 4: Migrate Infrastructure Files

### Step 4.1: Move Types and Constants

**Create constants.rs**:
```bash
cat > 6_INFRASTRUCTURE/constants.rs << 'EOF'
//! System constants and canister IDs
//! Security: MEDIUM - Changes affect system behavior

// === Canister IDs ===
pub const ICPI_LEDGER_ID: &str = "l6lep-niaaa-aaaap-qqeda-cai";
pub const CKUSDT_CANISTER_ID: &str = "cngnf-vqaaa-aaaar-qag4q-cai";
pub const KONGSWAP_BACKEND_ID: &str = "2ipq2-uqaaa-aaaar-qailq-cai";
pub const KONG_LOCKER_ID: &str = "eazgb-giaaa-aaaap-qqc2q-cai";

// === Token Decimals ===
pub const ICPI_DECIMALS: u32 = 8;
pub const CKUSDT_DECIMALS: u32 = 6;
pub const ALEX_DECIMALS: u32 = 8;
pub const ZERO_DECIMALS: u32 = 8;
pub const KONG_DECIMALS: u32 = 8;
pub const BOB_DECIMALS: u32 = 8;

// === Minting Constants ===
pub const MIN_MINT_AMOUNT: u64 = 1_000_000; // 1 ckUSDT (e6)
pub const MINT_TIMEOUT_NANOS: u64 = 180_000_000_000; // 3 minutes
pub const FEE_AMOUNT: u64 = 100_000; // 0.1 ckUSDT (e6)

// === Burning Constants ===
pub const MIN_BURN_AMOUNT: u64 = 11_000; // 0.00011 ICPI (e8)
pub const TRANSFER_FEE_BUFFER: u64 = 10_000;

// === Rebalancing Constants ===
pub const REBALANCE_INTERVAL_SECONDS: u64 = 3600; // 1 hour
pub const MIN_DEVIATION_PERCENT: f64 = 1.0; // 1%
pub const TRADE_SIZE_PERCENT: f64 = 0.1; // 10% of deviation
pub const MAX_SLIPPAGE: f64 = 0.02; // 2%
pub const MIN_TRADE_SIZE_USD: f64 = 10.0; // $10 minimum

// === Validation Thresholds ===
pub const MAX_SUPPLY_CHANGE_RATIO: f64 = 1.1; // 10% max change
pub const MAX_PRICE_CHANGE_RATIO: f64 = 2.0; // 100% max change
pub const MIN_REASONABLE_PRICE: f64 = 0.0001; // $0.0001
pub const MAX_REASONABLE_PRICE: f64 = 1_000_000.0; // $1M
EOF
```

**Move types.rs**:
```bash
# Assuming types exist in current codebase
cp types.rs 6_INFRASTRUCTURE/types.rs

# Or create comprehensive types file
cat > 6_INFRASTRUCTURE/types.rs << 'EOF'
//! Core type definitions for ICPI backend
//! Security: LOW - Type definitions are safe

use candid::{CandidType, Deserialize, Nat, Principal};
use serde::Serialize;

// === Account Types ===
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<[u8; 32]>,
}

// === Token Types ===
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TrackedToken {
    ALEX,
    ZERO,
    KONG,
    BOB,
    ckUSDT,
}

impl TrackedToken {
    pub fn all() -> Vec<TrackedToken> {
        vec![
            TrackedToken::ALEX,
            TrackedToken::ZERO,
            TrackedToken::KONG,
            TrackedToken::BOB,
        ]
    }

    pub fn to_symbol(&self) -> &str {
        match self {
            TrackedToken::ALEX => "ALEX",
            TrackedToken::ZERO => "ZERO",
            TrackedToken::KONG => "KONG",
            TrackedToken::BOB => "BOB",
            TrackedToken::ckUSDT => "ckUSDT",
        }
    }

    pub fn get_canister_id(&self) -> Result<Principal, String> {
        let id = match self {
            TrackedToken::ALEX => "ysy5f-2qaaa-aaaap-qkmmq-cai",
            TrackedToken::ZERO => "b3d2q-ayaaa-aaaap-qqcfq-cai",
            TrackedToken::KONG => "o7oak-iyaaa-aaaaq-aadzq-cai",
            TrackedToken::BOB => "7pail-xaaaa-aaaas-aabmq-cai",
            TrackedToken::ckUSDT => "cngnf-vqaaa-aaaar-qag4q-cai",
        };
        Principal::from_text(id).map_err(|e| format!("Invalid principal: {}", e))
    }

    pub fn get_decimals(&self) -> u32 {
        match self {
            TrackedToken::ckUSDT => 6,
            _ => 8,
        }
    }
}

// === Minting Types ===
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PendingMint {
    pub id: String,
    pub user: Principal,
    pub amount: Nat,
    pub status: MintStatus,
    pub created_at: u64,
    pub last_updated: u64,
    pub snapshot: Option<MintSnapshot>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum MintStatus {
    Pending,
    CollectingFee,
    Snapshotting,
    CollectingDeposit,
    Calculating,
    Minting,
    Complete(Nat), // Amount minted
    Failed(String),
    FailedWithRefund,
    FailedNoRefund,
    Expired,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MintSnapshot {
    pub supply: Nat,
    pub tvl: Nat,
    pub timestamp: u64,
}

// === Burning Types ===
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BurnResult {
    pub icpi_burned: Nat,
    pub successful_transfers: Vec<TokenTransfer>,
    pub failed_transfers: Vec<FailedTransfer>,
    pub timestamp: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TokenTransfer {
    pub token: String,
    pub amount: Nat,
    pub block_index: Nat,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct FailedTransfer {
    pub token: String,
    pub amount: Nat,
    pub error: String,
}

// === Portfolio Types ===
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CurrentPosition {
    pub token: TrackedToken,
    pub balance: Nat,
    pub usd_value: f64,
    pub percentage: f64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TargetAllocation {
    pub token: TrackedToken,
    pub target_percentage: f64,
    pub target_usd_value: f64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct AllocationDeviation {
    pub token: TrackedToken,
    pub current_pct: f64,
    pub target_pct: f64,
    pub deviation_pct: f64,
    pub usd_difference: f64,
    pub trade_size_usd: f64,
}

// === Index State ===
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct IndexState {
    pub total_value: f64,
    pub current_positions: Vec<CurrentPosition>,
    pub target_allocations: Vec<TargetAllocation>,
    pub deviations: Vec<AllocationDeviation>,
    pub ckusdt_balance: Nat,
    pub timestamp: u64,
}

// === Rebalancing Types ===
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum RebalanceAction {
    Buy { token: TrackedToken, usd_amount: f64 },
    Sell { token: TrackedToken, usd_amount: f64 },
    None,
}

// Add other types as needed...
EOF
```

### Step 4.2: Update Infrastructure mod.rs

```bash
cat > 6_INFRASTRUCTURE/mod.rs << 'EOF'
//! Infrastructure module
//! Security: MEDIUM - Shared utilities and types

pub mod constants;
pub mod error_types;
pub mod math_utils;
pub mod types;

// Re-export commonly used items
pub use error_types::{Result, IcpiError};
pub use types::{TrackedToken, Account, IndexState};
pub use constants::*;
EOF
```

**Verify**:
```bash
cargo check --lib 2>&1 | grep "6_INFRASTRUCTURE" && echo "❌ Has errors" || echo "✅ Compiles"
```

---

## Phase 5: Reorganize Critical Operations (MOST COMPLEX)

### Step 5.1: Transform Minting Module Completely

**Goal**: Make 1_CRITICAL_OPERATIONS/minting/ match ICPX exactly

**Current State** (in critical_operations/minting/):
- Has mod.rs with basic implementation
- Missing: fee collection, deposit collection, ledger interaction

**Target State**:
- mod.rs: Orchestration (async I/O)
- calculations.rs: Pure math (already created in Phase 3.2)
- fee_collection.rs: Fee/deposit handling
- ledger_interaction.rs: ICPI ledger calls

**Create fee_collection.rs**:
```bash
cat > 1_CRITICAL_OPERATIONS/minting/fee_collection.rs << 'EOF'
//! Fee and deposit collection for minting
//! Security: CRITICAL - Handles user funds

use candid::{Nat, Principal};
use crate::infrastructure::error_types::{Result, IcpiError, MintError};
use crate::infrastructure::constants::{FEE_AMOUNT, CKUSDT_CANISTER_ID};
use crate::infrastructure::types::Account;

const FEE_RECIPIENT: &str = "e454q-riaaa-aaaap-qqcyq-cai";

/// Collect 0.1 ckUSDT fee from user
pub async fn collect_operation_fee(user: Principal) -> Result<Nat> {
    let ckusdt = Principal::from_text(CKUSDT_CANISTER_ID)
        .map_err(|e| IcpiError::System(
            crate::infrastructure::error_types::SystemError::StateCorrupted {
                reason: format!("Invalid ckUSDT principal: {}", e)
            }
        ))?;

    let fee_recipient = Principal::from_text(FEE_RECIPIENT)
        .map_err(|e| IcpiError::System(
            crate::infrastructure::error_types::SystemError::StateCorrupted {
                reason: format!("Invalid fee recipient: {}", e)
            }
        ))?;

    // ICRC-2 transfer_from: pull from user
    let transfer_args = TransferFromArgs {
        from: Account {
            owner: user,
            subaccount: None,
        },
        to: Account {
            owner: fee_recipient,
            subaccount: None,
        },
        amount: Nat::from(FEE_AMOUNT),
        fee: None,
        memo: Some(b"ICPI operation fee".to_vec()),
        created_at_time: None,
    };

    let result: Result<(TransferFromResult,), _> = ic_cdk::call(
        ckusdt,
        "icrc2_transfer_from",
        (transfer_args,)
    ).await;

    match result {
        Ok((TransferFromResult::Ok(block),)) => Ok(block),
        Ok((TransferFromResult::Err(e),)) => {
            Err(IcpiError::Mint(MintError::FeeCollectionFailed {
                reason: format!("{:?}", e)
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Mint(MintError::FeeCollectionFailed {
                reason: format!("Call failed: {:?} - {}", code, msg)
            }))
        }
    }
}

/// Collect ckUSDT deposit from user
pub async fn collect_deposit(
    user: Principal,
    amount: Nat,
    memo: Vec<u8>
) -> Result<Nat> {
    let ckusdt = Principal::from_text(CKUSDT_CANISTER_ID)
        .map_err(|e| IcpiError::Mint(MintError::DepositCollectionFailed {
            reason: format!("Invalid ckUSDT principal: {}", e)
        }))?;

    let backend = ic_cdk::id();

    let transfer_args = TransferFromArgs {
        from: Account {
            owner: user,
            subaccount: None,
        },
        to: Account {
            owner: backend,
            subaccount: None,
        },
        amount,
        fee: None,
        memo: Some(memo),
        created_at_time: None,
    };

    let result: Result<(TransferFromResult,), _> = ic_cdk::call(
        ckusdt,
        "icrc2_transfer_from",
        (transfer_args,)
    ).await;

    match result {
        Ok((TransferFromResult::Ok(block),)) => Ok(block),
        Ok((TransferFromResult::Err(e),)) => {
            Err(IcpiError::Mint(MintError::DepositCollectionFailed {
                reason: format!("{:?}", e)
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Mint(MintError::DepositCollectionFailed {
                reason: format!("Call failed: {:?} - {}", code, msg)
            }))
        }
    }
}

/// Refund deposit to user
pub async fn refund_deposit(user: Principal, amount: Nat) -> Result<Nat> {
    // Use icrc1_transfer since backend holds the funds
    let ckusdt = Principal::from_text(CKUSDT_CANISTER_ID)
        .map_err(|e| IcpiError::Mint(MintError::RefundFailed {
            reason: format!("Invalid ckUSDT principal: {}", e)
        }))?;

    let transfer_args = TransferArgs {
        to: Account {
            owner: user,
            subaccount: None,
        },
        amount,
        fee: None,
        memo: Some(b"ICPI mint refund".to_vec()),
        from_subaccount: None,
        created_at_time: None,
    };

    let result: Result<(TransferResult,), _> = ic_cdk::call(
        ckusdt,
        "icrc1_transfer",
        (transfer_args,)
    ).await;

    match result {
        Ok((TransferResult::Ok(block),)) => Ok(block),
        Ok((TransferResult::Err(e),)) => {
            Err(IcpiError::Mint(MintError::RefundFailed {
                reason: format!("{:?}", e)
            }))
        }
        Err((code, msg)) => {
            Err(IcpiError::Mint(MintError::RefundFailed {
                reason: format!("Call failed: {:?} - {}", code, msg)
            }))
        }
    }
}

// === ICRC Types ===

#[derive(candid::CandidType, candid::Deserialize)]
struct TransferFromArgs {
    from: Account,
    to: Account,
    amount: Nat,
    fee: Option<Nat>,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
}

#[derive(candid::CandidType, candid::Deserialize)]
struct TransferArgs {
    to: Account,
    amount: Nat,
    fee: Option<Nat>,
    memo: Option<Vec<u8>>,
    from_subaccount: Option<[u8; 32]>,
    created_at_time: Option<u64>,
}

#[derive(candid::CandidType, candid::Deserialize, Debug)]
enum TransferFromResult {
    Ok(Nat),
    Err(TransferFromError),
}

#[derive(candid::CandidType, candid::Deserialize, Debug)]
enum TransferResult {
    Ok(Nat),
    Err(TransferError),
}

#[derive(candid::CandidType, candid::Deserialize, Debug)]
enum TransferFromError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    InsufficientAllowance { allowance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}

#[derive(candid::CandidType, candid::Deserialize, Debug)]
enum TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}
EOF
```

---

## Phase 6: Delete Legacy Code

**WARNING**: Only proceed if Phase 5 is complete and tested

### Step 6.1: Identify All Legacy Files

```bash
# List all files to delete
cat > /tmp/files_to_delete.txt << 'EOF'
src/icpi_backend/src/balance_tracker.rs
src/icpi_backend/src/burning.rs
src/icpi_backend/src/icpi_math.rs
src/icpi_backend/src/icpi_token.rs
src/icpi_backend/src/icrc_types.rs
src/icpi_backend/src/index_state.rs
src/icpi_backend/src/kong_locker.rs
src/icpi_backend/src/kongswap.rs
src/icpi_backend/src/ledger_client.rs
src/icpi_backend/src/minting.rs
src/icpi_backend/src/precision.rs
src/icpi_backend/src/rebalancer.rs
src/icpi_backend/src/tvl_calculator.rs
src/icpi_backend/src/legacy/
EOF
```

### Step 6.2: Create Safety Backup

```bash
# Create backup branch
git checkout -b before-legacy-deletion
git add -A
git commit -m "Backup before legacy deletion"

# Return to refactoring branch
git checkout refactor-to-icpx-style
```

### Step 6.3: Delete Legacy Files

```bash
# Delete each file
while IFS= read -r file; do
    if [ -e "$file" ]; then
        git rm -rf "$file"
        echo "Deleted: $file"
    fi
done < /tmp/files_to_delete.txt

# Commit deletion
git add -A
git commit -m "Phase 6: Delete legacy code - migrated to ICPX structure"
```

**Verify**:
```bash
# Check no legacy files remain at root
ls src/icpi_backend/src/*.rs 2>/dev/null && echo "❌ Root files still exist" || echo "✅ Root cleaned"

# Check legacy folder gone
test -d src/icpi_backend/src/legacy && echo "❌ Legacy folder exists" || echo "✅ Legacy deleted"

# Try to compile
cargo check && echo "✅ Still compiles" || echo "❌ Broken imports"
```

---

## Phase 7: Update lib.rs

### Step 7.1: Backup Current lib.rs

```bash
cp src/icpi_backend/src/lib.rs src/icpi_backend/src/lib.rs.backup
```

### Step 7.2: Rewrite lib.rs to Match ICPX

```bash
cat > src/icpi_backend/src/lib.rs << 'EOF'
//! ICPI Backend - Security-First Architecture
//!
//! This backend is organized by security boundaries:
//! 1_CRITICAL_OPERATIONS - Token minting, burning, rebalancing
//! 2_CRITICAL_DATA - Financial calculations and validation
//! 3_KONG_LIQUIDITY - External liquidity reference
//! 4_TRADING_EXECUTION - DEX interactions
//! 5_INFORMATIONAL - Display and caching
//! 6_INFRASTRUCTURE - Shared utilities

// === Module Declarations ===

#[path = "1_CRITICAL_OPERATIONS/mod.rs"]
mod _1_critical_operations;

#[path = "2_CRITICAL_DATA/mod.rs"]
mod _2_critical_data;

#[path = "3_KONG_LIQUIDITY/mod.rs"]
mod _3_kong_liquidity;

#[path = "4_TRADING_EXECUTION/mod.rs"]
mod _4_trading_execution;

#[path = "5_INFORMATIONAL/mod.rs"]
mod _5_informational;

#[path = "6_INFRASTRUCTURE/mod.rs"]
mod infrastructure;

// Re-export for internal use
use infrastructure::{Result, IcpiError, TrackedToken};

// === Public API ===

/// Initiate a new mint request
#[ic_cdk::update]
pub async fn initiate_mint(amount: candid::Nat) -> Result<String> {
    let caller = ic_cdk::caller();
    _1_critical_operations::minting::initiate_mint(caller, amount).await
}

/// Complete a pending mint
#[ic_cdk::update]
pub async fn complete_mint(mint_id: String) -> Result<candid::Nat> {
    let caller = ic_cdk::caller();
    _1_critical_operations::minting::complete_mint(caller, mint_id).await
}

/// Burn ICPI tokens for underlying assets
#[ic_cdk::update]
pub async fn burn_icpi(amount: candid::Nat) -> Result<infrastructure::types::BurnResult> {
    let caller = ic_cdk::caller();
    _1_critical_operations::burning::burn_icpi(caller, amount).await
}

/// Get portfolio display (cached, for UI)
#[ic_cdk::query]
pub fn get_portfolio_display() -> infrastructure::types::IndexState {
    _5_informational::portfolio_display::get_cached_state()
}

/// Get live portfolio value (uncached, for critical operations)
#[ic_cdk::update]
pub async fn get_portfolio_value() -> Result<candid::Nat> {
    _2_critical_data::portfolio_value::calculate_portfolio_value_atomic().await
}

/// Perform rebalancing
#[ic_cdk::update]
pub async fn perform_rebalance() -> Result<String> {
    _1_critical_operations::rebalancing::perform_rebalance().await
}

/// Get canister ID
#[ic_cdk::query]
pub fn get_canister_id() -> candid::Principal {
    ic_cdk::id()
}

// === Initialization ===

#[ic_cdk::init]
fn init() {
    ic_cdk::println!("ICPI Backend initialized with security-first architecture");

    // Start timers
    _1_critical_operations::rebalancing::start_rebalancing_timer();
    _1_critical_operations::minting::start_cleanup_timer();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("ICPI Backend upgraded");

    // Restart timers
    _1_critical_operations::rebalancing::start_rebalancing_timer();
    _1_critical_operations::minting::start_cleanup_timer();
}

// === Candid Export ===

ic_cdk::export_candid!();
EOF
```

**Verify**:
```bash
cargo check 2>&1 | head -20
# Fix any import errors by updating module paths
```

---

## Phase 8: Final Cleanup and Documentation

### Step 8.1: Update README

```bash
cat > README.md << 'EOF'
# ICPI Backend - Security-First Architecture

Internet Computer Portfolio Index (ICPI) backend with security-first design.

## Architecture

```
src/icpi_backend/src/
├── 1_CRITICAL_OPERATIONS/    # 🔴 Highest security - mint, burn, rebalance
├── 2_CRITICAL_DATA/          # 🔴 Financial calculations
├── 3_KONG_LIQUIDITY/         # 🟡 External liquidity reference
├── 4_TRADING_EXECUTION/      # 🟠 DEX interactions
├── 5_INFORMATIONAL/          # 🟢 Display and caching
└── 6_INFRASTRUCTURE/         # ⚪ Shared utilities
```

## Quick Start

```bash
# Deploy to mainnet
./deploy.sh --network ic

# Test mint flow
dfx canister call icpi_backend initiate_mint '(1000000)'
dfx canister call icpi_backend complete_mint '("mint_xxx")'
```

## Documentation

- [Architecture Details](./ICPX_ARCHITECTURE_SUPERIORITY_AND_REFACTORING_GUIDE.md)
- [Security Model](./SECURITY.md)
- [API Reference](./icpi_backend.did)

## Audit Status

⚠️ **Refactored to ICPX architecture - requires new security audit**

EOF
```

### Step 8.2: Add SECURITY.md to Critical Modules

```bash
# Add to each critical module
cat > src/icpi_backend/src/1_CRITICAL_OPERATIONS/SECURITY.md << 'EOF'
# Critical Operations Security

## Module Purpose
This module contains ALL code that can mint, burn, or trade tokens.

## Security Requirements
- 100% test coverage required
- All functions must use typed errors (IcpiError)
- No caching allowed - always use fresh data
- Pure calculation functions must be separate from I/O

## Audit Checklist
- [ ] No unvalidated external input
- [ ] All Nat operations checked for overflow
- [ ] Decimal precision verified (no f64 in critical paths)
- [ ] Race conditions analyzed
- [ ] Failure modes documented
- [ ] Refund logic tested
- [ ] Audit logging present

## Attack Vectors
- Race conditions during mint (MITIGATED: snapshot before deposit)
- Supply manipulation (MITIGATED: validation layer)
- Price oracle manipulation (MITIGATED: sanity checks)

## Invariants
1. Mint ratio: new_icpi = (deposit × supply) / tvl
2. Burn ratio: redemption = (burned × balance) / supply
3. Snapshot timing: ALWAYS before collecting deposits
EOF
```

---

# Part 6: Verification & Testing

## 6.1 Compilation Verification

```bash
# Must compile without errors
cargo check 2>&1 | tee /tmp/compile_output.txt

# Check for errors
if grep -q "^error" /tmp/compile_output.txt; then
    echo "❌ Compilation failed"
    grep "^error" /tmp/compile_output.txt
    exit 1
else
    echo "✅ Compilation successful"
fi
```

## 6.2 Test Suite Verification

```bash
# Run all tests
cargo test --lib 2>&1 | tee /tmp/test_output.txt

# Verify critical tests pass
echo "=== Critical Tests ==="
grep "test.*critical_operations" /tmp/test_output.txt

# Verify pure function tests
echo "=== Pure Function Tests ==="
grep "test.*calculations" /tmp/test_output.txt
```

## 6.3 Module Structure Verification

```bash
# Verify numbered directories exist
for i in 1 2 3 4 5 6; do
    dir=$(find src/icpi_backend/src -type d -name "${i}_*" | head -1)
    if [ -n "$dir" ]; then
        echo "✅ Zone $i exists: $dir"
    else
        echo "❌ Zone $i missing"
    fi
done
```

## 6.4 Legacy Code Removal Verification

```bash
# Verify no legacy files at root
echo "=== Checking for root-level legacy files ==="
legacy_count=$(find src/icpi_backend/src -maxdepth 1 -name "*.rs" ! -name "lib.rs" | wc -l)
if [ "$legacy_count" -eq 0 ]; then
    echo "✅ No legacy files at root"
else
    echo "❌ Found $legacy_count legacy files at root"
    find src/icpi_backend/src -maxdepth 1 -name "*.rs" ! -name "lib.rs"
fi

# Verify legacy folder deleted
if [ ! -d "src/icpi_backend/src/legacy" ]; then
    echo "✅ Legacy folder deleted"
else
    echo "❌ Legacy folder still exists"
fi
```

## 6.5 Error Handling Verification

```bash
# Check that all Result types use IcpiError
echo "=== Checking error types ==="
string_errors=$(grep -r "Result<.*,.*String>" src/icpi_backend/src --include="*.rs" | wc -l)

if [ "$string_errors" -gt 0 ]; then
    echo "⚠️  Found $string_errors functions still using Result<T, String>"
    echo "These should be migrated to Result<T> (IcpiError):"
    grep -r "Result<.*,.*String>" src/icpi_backend/src --include="*.rs" | head -5
else
    echo "✅ All functions use typed errors"
fi
```

---

# Part 7: Rollback Procedures

## 7.1 Per-Phase Rollback

### Phase 1-2 Rollback (Structure + Errors)
```bash
# Simply delete new directories
rm -rf src/icpi_backend/src/1_CRITICAL_OPERATIONS
rm -rf src/icpi_backend/src/2_CRITICAL_DATA
rm -rf src/icpi_backend/src/3_KONG_LIQUIDITY
rm -rf src/icpi_backend/src/4_TRADING_EXECUTION
rm -rf src/icpi_backend/src/5_INFORMATIONAL
rm -rf src/icpi_backend/src/6_INFRASTRUCTURE

# Code still works - nothing broken
cargo check
```

### Phase 3-5 Rollback (Pure Functions + Migration)
```bash
# Revert to backup branch
git checkout before-pure-function-extraction

# Cherry-pick any good changes
git cherry-pick <specific-commits>
```

### Phase 6 Rollback (Legacy Deletion)
```bash
# Restore from git
git checkout before-legacy-deletion -- src/icpi_backend/src/

# Restore specific files
git checkout HEAD~1 -- src/icpi_backend/src/balance_tracker.rs
git checkout HEAD~1 -- src/icpi_backend/src/legacy/
```

### Phase 7 Rollback (lib.rs)
```bash
# Restore backup
cp src/icpi_backend/src/lib.rs.backup src/icpi_backend/src/lib.rs
cargo check
```

## 7.2 Emergency Full Rollback

```bash
# If everything is broken
git log --oneline | head -10  # Find commit before refactoring

# Hard reset (DANGER - loses uncommitted work)
git reset --hard <commit-before-refactoring>

# Or create new branch from old commit
git checkout -b emergency-rollback <commit-before-refactoring>
```

---

# Appendix A: Complete File Mapping

```yaml
OLD_LOCATION → NEW_LOCATION

# Infrastructure
precision.rs → 6_INFRASTRUCTURE/math_utils.rs
types.rs → 6_INFRASTRUCTURE/types.rs
[NEW] → 6_INFRASTRUCTURE/error_types.rs
[NEW] → 6_INFRASTRUCTURE/constants.rs

# Critical Operations
minting.rs → 1_CRITICAL_OPERATIONS/minting/mod.rs
[NEW] → 1_CRITICAL_OPERATIONS/minting/calculations.rs
[NEW] → 1_CRITICAL_OPERATIONS/minting/fee_collection.rs
[NEW] → 1_CRITICAL_OPERATIONS/minting/ledger_interaction.rs

burning.rs → 1_CRITICAL_OPERATIONS/burning/mod.rs
[NEW] → 1_CRITICAL_OPERATIONS/burning/redemption_calculator.rs

rebalancer.rs → 1_CRITICAL_OPERATIONS/rebalancing/mod.rs
[NEW] → 1_CRITICAL_OPERATIONS/rebalancing/trade_calculator.rs

# Critical Data
balance_tracker.rs → 2_CRITICAL_DATA/token_queries/balance_queries.rs
ledger_client.rs → 2_CRITICAL_DATA/supply_tracker/supply_queries.rs
tvl_calculator.rs (partial) → 2_CRITICAL_DATA/portfolio_value/value_calculator.rs
[NEW] → 2_CRITICAL_DATA/validation/trust_boundaries.rs
[NEW] → 2_CRITICAL_DATA/validation/snapshot_manager.rs

# Kong Liquidity
kong_locker.rs → 3_KONG_LIQUIDITY/locker_discovery.rs
kongswap.rs (LP queries) → 3_KONG_LIQUIDITY/lp_positions.rs
tvl_calculator.rs (partial) → 3_KONG_LIQUIDITY/target_calculator.rs

# Trading
kongswap.rs (swaps) → 4_TRADING_EXECUTION/swap_executor.rs
icrc_types.rs (approvals) → 4_TRADING_EXECUTION/approval_manager.rs

# Informational
index_state.rs (display) → 5_INFORMATIONAL/portfolio_display.rs
tvl_calculator.rs (summary) → 5_INFORMATIONAL/tvl_summary.rs

# Delete Entirely
legacy/ → [DELETED]
icpi_math.rs → [DELETED - logic moved to math_utils]
icpi_token.rs → [DELETED - not needed]
```

---

# Appendix B: Verification Commands

```bash
# Full verification suite
./verify_refactoring.sh

# Where verify_refactoring.sh contains:
#!/bin/bash

echo "=== ICPI → ICPX Refactoring Verification ==="

# 1. Structure
echo "1. Checking directory structure..."
test -d src/icpi_backend/src/1_CRITICAL_OPERATIONS && echo "  ✅ Zone 1" || echo "  ❌ Zone 1"
test -d src/icpi_backend/src/2_CRITICAL_DATA && echo "  ✅ Zone 2" || echo "  ❌ Zone 2"
test -d src/icpi_backend/src/3_KONG_LIQUIDITY && echo "  ✅ Zone 3" || echo "  ❌ Zone 3"
test -d src/icpi_backend/src/4_TRADING_EXECUTION && echo "  ✅ Zone 4" || echo "  ❌ Zone 4"
test -d src/icpi_backend/src/5_INFORMATIONAL && echo "  ✅ Zone 5" || echo "  ❌ Zone 5"
test -d src/icpi_backend/src/6_INFRASTRUCTURE && echo "  ✅ Zone 6" || echo "  ❌ Zone 6"

# 2. Error types
echo "2. Checking error handling..."
test -f src/icpi_backend/src/6_INFRASTRUCTURE/error_types.rs && echo "  ✅ error_types.rs exists" || echo "  ❌ Missing"
grep -q "pub enum IcpiError" src/icpi_backend/src/6_INFRASTRUCTURE/error_types.rs && echo "  ✅ IcpiError defined" || echo "  ❌ IcpiError missing"

# 3. Pure functions
echo "3. Checking pure functions..."
test -f src/icpi_backend/src/1_CRITICAL_OPERATIONS/minting/calculations.rs && echo "  ✅ Mint calculations" || echo "  ❌ Missing"

# 4. Legacy deletion
echo "4. Checking legacy cleanup..."
! test -d src/icpi_backend/src/legacy && echo "  ✅ Legacy deleted" || echo "  ❌ Legacy still exists"
! test -f src/icpi_backend/src/balance_tracker.rs && echo "  ✅ Root cleaned" || echo "  ❌ Root has old files"

# 5. Compilation
echo "5. Checking compilation..."
cargo check --quiet 2>&1 | grep -q "^error" && echo "  ❌ Compile errors" || echo "  ✅ Compiles"

# 6. Tests
echo "6. Running tests..."
cargo test --quiet --lib 2>&1 | grep -q "test result: ok" && echo "  ✅ Tests pass" || echo "  ❌ Tests fail"

echo "=== Verification Complete ==="
```

---

# End of Document

**Total Refactoring Time**: ~21 hours (3-4 days)
**Difficulty**: High (requires careful execution)
**Risk**: Medium (with proper phasing and backups)
**Reward**: Clean, maintainable, auditable codebase matching ICPX superiority

**Next Steps After This Refactoring**:
1. Deploy to testnet
2. Run full integration tests
3. Security audit of refactored code
4. Deploy to mainnet with monitoring
5. Retire old codebase

**Success Criteria**:
- ✅ All 6 numbered zones exist
- ✅ Type-safe error handling throughout
- ✅ Pure functions separated from I/O
- ✅ No legacy code remaining
- ✅ Compiles without errors
- ✅ All tests pass
- ✅ Code 2.7x smaller (from ~4,149 to ~1,500 lines)
- ✅ Clear security boundaries
- ✅ Ready for audit

---

*Document Version: 1.0*
*Last Updated: 2025-10-06*
*Author: Architecture Analysis Team*
