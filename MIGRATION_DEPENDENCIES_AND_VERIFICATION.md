# Module Dependencies and Verification Guide

## Module Dependency Diagram

```
1_CRITICAL_OPERATIONS/
├─ minting/
│  ├─ imports: 6_INFRASTRUCTURE (math, errors, constants, logging)
│  ├─ imports: 2_CRITICAL_DATA (supply_tracker, portfolio_value)
│  ├─ imports: types (Account, TransferArgs)
│  └─ calls: ICPI ledger, ckUSDT ledger (external)
│
├─ burning/
│  ├─ imports: 6_INFRASTRUCTURE (math, errors, constants)
│  ├─ imports: 2_CRITICAL_DATA (token_queries, supply_tracker)
│  ├─ imports: types (TrackedToken, Account)
│  └─ calls: Token ledgers [ALEX, ZERO, KONG, BOB, ckUSDT] (external)
│
└─ rebalancing/
   ├─ imports: 6_INFRASTRUCTURE (math, errors, constants)
   ├─ imports: 2_CRITICAL_DATA (portfolio_value)
   ├─ imports: 3_KONG_LIQUIDITY (tvl, pricing)
   ├─ imports: 4_TRADING_EXECUTION (swaps)
   └─ calls: Kongswap backend (external)

2_CRITICAL_DATA/
├─ portfolio_value/
│  ├─ imports: 6_INFRASTRUCTURE (math, cache)
│  ├─ imports: 3_KONG_LIQUIDITY (tvl)
│  ├─ imports: types (TrackedToken, IndexState)
│  └─ calls: None (pure calculations)
│
├─ supply_tracker/
│  ├─ imports: 6_INFRASTRUCTURE (cache, validation)
│  ├─ imports: types (Nat)
│  └─ calls: ICPI ledger (external)
│
├─ token_queries/
│  ├─ imports: 6_INFRASTRUCTURE (cache)
│  ├─ imports: types (TrackedToken, Account)
│  └─ calls: Token ledgers (external)
│
└─ validation/
   ├─ imports: 6_INFRASTRUCTURE (errors, constants)
   └─ calls: None (pure validation)

3_KONG_LIQUIDITY/
├─ locker/
│  ├─ imports: types (Principal)
│  └─ calls: Kong Locker canister (external)
│
├─ pools/
│  ├─ imports: types (LPBalancesReply, UserBalancesReply)
│  └─ calls: Kongswap backend (external)
│
├─ tvl/
│  ├─ imports: 6_INFRASTRUCTURE (cache)
│  ├─ imports: types (TrackedToken, Decimal)
│  └─ calls: pools module (internal)
│
└─ pricing/
   ├─ imports: types (SwapAmountsReply)
   └─ calls: Kongswap backend (external)

4_TRADING_EXECUTION/
├─ swaps/
│  ├─ imports: 6_INFRASTRUCTURE (errors)
│  ├─ imports: types (SwapArgs, SwapReply)
│  └─ calls: Kongswap backend (external)
│
├─ approvals/
│  ├─ imports: types (ApproveArgs)
│  └─ calls: Token ledgers (external)
│
└─ slippage/
   ├─ imports: 6_INFRASTRUCTURE (math, constants)
   └─ calls: None (pure calculations)

5_INFORMATIONAL/
├─ display/
│  ├─ imports: 2_CRITICAL_DATA (portfolio_value - cached)
│  ├─ imports: 3_KONG_LIQUIDITY (tvl - cached)
│  ├─ imports: 6_INFRASTRUCTURE (cache)
│  └─ calls: None (formatting only)
│
├─ health/
│  ├─ imports: 2_CRITICAL_DATA (supply_tracker - cached)
│  └─ calls: None (status checks)
│
└─ cache/
   ├─ imports: 6_INFRASTRUCTURE (cache policy)
   └─ calls: None (cache management)

6_INFRASTRUCTURE/
├─ constants/ (no imports, no calls)
├─ errors/ (no imports, no calls)
├─ math/ (no imports, no calls - PURE)
├─ types/ (imports: candid, serde)
├─ logging/ (no external calls)
├─ cache/ (no external calls)
└─ rate_limiting/ (no external calls)
```

## Import Rules

