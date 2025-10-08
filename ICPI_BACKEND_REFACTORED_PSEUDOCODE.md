# ICPX Backend - Security-First Refactored Architecture

**Purpose**: Reorganized ICPX backend architecture prioritizing security boundaries and audit clarity
**Date Generated**: 2025-10-06
**Version**: 2.0 - Security Refactoring
**Base Document**: ICPX_BACKEND_PSEUDOCODE.md

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Design Principles](#design-principles)
3. [High-Level Architecture](#high-level-architecture)
4. [Security Zones](#security-zones)
5. [Module Organization](#module-organization)
6. [Detailed Module Pseudocode](#detailed-module-pseudocode)
7. [Data Flow Diagrams](#data-flow-diagrams)
8. [Security Improvements](#security-improvements)
9. [Invariants Documentation](#invariants-documentation)
10. [Migration Strategy](#migration-strategy)
11. [Testing Requirements](#testing-requirements)
12. [Open Questions](#open-questions)

---

## Executive Summary

### Why This Refactoring?

The current ICPX backend architecture mixes critical financial operations with informational queries, making security audits difficult and increasing the risk of bugs in critical paths. This refactoring reorganizes the codebase by **security boundaries** rather than functional domains.

### Key Benefits

1. **Clear Audit Boundaries**: Critical financial code isolated for focused review
2. **Explicit Trust Validation**: All external data validated at entry points
3. **Separated Concerns**: Portfolio value (critical) vs locked liquidity (informational)
4. **Testability**: Pure calculation functions separated from I/O operations
5. **Reduced Attack Surface**: Critical operations have minimal dependencies

### Critical Findings from Analysis

- **Confusion Risk**: "TVL" means two different things (portfolio value vs locked liquidity)
- **Mixed Security Levels**: Critical and informational code in same modules
- **Missing Validations**: No sanity checks on external price/supply data
- **Race Condition Risk**: No atomicity guarantees for multi-step operations

---

## Design Principles

```yaml
principles:
  1. Security-First Organization:
     - Group by security impact, not feature
     - Critical operations isolated and minimal

  2. Clear Separation of Concerns:
     - Portfolio holdings (internal) vs Locked liquidity (external)
     - Financial calculations (pure) vs External I/O (impure)
     - Live queries (critical) vs Cached data (display)

  3. Explicit Trust Boundaries:
     - Validate ALL external data
     - Document validation requirements
     - Fail fast on invalid data

  4. Auditable Critical Paths:
     - Minimal code in critical zones
     - Clear data provenance
     - Comprehensive invariant documentation
```

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     User Interface                           │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                      lib.rs                                  │
│                  (API Router)                                │
└──────┬──────────────┬──────────────┬────────────────────────┘
       │              │              │
       │     ┌────────▼────────┐     │
       │     │  INFORMATIONAL  │     │
       │     │  Display/Cache  │     │
       │     └────────┬────────┘     │
       │              │              │
┌──────▼──────────────▼──────────────▼────────────────────────┐
│              CRITICAL OPERATIONS                             │
│         Mint │ Burn │ Rebalance                              │
└──────────────────────┬───────────────────────────────────────┘
                      │
        ┌─────────────▼──────────────┐
        │     CRITICAL DATA          │
        │  (No Cache, Validated)     │
        └─────────────┬──────────────┘
                      │
   ┌──────────────────┼──────────────────┐
   │                  │                  │
┌──▼─────────┐ ┌──────▼──────┐ ┌────────▼────────┐
│Token       │ │  Kongswap   │ │  Kong Locker    │
│Canisters   │ │   (DEX)     │ │  (Liquidity)    │
└────────────┘ └─────────────┘ └─────────────────┘
```

---

## Security Zones

### Zone Classification

| Zone | Purpose | Audit Priority | Testing Requirements |
|------|---------|----------------|---------------------|
| **CRITICAL_OPERATIONS** | Token minting, burning, rebalancing | HIGHEST | 100% coverage + fuzzing |
| **CRITICAL_DATA** | Queries for financial calculations | HIGHEST | Property-based tests |
| **TRADING_EXECUTION** | DEX interaction for trades | HIGH | Integration tests |
| **KONG_LIQUIDITY** | External liquidity reference | MEDIUM | Validation tests |
| **INFORMATIONAL** | Display and monitoring | LOW | Basic unit tests |
| **INFRASTRUCTURE** | Types, utilities, constants | MEDIUM | Unit tests |

---

## Module Organization

```yaml
backend_refactored/
├── 1_CRITICAL_OPERATIONS/         # 🔴 HIGHEST SECURITY
│   ├── minting/
│   │   ├── mint_orchestrator.rs   # Two-phase mint coordination
│   │   ├── mint_calculator.rs     # Pure calculation functions
│   │   └── SECURITY.md           # Threat model & invariants
│   │
│   ├── burning/
│   │   ├── burn_orchestrator.rs   # Atomic burn coordination
│   │   ├── redemption_calculator.rs # Pure redemption math
│   │   └── SECURITY.md
│   │
│   └── rebalancing/
│       ├── rebalance_orchestrator.rs # Hourly rebalancing logic
│       ├── trade_calculator.rs    # Trade size calculations
│       └── SECURITY.md
│
├── 2_CRITICAL_DATA/               # 🔴 Financial calculations
│   ├── portfolio_value/
│   │   ├── value_calculator.rs    # TVL for mint ratio (NO CACHE)
│   │   └── balance_aggregator.rs  # Parallel balance queries
│   │
│   ├── supply_tracker/
│   │   ├── supply_queries.rs      # ICPX supply (NO CACHE)
│   │   └── supply_validator.rs    # Supply sanity checks
│   │
│   ├── token_queries/
│   │   ├── balance_queries.rs     # Current holdings
│   │   ├── price_queries.rs       # Token prices from DEX
│   │   └── query_validator.rs     # Price sanity checks
│   │
│   └── validation/
│       ├── trust_boundaries.rs    # External data validation
│       └── snapshot_manager.rs    # Atomic data snapshots
│
├── 3_KONG_LIQUIDITY/              # 🟡 External reference
│   ├── locker_discovery.rs        # Find lock canisters
│   ├── lp_analyzer.rs            # Calculate locked positions
│   ├── target_calculator.rs       # Portfolio targets
│   ├── cache_manager.rs          # Safe caching (1hr TTL)
│   └── README.md
│
├── 4_TRADING_EXECUTION/           # 🟠 Trade execution
│   ├── swap_executor.rs           # Kongswap interaction
│   ├── approval_manager.rs        # ICRC2 approvals
│   ├── slippage_guard.rs         # Trade protection
│   └── SECURITY.md
│
├── 5_INFORMATIONAL/               # 🟢 Display only
│   ├── portfolio_display.rs       # UI state (CACHED)
│   ├── tvl_display.rs            # Locked TVL summary (CACHED)
│   ├── rebalancer_monitor.rs     # Timer status
│   ├── cache_coordinator.rs      # Display cache management
│   └── README.md
│
├── 6_INFRASTRUCTURE/              # ⚪ Shared utilities
│   ├── types.rs                   # Core type definitions
│   ├── constants.rs               # Canister IDs, decimals
│   ├── math_utils.rs             # Safe arithmetic
│   ├── async_utils.rs            # Parallel query helpers
│   └── error_types.rs            # Error definitions
│
└── lib.rs                         # Public API router
```

---

## Detailed Module Pseudocode

### Section 1: Critical Operations - Minting

#### `1_CRITICAL_OPERATIONS/minting/mint_orchestrator.rs`

```yaml
purpose: Coordinate two-phase minting with security guarantees
security: CRITICAL - Creates new ICPX tokens
testing: 100% coverage + property tests + fuzzing

constants:
  MIN_MINT_AMOUNT: 1_000_000 (1 ckUSDT with e6)
  MINT_TIMEOUT: 180 seconds
  FEE_AMOUNT: 100_000 (0.1 ckUSDT)

storage:
  PENDING_MINTS: HashMap<String, PendingMint>

public function initiate_mint(caller: Principal, amount: Nat) -> Result<String>:
  validation:
    require amount >= MIN_MINT_AMOUNT
    require caller != anonymous

  mint_record = PendingMint:
    id: generate_unique_id()
    user: caller
    amount: amount
    status: Pending
    created_at: ic_time_now()
    snapshot: None  # Will be set during complete_mint

  PENDING_MINTS.insert(mint_record.id, mint_record)

  return Ok(mint_record.id)

public function complete_mint(caller: Principal, mint_id: String) -> Result<Nat>:
  # CRITICAL: This function coordinates the mint with strict ordering

  # Step 1: Validate mint request
  mint = PENDING_MINTS.get(mint_id)?
  require mint.user == caller
  require !is_expired(mint.created_at)
  require mint.status == Pending

  # Step 2: Collect fee (point of no return for fee)
  update_status(mint_id, CollectingFee)
  fee_result = critical_data::collect_operation_fee(caller, FEE_AMOUNT)
  if fee_result.is_err():
    update_status(mint_id, FailedFeeCollection)
    return Err("Fee collection failed")

  # Step 3: CRITICAL - Snapshot financial state BEFORE deposit
  # This prevents manipulation attacks
  update_status(mint_id, Snapshotting)

  snapshot = parallel_join!(
    supply = critical_data::supply_tracker::get_validated_supply(),
    tvl = critical_data::portfolio_value::calculate_portfolio_value_atomic()
  ).await?

  # Validate snapshot
  validation::validate_mint_snapshot(&snapshot)?
  mint.snapshot = Some(snapshot)

  # Step 4: Collect deposit
  update_status(mint_id, CollectingDeposit)
  deposit_result = critical_data::collect_user_deposit(
    caller,
    mint.amount,
    generate_mint_memo(mint_id)
  )

  if deposit_result.is_err():
    # Fee not refunded (by design)
    update_status(mint_id, FailedDepositCollection)
    return Err("Deposit collection failed")

  # Step 5: Calculate mint amount (pure function)
  update_status(mint_id, Calculating)

  icpx_amount = mint_calculator::calculate_mint_amount(
    deposit_amount: mint.amount,
    current_supply: snapshot.supply,
    current_tvl: snapshot.tvl
  )?

  # Step 6: Execute mint on ledger
  update_status(mint_id, Minting)

  mint_result = ledger_interaction::mint_icpx_tokens(
    recipient: caller,
    amount: icpx_amount,
    memo: mint_id
  )

  match mint_result:
    Ok(block) =>
      update_status(mint_id, Complete(block))
      cleanup_completed_mint(mint_id)
      return Ok(icpx_amount)

    Err(e) =>
      # Attempt deposit refund
      update_status(mint_id, RefundingDeposit)
      refund_result = critical_data::refund_deposit(caller, mint.amount)

      if refund_result.is_ok():
        update_status(mint_id, FailedWithRefund)
      else:
        update_status(mint_id, FailedNoRefund)
        log_critical_error("Refund failed", mint_id, refund_result.err())

      return Err(format!("Mint failed: {}", e))

private function update_status(mint_id: String, status: MintStatus):
  if let Some(mint) = PENDING_MINTS.get_mut(mint_id):
    mint.status = status
    mint.last_updated = ic_time_now()

private function cleanup_completed_mint(mint_id: String):
  # Keep for 24 hours for debugging
  schedule_removal(mint_id, 24_hours)
```

#### `1_CRITICAL_OPERATIONS/minting/mint_calculator.rs`

```yaml
purpose: Pure functions for mint amount calculation
security: CRITICAL - Determines token issuance
testing: Property-based tests, fuzzing, formal verification

use precision::SafeMath

public function calculate_mint_amount(
  deposit_amount: Nat,
  current_supply: Nat,
  current_tvl: Nat
) -> Result<Nat>:

  # Input validation
  require deposit_amount > 0
  require current_tvl >= 0  # Can be 0 for initial mint
  require current_supply >= 0  # Can be 0 for initial mint

  # Initial mint case: 1:1 ratio (adjusted for decimals)
  if current_supply == 0 || current_tvl == 0:
    # Convert ckUSDT (e6) to ICPX (e8)
    return SafeMath::convert_decimals(
      deposit_amount,
      from_decimals: 6,  # ckUSDT
      to_decimals: 8     # ICPX
    )

  # Subsequent mints: Proportional ownership
  # Formula: new_icpx = (deposit × supply) / tvl
  #
  # This ensures: deposit/tvl = new_icpx/new_supply
  # Meaning: User's ownership % equals their contribution %

  icpx_amount = SafeMath::multiply_and_divide(
    a: deposit_amount,
    b: current_supply,
    c: current_tvl
  )?

  # Sanity check: Mint must be non-zero
  if icpx_amount == 0:
    return Err("Mint amount too small")

  # Sanity check: Reasonable bounds (prevent overflow attacks)
  max_reasonable_mint = current_supply / 10  # Max 10% supply increase
  if icpx_amount > max_reasonable_mint && current_supply > 0:
    return Err("Mint amount exceeds reasonable bounds")

  return Ok(icpx_amount)

# Property tests to verify
invariant proportional_ownership:
  # After minting, user owns exactly their contribution percentage
  let initial_tvl = 10000
  let initial_supply = 1000
  let deposit = 1000

  let new_icpx = calculate_mint_amount(deposit, initial_supply, initial_tvl)
  let new_supply = initial_supply + new_icpx
  let new_tvl = initial_tvl + deposit

  assert (new_icpx / new_supply) == (deposit / new_tvl)

invariant no_dilution_attack:
  # Large deposits cannot unfairly dilute existing holders
  let mint1 = calculate_mint_amount(100, 0, 0)      # First minter
  let mint2 = calculate_mint_amount(100, mint1, 100) # Second minter

  # Both should own 50% after equal deposits
  assert mint1 == mint2
```

### Section 2: Critical Operations - Burning

#### `1_CRITICAL_OPERATIONS/burning/burn_orchestrator.rs`

```yaml
purpose: Coordinate atomic burn and redemption
security: CRITICAL - Returns user funds
testing: 100% coverage + edge cases + partial failure scenarios

constants:
  MIN_BURN_AMOUNT: 11_000 (0.00011 ICPX)
  TRANSFER_FEE_BUFFER: 10_000 (per token)

public function burn_icpx(caller: Principal, burn_amount: Nat) -> Result<BurnResult>:
  # CRITICAL: User must have already transferred ICPX to backend
  # The transfer to backend (minting account) automatically burns the ICPX

  validation:
    require burn_amount >= MIN_BURN_AMOUNT
    require caller != anonymous

  # Step 1: Collect operation fee
  fee_result = critical_data::collect_operation_fee(caller, FEE_AMOUNT)
  if fee_result.is_err():
    return Err("Fee collection required for burn operation")

  # Step 2: Get current state snapshot
  # Supply already reflects the burn (backend is minting account)
  snapshot = parallel_join!(
    supply_after_burn = critical_data::supply_tracker::get_validated_supply(),
    balances = critical_data::token_queries::get_all_token_balances()
  ).await?

  # Step 3: Calculate redemptions (pure function)
  redemptions = redemption_calculator::calculate_proportional_redemption(
    burn_amount: burn_amount,
    supply_after_burn: snapshot.supply_after_burn,
    token_balances: snapshot.balances
  )?

  # Step 4: Execute redemptions
  results = execute_redemptions(caller, redemptions).await

  # Step 5: Build result
  burn_result = BurnResult:
    icpx_burned: burn_amount
    successful_transfers: results.successful
    failed_transfers: results.failed
    timestamp: ic_time_now()

  # Log for audit
  log_burn_operation(caller, burn_amount, &burn_result)

  # Return even with partial failures
  return Ok(burn_result)

private async function execute_redemptions(
  recipient: Principal,
  redemptions: Vec<TokenRedemption>
) -> RedemptionResults:

  successful = Vec::new()
  failed = Vec::new()

  # Execute all redemptions, continuing on failure
  for redemption in redemptions:
    transfer_result = critical_data::transfer_token_to_user(
      token: redemption.token,
      recipient: recipient,
      amount: redemption.amount,
      memo: "ICPX burn redemption"
    ).await

    match transfer_result:
      Ok(block) =>
        successful.push(TransferRecord:
          token: redemption.token
          amount: redemption.amount
          block: block
        )

      Err(e) =>
        failed.push(FailedTransfer:
          token: redemption.token
          amount: redemption.amount
          error: e.to_string()
        )
        # Continue with other transfers

  return RedemptionResults { successful, failed }
```

#### `1_CRITICAL_OPERATIONS/burning/redemption_calculator.rs`

```yaml
purpose: Pure functions for redemption calculation
security: CRITICAL - Determines token returns
testing: Property-based tests with rounding edge cases

use precision::SafeMath

public function calculate_proportional_redemption(
  burn_amount: Nat,
  supply_after_burn: Nat,
  token_balances: HashMap<Token, Nat>
) -> Result<Vec<TokenRedemption>>:

  validation:
    require burn_amount > 0
    require supply_after_burn >= 0  # Can be 0 if burning all

  redemptions = Vec::new()

  # Special case: Burning all remaining supply
  if supply_after_burn == 0:
    # Return all remaining balances
    for (token, balance) in token_balances:
      if balance > TRANSFER_FEE_BUFFER:
        redemptions.push(TokenRedemption:
          token: token
          amount: balance - TRANSFER_FEE_BUFFER
        )
    return Ok(redemptions)

  # Normal case: Proportional redemption
  # Formula: token_return = (burn_amount × token_balance) / supply_after_burn

  for (token, balance) in token_balances:
    if balance == 0:
      continue

    gross_amount = SafeMath::multiply_and_divide(
      a: burn_amount,
      b: balance,
      c: supply_after_burn
    )?

    # Subtract transfer fee and minimum balance
    if gross_amount > TRANSFER_FEE_BUFFER:
      net_amount = gross_amount - TRANSFER_FEE_BUFFER

      redemptions.push(TokenRedemption:
        token: token
        amount: net_amount
      )

  # Validation: At least some redemption
  if redemptions.is_empty():
    return Err("Burn amount too small for any redemption")

  return Ok(redemptions)

invariant proportional_redemption:
  # Burning X% of supply returns X% of each token
  let supply = 1000
  let balances = {
    ALEX: 500,
    ZERO: 300,
    ckUSDT: 200
  }

  let burn_amount = 100  # 10% of supply
  let redemptions = calculate_proportional_redemption(
    burn_amount,
    supply - burn_amount,  # Supply after burn
    balances
  )

  # Should receive ~10% of each token (minus fees)
  assert redemptions[ALEX] ≈ 50
  assert redemptions[ZERO] ≈ 30
  assert redemptions[ckUSDT] ≈ 20
```

### Section 3: Critical Operations - Rebalancing

#### `1_CRITICAL_OPERATIONS/rebalancing/rebalance_orchestrator.rs`

```yaml
purpose: Execute portfolio rebalancing trades
security: CRITICAL - Moves portfolio funds
testing: Integration tests with trade simulation

constants:
  REBALANCE_INTERVAL: 3600 seconds
  MIN_DEVIATION_PCT: 1.0  # 1% threshold
  TRADE_SIZE_PCT: 0.1     # 10% of deviation
  MAX_SLIPPAGE: 0.02      # 2%
  MIN_TRADE_SIZE_USD: 10.0

storage:
  REBALANCE_TIMER: Option<TimerId>
  LAST_REBALANCE: Option<Timestamp>
  REBALANCE_HISTORY: Vec<RebalanceRecord> (max 100)

public function perform_rebalance() -> Result<String>:
  # Check timing
  if !can_rebalance_now():
    return Ok("Too soon since last rebalance")

  # Get current state
  state = get_rebalancing_state().await?

  # Determine action
  action = trade_calculator::determine_rebalance_action(
    current_positions: state.positions,
    target_allocations: state.targets,
    available_ckusdt: state.ckusdt_balance,
    min_deviation: MIN_DEVIATION_PCT,
    min_trade_size: MIN_TRADE_SIZE_USD
  )?

  # Execute action
  result = match action:
    RebalanceAction::Buy(token, usd_amount) =>
      execute_buy_trade(token, usd_amount).await

    RebalanceAction::Sell(token, usd_amount) =>
      execute_sell_trade(token, usd_amount).await

    RebalanceAction::None =>
      Ok("No rebalancing needed")

  # Record result
  record_rebalance(action, result.clone())
  update_last_rebalance()

  return result

private async function execute_buy_trade(
  token: Token,
  usd_amount: Decimal
) -> Result<String>:

  # Convert USD to ckUSDT amount
  ckusdt_amount = (usd_amount * 1_000_000) as Nat

  # Execute swap through trading layer
  swap_result = trading_execution::execute_swap_with_slippage(
    pay_token: ckUSDT,
    pay_amount: ckusdt_amount,
    receive_token: token,
    max_slippage: MAX_SLIPPAGE
  ).await?

  return Ok(format!("Bought {} {} for {} ckUSDT",
    swap_result.amount_received,
    token,
    ckusdt_amount
  ))

private async function execute_sell_trade(
  token: Token,
  usd_amount: Decimal
) -> Result<String>:

  # Get current price to calculate token amount
  price = critical_data::token_queries::get_validated_price(token).await?

  # Calculate token amount needed
  token_amount = trade_calculator::calculate_token_amount_for_usd(
    token: token,
    usd_value: usd_amount,
    price: price
  )?

  # Execute swap
  swap_result = trading_execution::execute_swap_with_slippage(
    pay_token: token,
    pay_amount: token_amount,
    receive_token: ckUSDT,
    max_slippage: MAX_SLIPPAGE
  ).await?

  return Ok(format!("Sold {} {} for {} ckUSDT",
    token_amount,
    token,
    swap_result.amount_received
  ))

private async function get_rebalancing_state() -> Result<RebalanceState>:
  # Parallel fetch all required data
  (positions, targets, ckusdt_balance) = parallel_join!(
    positions = critical_data::token_queries::get_current_positions(),
    targets = kong_liquidity::get_target_allocations(),
    ckusdt = critical_data::token_queries::get_ckusdt_balance()
  ).await?

  return Ok(RebalanceState {
    positions,
    targets,
    ckusdt_balance,
    timestamp: ic_time_now()
  })
```

### Section 4: Critical Data Layer

#### `2_CRITICAL_DATA/portfolio_value/value_calculator.rs`

```yaml
purpose: Calculate portfolio value for mint ratio (NO CACHE)
security: CRITICAL - Directly affects token issuance
testing: Property tests + concurrent query tests

invariants:
  - NEVER cache values used in minting
  - ALWAYS use current prices
  - MUST complete atomically

public async function calculate_portfolio_value_atomic() -> Result<Nat>:
  # CRITICAL: This value determines mint ratios
  # Must be calculated BEFORE collecting deposits

  # Fetch all data in parallel for speed and consistency
  query_futures = Vec::new()

  # ckUSDT balance (already in USD units)
  query_futures.push(
    token_queries::get_token_balance(ckUSDT)
  )

  # All tracked tokens
  for token in TRACKED_TOKENS:
    query_futures.push(async {
      let (balance, price) = parallel_join!(
        balance = token_queries::get_token_balance(token),
        price = token_queries::get_validated_price(token)
      ).await?

      Ok((token, balance, price))
    })

  # Await all queries
  results = join_all(query_futures).await

  # Calculate total value
  total_value_usd = Decimal::ZERO

  for result in results:
    match result:
      Ok((token, balance, price)) =>
        if token == ckUSDT:
          # ckUSDT is already in USD (1:1 peg)
          value = Decimal::from(balance) / Decimal::from(1_000_000)
        else:
          # token_value = (balance / 10^decimals) × price
          decimals = get_token_decimals(token)
          balance_decimal = Decimal::from(balance) / Decimal::from(10_u64.pow(decimals))
          value = balance_decimal * price

        total_value_usd += value

      Err(e) =>
        # CRITICAL: Never guess or use defaults
        return Err(format!("Failed to query {}: {}", token, e))

  # Convert to ckUSDT units (e6)
  total_value_ckusdt = (total_value_usd * Decimal::from(1_000_000)).to_u64()

  # Sanity validation
  validation::validate_portfolio_value(total_value_ckusdt)?

  return Ok(Nat::from(total_value_ckusdt))

public async function calculate_portfolio_value_breakdown() -> Result<PortfolioBreakdown>:
  # Similar to above but returns detailed breakdown
  # Used for display, can be cached
  ...
```

#### `2_CRITICAL_DATA/validation/trust_boundaries.rs`

```yaml
purpose: Validate all external data at trust boundaries
security: CRITICAL - Prevents manipulation attacks
testing: Exhaustive edge case testing

constants:
  MAX_SUPPLY_CHANGE_RATIO: 1.1  # 10% max change
  MAX_PRICE_CHANGE_RATIO: 2.0   # 100% max change
  MIN_REASONABLE_PRICE: 0.0001  # $0.0001
  MAX_REASONABLE_PRICE: 1_000_000.0  # $1M

public function validate_external_supply(
  new_supply: Nat,
  cached_supply: Option<Nat>
) -> Result<Nat>:

  # Check absolute bounds
  if new_supply > MAX_POSSIBLE_SUPPLY:
    return Err("Supply exceeds maximum possible")

  # Check relative change if we have cache
  if let Some(cached) = cached_supply:
    if cached > 0:
      ratio = new_supply as f64 / cached as f64

      if ratio > MAX_SUPPLY_CHANGE_RATIO:
        return Err("Supply increased too rapidly")

      if ratio < (1.0 / MAX_SUPPLY_CHANGE_RATIO):
        return Err("Supply decreased too rapidly")

  return Ok(new_supply)

public function validate_token_price(
  token: Token,
  price: Decimal,
  cached_price: Option<Decimal>
) -> Result<Decimal>:

  # Absolute bounds
  if price < MIN_REASONABLE_PRICE || price > MAX_REASONABLE_PRICE:
    return Err(format!("Price {} outside reasonable bounds", price))

  # Relative change check
  if let Some(cached) = cached_price:
    ratio = price / cached

    if ratio > MAX_PRICE_CHANGE_RATIO:
      # Log warning but allow - could be legitimate
      log_warning("Large price increase detected", token, ratio)

    if ratio < (1.0 / MAX_PRICE_CHANGE_RATIO):
      log_warning("Large price decrease detected", token, ratio)

  return Ok(price)

public function validate_portfolio_value(value: Nat) -> Result<()>:
  # Sanity checks for portfolio value

  if value == 0:
    # Could be legitimate for new canister
    log_info("Portfolio value is zero")

  if value > MAX_REASONABLE_PORTFOLIO_VALUE:
    return Err("Portfolio value unreasonably high")

  return Ok(())

public function validate_mint_snapshot(snapshot: &MintSnapshot) -> Result<()>:
  # Comprehensive validation for mint operation

  # Supply and TVL relationship
  if snapshot.supply > 0 && snapshot.tvl == 0:
    return Err("Invalid state: supply exists but no value")

  # Initial mint
  if snapshot.supply == 0 && snapshot.tvl > 0:
    return Err("Invalid state: value exists but no supply")

  return Ok(())
```

### Section 5: Kong Liquidity (External Reference)

#### `3_KONG_LIQUIDITY/target_calculator.rs`

```yaml
purpose: Calculate portfolio targets from external liquidity
security: IMPORTANT - Affects rebalancing but not minting
caching: ALLOWED - 1 hour TTL acceptable

public async function calculate_target_allocations() -> Result<HashMap<Token, Decimal>>:
  # Get locked liquidity data (can be cached)
  locked_tvl = cache_manager::get_or_calculate(
    key: "locked_tvl",
    ttl: 3600,  # 1 hour cache
    calculator: || calculate_locked_liquidity()
  ).await?

  # Calculate percentages
  total_tvl = locked_tvl.values().sum()

  if total_tvl == 0:
    # No locked liquidity - equal weight default
    return Ok(equal_weight_allocation())

  # Calculate each token's target
  targets = HashMap::new()

  for token in TRACKED_TOKENS:
    token_tvl = locked_tvl.get(token).unwrap_or(0)
    percentage = token_tvl / total_tvl
    targets.insert(token, percentage)

  # Normalize to exactly 100%
  normalize_allocations(&mut targets)

  return Ok(targets)

private async function calculate_locked_liquidity() -> Result<HashMap<Token, Decimal>>:
  # Query kong_locker for lock canisters
  lock_canisters = locker_discovery::get_all_lock_canisters().await?

  # Query LP positions (batched to avoid overwhelming)
  lp_positions = Vec::new()

  for batch in lock_canisters.chunks(10):
    batch_results = parallel_map(batch, |canister|
      lp_analyzer::get_lp_positions(canister)
    ).await

    lp_positions.extend(batch_results)

  # Aggregate by token
  tvl_by_token = HashMap::new()

  for position in lp_positions:
    # LP pools are 50/50 by value
    half_value = position.usd_value / 2.0

    for token in position.tokens:
      if TRACKED_TOKENS.contains(token):
        *tvl_by_token.entry(token).or_insert(0.0) += half_value

  return Ok(tvl_by_token)
```

### Section 6: Informational Queries

#### `5_INFORMATIONAL/portfolio_display.rs`

```yaml
purpose: Cached portfolio state for UI display
security: INFORMATIONAL - Never used in calculations
caching: AGGRESSIVE - 5 minute TTL

cache:
  PORTFOLIO_CACHE: ThreadLocal<RefCell<Option<CachedPortfolio>>>
  CACHE_TTL: 300 seconds

public function get_portfolio_display() -> PortfolioDisplay:
  # Check cache first
  if let Some(cached) = get_cached_portfolio():
    if !cached.is_expired():
      return cached.data

  # Cache miss or expired - recalculate
  portfolio = calculate_display_portfolio().await

  # Update cache
  update_cache(portfolio.clone())

  return portfolio

private async function calculate_display_portfolio() -> PortfolioDisplay:
  # Can use relaxed querying since display only

  # Parallel fetch everything
  (positions, targets, supply, history) = parallel_join!(
    positions = get_current_positions_relaxed(),
    targets = kong_liquidity::get_cached_targets(),
    supply = get_supply_cached(),
    history = get_recent_history()
  ).await

  # Calculate deviations for display
  deviations = calculate_deviations_for_display(positions, targets)

  return PortfolioDisplay {
    total_value_usd: positions.total_value,
    positions: positions.tokens,
    targets: targets,
    deviations: deviations,
    supply: supply,
    last_rebalance: history.last_rebalance,
    timestamp: ic_time_now()
  }

private async function get_current_positions_relaxed() -> Positions:
  # Relaxed version - can fail gracefully

  positions = Vec::new()
  total_value = 0.0

  for token in ALL_TOKENS:  # Include ckUSDT
    try:
      balance = token_queries::get_token_balance(token).await?
      price = get_price_cached_or_fresh(token).await?

      value = calculate_value(balance, price, token)
      total_value += value

      positions.push(Position { token, balance, value, price })
    catch error:
      # Log but continue - this is display only
      log_warning("Failed to get position", token, error)
      # Use last known value if available
      if let Some(last) = get_last_known_position(token):
        positions.push(last)

  return Positions { tokens: positions, total_value }

```

---

## Data Flow Diagrams

### Minting Data Flow

```
User Request
     │
     ▼
[1] initiate_mint(amount)
     │
     ├─→ Create PendingMint record
     └─→ Return mint_id

[2] complete_mint(mint_id)
     │
     ├─→ [2a] Validate mint request
     │
     ├─→ [2b] Collect fee (0.1 ckUSDT)
     │
     ├─→ [2c] CRITICAL: Snapshot state BEFORE deposit
     │      ├─→ Get ICPX supply (validated)
     │      └─→ Calculate portfolio value (atomic)
     │           ├─→ Query all balances (parallel)
     │           └─→ Query all prices (parallel)
     │
     ├─→ [2d] Collect deposit (ckUSDT from user)
     │
     ├─→ [2e] Calculate ICPX amount (pure function)
     │      └─→ (deposit × supply) / tvl
     │
     ├─→ [2f] Mint ICPX via ledger
     │
     └─→ [2g] Return ICPX amount or handle failure
            └─→ Refund deposit on failure (not fee)
```

### Burning Data Flow

```
Prerequisites: User transfers ICPX to backend
                    │
                    ▼
              [Automatic burn via minting account]
                    │
                    ▼
User calls burn_icpx(amount)
     │
     ├─→ [1] Collect fee (0.1 ckUSDT)
     │
     ├─→ [2] Get current state (supply already reflects burn)
     │      ├─→ Query ICPX supply
     │      └─→ Query all token balances
     │
     ├─→ [3] Calculate redemptions (pure function)
     │      └─→ For each token: (burn × balance) / supply
     │
     ├─→ [4] Execute transfers (continue on failure)
     │      ├─→ Transfer ALEX to user
     │      ├─→ Transfer ZERO to user
     │      ├─→ Transfer KONG to user
     │      ├─→ Transfer BOB to user
     │      └─→ Transfer ckUSDT to user
     │
     └─→ [5] Return BurnResult with successes and failures
```

---

## Security Improvements

### 5.1 Clear Audit Boundaries

**BEFORE:**
- `tvl_calculator.rs` mixed critical (`calculate_tvl_in_ckusdt`) and informational (`get_tvl_summary`) functions
- Hard to identify what needs careful review

**AFTER:**
- `2_CRITICAL_DATA/portfolio_value/` - Audit carefully, used in minting
- `5_INFORMATIONAL/tvl_display/` - Lower priority, display only
- Clear separation makes audits focused and efficient

### 5.2 Validation at Trust Boundaries

**NEW:** All external data validated before use:
```yaml
External Data → Validation → Trusted Use

Examples:
- ICPX supply → Range check → Mint calculation
- Token price → Sanity check → Rebalancing
- LP positions → Format check → Target calculation
```

### 5.3 Atomic Operations

**IMPROVED:** Critical operations now atomic:
- Mint snapshot taken before deposit (prevents manipulation)
- Burn calculations use post-burn supply (no race condition)
- Parallel queries for consistency

### 5.4 Pure Calculation Functions

**NEW:** Financial math isolated in pure functions:
- `mint_calculator::calculate_mint_amount` - No I/O, fully testable
- `redemption_calculator::calculate_proportional_redemption` - Pure math
- `trade_calculator::determine_rebalance_action` - Deterministic logic

---

## Invariants Documentation

### System Invariants

```yaml
invariant_1:
  statement: "Total ICPX = Sum of all user balances"
  enforcement: ICPX ledger canister
  verification: Periodic audit query

invariant_2:
  statement: "Minting account balance = 0"
  enforcement: Backend is minting account, burns on receipt
  verification: Query after each burn

invariant_3:
  statement: "New ICPX = (deposit × supply_before) / tvl_before"
  enforcement: mint_calculator.rs with snapshot timing
  verification: Property-based tests

invariant_4:
  statement: "Redemption = (burn × balance) / supply_after"
  enforcement: redemption_calculator.rs
  verification: Property tests with edge cases

invariant_5:
  statement: "One trade per hour maximum"
  enforcement: rebalance_orchestrator.rs timing check
  verification: Integration tests

invariant_6:
  statement: "TVL snapshot before deposit collection"
  enforcement: mint_orchestrator.rs step ordering
  verification: Code review + tests
```

---

## Migration Strategy

### Phase 1: Infrastructure (Week 1) - LOW RISK
```yaml
tasks:
  - Create new directory structure
  - Move types.rs → 6_INFRASTRUCTURE/
  - Move precision.rs → 6_INFRASTRUCTURE/math_utils.rs
  - Update imports

validation: Compilation succeeds, all tests pass
```

### Phase 2: Informational Modules (Week 2) - LOW RISK
```yaml
tasks:
  - Extract display functions → 5_INFORMATIONAL/
  - Implement cache_coordinator.rs
  - Update frontend calls

validation: UI displays correctly
```

### Phase 3: Kong Liquidity (Week 3) - MEDIUM RISK
```yaml
tasks:
  - Move kong_locker.rs → 3_KONG_LIQUIDITY/
  - Separate target calculation from TVL
  - Add cache management

validation: Rebalancing targets unchanged
```

### Phase 4: Critical Data (Week 4) - HIGH RISK
```yaml
tasks:
  - Create 2_CRITICAL_DATA/ structure
  - Add validation layer
  - Separate portfolio value from display

validation: Extensive testing on testnet
```

### Phase 5: Critical Operations (Weeks 5-6) - HIGHEST RISK
```yaml
tasks:
  - Refactor minting.rs → 1_CRITICAL_OPERATIONS/minting/
  - Refactor burning.rs → 1_CRITICAL_OPERATIONS/burning/
  - Add comprehensive logging

validation:
  - All integration tests pass
  - Security audit
  - Staged rollout with monitoring
```

---

## Testing Requirements

### Critical Modules (100% Coverage Required)

```yaml
1_CRITICAL_OPERATIONS/:
  unit_tests:
    - All calculation functions
    - State transitions
    - Error paths

  property_tests:
    - Proportional ownership maintained
    - No token creation without deposit
    - Redemption fairness

  integration_tests:
    - Full mint flow
    - Full burn flow
    - Rebalancing execution

  fuzzing:
    - Calculator functions with extreme values
    - Concurrent operations
```

### Informational Modules (Standard Coverage)

```yaml
5_INFORMATIONAL/:
  unit_tests:
    - Cache hit/miss
    - Display formatting

  integration_tests:
    - UI queries work
    - Graceful degradation
```

---

## Open Questions

### Q1: Cache Policy for Prices
**Context:** Price queries could use 1-minute cache for display
**Trade-off:** Performance vs. freshness
**Recommendation:** Live for critical, cache for display
**Decision Needed:** Confirm acceptable staleness

### Q2: Parallel Query Limits
**Context:** How many parallel queries can canisters handle?
**Trade-off:** Speed vs. stability
**Recommendation:** Batch in groups of 10
**Decision Needed:** Test and determine optimal batch size

### Q3: Minimum Trade Sizes
**Context:** Very small trades may not be worth gas fees
**Trade-off:** Precision vs. cost
**Recommendation:** $10 minimum trade size
**Decision Needed:** Confirm threshold

---

## Appendices

### A. Migration Checklist

- [ ] Create new directory structure
- [ ] Move infrastructure modules
- [ ] Extract informational functions
- [ ] Separate Kong liquidity logic
- [ ] Isolate critical data layer
- [ ] Refactor critical operations
- [ ] Update all imports
- [ ] Run full test suite
- [ ] Security audit
- [ ] Deploy to testnet
- [ ] Monitor for 48 hours
- [ ] Deploy to mainnet

### B. Security Checklist

- [ ] All external data validated
- [ ] Critical operations atomic
- [ ] Pure functions for calculations
- [ ] No caching in critical paths
- [ ] Comprehensive error handling
- [ ] Audit trail for operations
- [ ] Property tests for invariants
- [ ] Fuzzing for edge cases

### C. File Mapping Reference

```yaml
OLD → NEW

minting.rs → 1_CRITICAL_OPERATIONS/minting/
burning.rs → 1_CRITICAL_OPERATIONS/burning/
rebalancer.rs → 1_CRITICAL_OPERATIONS/rebalancing/

tvl_calculator.rs (calculate_tvl_in_ckusdt) → 2_CRITICAL_DATA/portfolio_value/
tvl_calculator.rs (get_tvl_summary) → 5_INFORMATIONAL/tvl_display/

ledger_client.rs → 2_CRITICAL_DATA/supply_tracker/
balance_tracker.rs → 2_CRITICAL_DATA/token_queries/

kong_locker.rs → 3_KONG_LIQUIDITY/locker_discovery/
kongswap.rs (LP queries) → 3_KONG_LIQUIDITY/lp_analyzer/
kongswap.rs (swaps) → 4_TRADING_EXECUTION/swap_executor/

index_state.rs (calculations) → Various CRITICAL modules
index_state.rs (display) → 5_INFORMATIONAL/portfolio_display/

types.rs → 6_INFRASTRUCTURE/types.rs
precision.rs → 6_INFRASTRUCTURE/math_utils.rs
```

---

## References

- Original Pseudocode: [ICPX_BACKEND_PSEUDOCODE.md](./ICPX_BACKEND_PSEUDOCODE.md)
- Security Best Practices: OWASP Smart Contract Top 10
- Internet Computer Security Guidelines

---

**End of Document**

Generated: 2025-10-06
Version: 2.0 - Security Refactoring
Purpose: Security-first architecture for ICPX backend