1. **Upward imports forbidden**: Lower zones cannot import from higher zones
   - ✅ Zone 6 can import from: None
   - ✅ Zone 5 can import from: 6, 2 (cached), 3 (cached)
   - ✅ Zone 4 can import from: 6
   - ✅ Zone 3 can import from: 6
   - ✅ Zone 2 can import from: 6, 3
   - ✅ Zone 1 can import from: 6, 2, 3, 4

2. **Cache rules**:
   - Zones 1-4: NO caching for critical operations
   - Zone 5: ALWAYS use cached data
   - Zone 6: Provides cache infrastructure

3. **External call rules**:
   - Only zones 1-4 make external canister calls
   - Zone 5 is display only - no external calls
   - Zone 6 is utilities only - no external calls

---

## Phase Verification Commands

### Phase 5.2 Verification (Burning Module)
```bash
# After creating burning module files
cd /home/theseus/alexandria/daopad/src/icpi

# Check files exist
test -f src/icpi_backend/src/1_CRITICAL_OPERATIONS/burning/mod.rs && echo "✅ burning/mod.rs"
test -f src/icpi_backend/src/1_CRITICAL_OPERATIONS/burning/redemption_calculator.rs && echo "✅ redemption_calculator.rs"
test -f src/icpi_backend/src/1_CRITICAL_OPERATIONS/burning/token_distributor.rs && echo "✅ token_distributor.rs"
test -f src/icpi_backend/src/1_CRITICAL_OPERATIONS/burning/burn_validator.rs && echo "✅ burn_validator.rs"

# Try to compile
cargo check --manifest-path src/icpi_backend/Cargo.toml 2>&1 | grep -E "1_CRITICAL_OPERATIONS/burning" | head -5

# Run burning tests
cargo test --manifest-path src/icpi_backend/Cargo.toml burning::tests 2>&1 | grep -E "test result:|running"
```

### Phase 5.3 Verification (Rebalancing Module)
```bash
# Check files exist
test -f src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing/mod.rs && echo "✅ rebalancing/mod.rs"
test -f src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing/deviation_analyzer.rs && echo "✅ deviation_analyzer.rs"
test -f src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing/trade_executor.rs && echo "✅ trade_executor.rs"
test -f src/icpi_backend/src/1_CRITICAL_OPERATIONS/rebalancing/rebalance_timer.rs && echo "✅ rebalance_timer.rs"

# Try to compile
cargo check --manifest-path src/icpi_backend/Cargo.toml 2>&1 | grep -E "1_CRITICAL_OPERATIONS/rebalancing" | head -5

# Run rebalancing tests
cargo test --manifest-path src/icpi_backend/Cargo.toml rebalancing::tests 2>&1 | grep -E "test result:|running"
```

### Phase 5.4 Verification (Portfolio Data)
```bash
# Check Zone 2 structure
tree -L 2 src/icpi_backend/src/2_CRITICAL_DATA/ 2>/dev/null || \
  find src/icpi_backend/src/2_CRITICAL_DATA -type f -name "*.rs" | head -10

# Compile check
cargo check --manifest-path src/icpi_backend/Cargo.toml 2>&1 | grep -E "2_CRITICAL_DATA" | head -5
```

### Phase 5.5 Verification (Kong Integration)
```bash
# Check Zone 3 structure
tree -L 2 src/icpi_backend/src/3_KONG_LIQUIDITY/ 2>/dev/null || \
  find src/icpi_backend/src/3_KONG_LIQUIDITY -type f -name "*.rs" | head -10

# Compile check
cargo check --manifest-path src/icpi_backend/Cargo.toml 2>&1 | grep -E "3_KONG_LIQUIDITY" | head -5
```

### Phase 5.6 Verification (Trading)
```bash
# Check Zone 4 structure
tree -L 2 src/icpi_backend/src/4_TRADING_EXECUTION/ 2>/dev/null || \
  find src/icpi_backend/src/4_TRADING_EXECUTION -type f -name "*.rs" | head -10

# Compile check
cargo check --manifest-path src/icpi_backend/Cargo.toml 2>&1 | grep -E "4_TRADING_EXECUTION" | head -5
```

### Phase 5.7 Verification (Informational)
```bash
# Check Zone 5 structure
tree -L 2 src/icpi_backend/src/5_INFORMATIONAL/ 2>/dev/null || \
  find src/icpi_backend/src/5_INFORMATIONAL -type f -name "*.rs" | head -10

# Compile check
cargo check --manifest-path src/icpi_backend/Cargo.toml 2>&1 | grep -E "5_INFORMATIONAL" | head -5
```

### Overall Verification After Phase 5
```bash
# Count all new module files
find src/icpi_backend/src/[1-6]_* -name "*.rs" | wc -l
# Should be 40+ files

# Check compilation of entire project
cargo build --manifest-path src/icpi_backend/Cargo.toml --release 2>&1 | tail -5

# Run all tests
cargo test --manifest-path src/icpi_backend/Cargo.toml --all 2>&1 | grep "test result:"

# Check feature flags work
echo "Testing feature flag system..."
cargo test --manifest-path src/icpi_backend/Cargo.toml feature_flags 2>&1 | grep -E "test result:|ok"
```

---

## Mainnet Testing Strategy

### Step 1: Deploy with Legacy Mode
```bash
# Deploy to mainnet with all flags set to Legacy
dfx deploy icpi_backend --network ic --argument '(record {
  feature_flags = record {
    minting = "legacy";
    burning = "legacy";
    rebalancing = "legacy";
    query = "legacy";
  }
})'
```

### Step 2: Test Shadow Mode Component by Component
```bash
# Test minting in shadow mode
dfx canister --network ic call icpi_backend set_feature_flag '("minting", "shadow")'

# Perform test mint
dfx canister --network ic call icpi_backend mint_icpi '(100000)' # 0.1 ckUSDT

# Check logs for discrepancies
dfx canister --network ic logs icpi_backend | grep "Shadow Mode Comparison"
```

### Step 3: Gradual Migration
```bash
# If shadow mode shows matching results, migrate to refactored
dfx canister --network ic call icpi_backend set_feature_flag '("minting", "refactored")'

# Monitor for 24 hours
# If stable, continue with next component
dfx canister --network ic call icpi_backend set_feature_flag '("burning", "shadow")'
# ... repeat pattern
```

### Step 4: Rollback Procedure
```bash
# If issues detected, immediate rollback
dfx canister --network ic call icpi_backend set_feature_flag '("minting", "legacy")'
dfx canister --network ic call icpi_backend set_feature_flag '("burning", "legacy")'
dfx canister --network ic call icpi_backend set_feature_flag '("rebalancing", "legacy")'
dfx canister --network ic call icpi_backend set_feature_flag '("query", "legacy")'
```

---

## Success Metrics

After complete migration, verify:

1. **Code Organization**
   ```bash
   # No root-level .rs files except lib.rs
   ls src/icpi_backend/src/*.rs | grep -v lib.rs | wc -l
   # Should be: 0
   ```

2. **No Legacy Folder**
   ```bash
   test -d src/icpi_backend/src/legacy && echo "❌ Legacy exists" || echo "✅ Legacy removed"
   ```

3. **All Zones Populated**
   ```bash
   for i in {1..6}; do
     count=$(find src/icpi_backend/src/${i}_* -name "*.rs" | wc -l)
     echo "Zone $i: $count files"
   done
   ```

4. **Tests Pass**
   ```bash
   cargo test --manifest-path src/icpi_backend/Cargo.toml --all
   ```

5. **Feature Flags Control**
   ```bash
   dfx canister --network ic call icpi_backend get_feature_flags
   ```

6. **Shadow Mode Validation**
   ```bash
   # Run operations in shadow mode and check logs
   dfx canister --network ic logs icpi_backend | grep "Results MATCH" | wc -l
   ```

---

## Emergency Procedures

### If Compilation Breaks
```bash
# Restore from backup
cd /home/theseus/alexandria/daopad
git stash
git checkout main
```

### If Mainnet Breaks
```bash
# Deploy previous working version
dfx canister --network ic install icpi_backend --mode reinstall --wasm /path/to/working.wasm
```

### If Tests Fail
```bash
# Run specific test in verbose mode
RUST_BACKTRACE=1 cargo test --manifest-path src/icpi_backend/Cargo.toml [test_name] -- --nocapture
```