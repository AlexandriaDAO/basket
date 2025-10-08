# ICPI Backend Architecture - Pseudocode Reference

**Purpose**: Internet Computer Portfolio Index backend canister business logic
**Date Generated**: 2025-10-02
**For**: Offline review and code understanding

---

## File Tree Structure

```
icpi_backend/
├── Cargo.toml                    # Rust dependencies
├── icpi_backend.did              # Candid interface definition
└── src/
    ├── lib.rs                    # Main entry point, public API endpoints
    ├── types.rs                  # Core type definitions and constants
    ├── icrc_types.rs             # ICRC token standard types
    ├── precision.rs              # Safe math operations
    ├── icpi_math.rs              # Mint/burn calculation logic
    ├── kong_locker.rs            # Query locked liquidity canisters
    ├── kongswap.rs               # Interact with Kongswap DEX
    ├── icpi_token.rs             # ICPI token interface (ICRC-1)
    ├── balance_tracker.rs        # Query token balances
    ├── index_state.rs            # Calculate portfolio state
    ├── tvl_calculator.rs         # Calculate total value locked
    ├── minting.rs                # Mint new ICPI tokens
    ├── burning.rs                # Burn ICPI and redeem tokens
    ├── ledger_client.rs          # Interact with ICPI ledger
    └── rebalancer.rs             # Automated rebalancing timer
```

---

## File Pseudocode Breakdown

### 1. `Cargo.toml` - Dependencies

```yaml
package:
  name: icpi_backend
  version: 0.1.0

library_type: cdylib  # WebAssembly canister

dependencies:
  - candid: Interface definition language
  - ic-cdk: Internet Computer development kit
  - ic-cdk-timers: Scheduled task execution
  - rust_decimal: High precision decimal math
  - num-bigint: Large integer arithmetic
  - futures: Async parallel operations
  - serde: Data serialization
```

---

### 2. `lib.rs` - Main Entry Point

```yaml
purpose: Public API endpoints and canister lifecycle

constants:
  INDEX_STATE_CACHE: thread-local cache for performance
  CACHE_REFRESH_INTERVAL: 5 minutes

lifecycle_hooks:
  init:
    - Start hourly rebalancing timer
    - Start mint cleanup timer
    - Start cache refresh timer

  post_upgrade:
    - Restart all timers
    - Cache survives upgrades (in-memory)

public_endpoints:

  # ICRC-1 Token Interface (proxied to real ledger)
  icrc1_name: "Internet Computer Portfolio Index"
  icrc1_symbol: "ICPI"
  icrc1_decimals: 8
  icrc1_total_supply: queries real ledger canister
  icrc1_balance_of: queries real ledger canister

  # Minting Flow
  initiate_mint:
    input: ckUSDT amount
    output: mint_id (tracking string)
    action: Create pending mint operation

  complete_mint:
    input: mint_id
    output: ICPI amount minted
    steps:
      - Collect 0.1 ckUSDT fee
      - Calculate supply/TVL BEFORE deposit
      - Collect ckUSDT deposit
      - Calculate proportional ICPI amount
      - Mint ICPI to user via ledger
      - Refund on any failure

  check_mint_status:
    input: mint_id
    output: current status of mint

  # Burning Flow
  burn_icpi:
    input: ICPI amount to burn
    output: tokens returned to user
    steps:
      - User transfers ICPI to backend (burns it)
      - Collect 0.1 ckUSDT fee
      - Calculate proportional redemption
      - Transfer all portfolio tokens to user

  # Index State
  get_index_state:
    output: complete portfolio state
    includes:
      - Current positions with USD values
      - Target allocations from locked TVL
      - Deviations requiring rebalancing
      - Available ckUSDT balance

  get_index_state_cached:
    output: cached state (fast, query)
    refresh_interval: 5 minutes

  get_tvl_summary:
    output: locked liquidity breakdown by token

  # Rebalancer Controls
  trigger_manual_rebalance:
    action: Force immediate rebalance check

  get_rebalancer_status:
    output:
      - Timer active status
      - Last rebalance timestamp
      - Recent history
      - Next scheduled rebalance

  debug_rebalancer:
    output: detailed rebalancing logic trace

  # Testing
  test_kong_locker_connection: verify external canister
  test_kongswap_connection: verify DEX connection
  test_balance_query: verify token balance query
  clear_all_caches: reset cached data
```

---

### 3. `types.rs` - Core Type Definitions

```yaml
purpose: Shared types, constants, and helper functions

tracked_tokens:
  ALEX: ysy5f-2qaaa-aaaap-qkmmq-cai (8 decimals)
  ZERO: b3d2q-ayaaa-aaaap-qqcfq-cai (8 decimals)
  KONG: o7oak-iyaaa-aaaaq-aadzq-cai (8 decimals)
  BOB: 7pail-xaaaa-aaaas-aabmq-cai (8 decimals)
  ckUSDT: cngnf-vqaaa-aaaar-qag4q-cai (6 decimals)

external_canisters:
  kong_locker: eazgb-giaaa-aaaap-qqc2q-cai
  kongswap_backend: 2ipq2-uqaaa-aaaar-qailq-cai
  icpi_ledger: l6lep-niaaa-aaaap-qqeda-cai

data_structures:

  CurrentPosition:
    token: which token
    balance: raw amount with decimals
    usd_value: dollar value
    percentage: % of total portfolio

  TargetAllocation:
    token: which token
    target_percentage: from locked liquidity ratio
    target_usd_value: dollar amount

  AllocationDeviation:
    token: which token
    current_pct: current allocation %
    target_pct: desired allocation %
    deviation_pct: how far off (target - current)
    usd_difference: dollar amount difference
    trade_size_usd: 10% of difference for hourly trade

  IndexState:
    total_value: portfolio value in USD
    current_positions: actual holdings
    target_allocations: desired holdings
    deviations: what needs rebalancing
    timestamp: when calculated
    ckusdt_balance: available for rebalancing

  RebalanceAction:
    Buy: purchase token with ckUSDT
    Sell: sell token for ckUSDT
    None: no action needed

helper_functions:
  validate_principal: check if canister is whitelisted
  decimal_to_f64: safe conversion for display
  f64_to_decimal: safe conversion for math
```

---

### 4. `icrc_types.rs` - ICRC Token Standards

```yaml
purpose: ICRC-1 and ICRC-2 token interaction helpers

constants:
  FEE_AMOUNT: 100,000 (0.1 ckUSDT with 6 decimals)
  FEE_RECIPIENT: e454q-riaaa-aaaap-qqcyq-cai
  CKUSDT_DECIMALS: 6
  ICPI_DECIMALS: 8

functions:

  collect_fee:
    input: user principal
    output: block index or error
    action: Transfer 0.1 ckUSDT from user to fee recipient
    method: icrc2_transfer_from (requires prior approval)

  collect_deposit:
    input: user, amount, memo
    output: block index or error
    action: Transfer ckUSDT from user to backend
    method: icrc2_transfer_from (requires prior approval)
    note: Memo truncated to 32 bytes for ALEX compatibility

  transfer_to_user:
    input: token canister, user, amount, memo
    output: block index or error
    action: Send tokens from backend to user
    method: icrc1_transfer
    use_case: Refunds and redemptions

  query_icrc1_balance:
    input: token canister, account owner
    output: balance amount
    method: icrc1_balance_of

  approve_kongswap_spending:
    input: token canister, amount
    output: approval block index
    action: Allow Kongswap to spend tokens
    method: icrc2_approve
    note: Includes gas fee in approval amount
```

---

### 5. `precision.rs` - Safe Math Operations

```yaml
purpose: Prevent overflow and precision loss in token calculations

constants:
  PRECISION_FACTOR: 1,000,000,000,000 (10^12 for intermediate calcs)

functions:

  multiply_and_divide:
    formula: (a × b) ÷ c
    implementation: Uses BigUint for arbitrary precision
    safety: Returns error on division by zero
    example: (deposit × supply) ÷ TVL = ICPI to mint

  calculate_proportional_amount:
    formula: (amount × numerator) ÷ denominator
    use_case: Calculate share of portfolio
    implementation: Uses precision factor to avoid rounding
    example: (burn_amount × token_balance) ÷ total_supply

  convert_decimals:
    input: amount, from_decimals, to_decimals
    output: converted amount
    example: ckUSDT (e6) to ICPI (e8) = multiply by 100
    example: ICPI (e8) to ckUSDT (e6) = divide by 100

internal_helpers:
  nat_to_biguint: Convert Candid Nat to BigUint
  biguint_to_nat: Convert BigUint to Candid Nat
```

---

### 6. `icpi_math.rs` - Mint/Burn Logic (Legacy)

```yaml
purpose: Original mint/burn calculations (mostly moved to minting.rs)

constants:
  DECIMALS: 8
  SCALING_FACTOR: 100,000,000 (10^8)
  MIN_MINT_INCREMENT: 1000 basis points (0.01% minimum)

mint_formula:
  initial_mint: 1 ICPI = 1 USDT
  subsequent_mints: new_icpi = (usdt × supply) ÷ tvl

  validation:
    - Minimum 0.1% of current TVL
    - Ensures no dust mints

  token_purchases:
    - Calculate each token's target value
    - Purchase difference proportionally

burn_formula:
  token_return = (icpi_burned × token_balance) ÷ total_supply

  example:
    - Burn 10% of ICPI supply
    - Receive 10% of each token held

note: This module contains test cases and validation logic
```

---

### 7. `kong_locker.rs` - Locked Liquidity Queries

```yaml
purpose: Fetch list of lock canisters from kong_locker

caching:
  cache_location: thread-local storage
  ttl: 3600 seconds (1 hour)
  reason: Reduce inter-canister calls

functions:

  get_all_lock_canisters:
    output: list of (user, lock_canister) pairs
    flow:
      - Check cache first
      - If expired, query kong_locker
      - Update cache with results
      - Return canister list

  get_lock_canisters_with_retry:
    input: max_retries
    output: lock canisters or error
    flow:
      - Try get_all_lock_canisters
      - On failure, exponential backoff
      - Retry up to max_retries times

  clear_lock_canisters_cache:
    action: Force refresh on next query

  get_cache_stats:
    output: (is_cached, last_updated, count)
    use: Debugging cache state
```

---

### 8. `kongswap.rs` - DEX Integration

```yaml
purpose: Interact with Kongswap for trading and pricing

functions:

  get_lp_positions:
    input: lock canister principal
    output: list of LP positions
    method: user_balances on Kongswap
    use: Calculate locked TVL

  get_token_price_in_usdt:
    input: token
    output: price in USDT (Decimal)
    method: swap_amounts for 1 token
    fallback: Try reverse direction if direct pair missing
    formula: If reverse, price = 1 ÷ reverse_price

  execute_swap:
    input: pay_symbol, pay_amount, receive_symbol, max_slippage
    output: swap reply with details
    steps:
      1. Get quote to verify viable
      2. Check slippage threshold (default 2%)
      3. Approve Kongswap spending (if not ICP)
      4. Execute swap via ICRC2 flow
      5. Verify status is "SUCCESS"
    note: One trade at a time (sequential)

  batch_query_lp_positions:
    input: list of lock canisters, batch_size
    output: list of LP position results
    note: Sequential processing to avoid overwhelming Kongswap

  get_all_token_prices:
    output: prices for ALEX, ZERO, KONG, BOB
    note: Sequential queries (Kongswap requirement)
```

---

### 9. `icpi_token.rs` - Token Interface

```yaml
purpose: ICRC-1 standard methods (proxy to real ledger)

token_metadata:
  name: "Internet Computer Portfolio Index"
  symbol: "ICPI"
  decimals: 8
  fee: 0
  ledger_canister: l6lep-niaaa-aaaap-qqeda-cai

endpoints:

  icrc1_name: query, returns token name
  icrc1_symbol: query, returns "ICPI"
  icrc1_decimals: query, returns 8
  icrc1_fee: query, returns 0

  icrc1_total_supply:
    type: update (inter-canister call)
    action: Query real ledger for total supply
    note: Backend doesn't store supply

  icrc1_balance_of:
    type: update (inter-canister call)
    input: account
    action: Query real ledger for balance
    note: Backend doesn't store balances

  icrc1_metadata:
    type: query
    output: list of metadata key-value pairs

  icrc1_supported_standards:
    type: query
    output: ["ICRC-1"]

  get_all_balances:
    type: query (deprecated)
    note: Returns empty - query ledger directly

critical_note: Backend is minting account, not storage
```

---

### 10. `balance_tracker.rs` - Token Balance Queries

```yaml
purpose: Query current token holdings for portfolio

functions:

  get_token_balance:
    input: tracked token
    output: balance (Nat)
    method: icrc1_balance_of on token canister
    account: backend canister's account

  get_ckusdt_balance:
    output: ckUSDT balance (Nat)
    special: ckUSDT has 6 decimals (not 8)

  get_current_positions:
    output: list of current positions
    steps:
      1. Launch balance queries in parallel (all tokens)
      2. Launch price queries in parallel (all tokens)
      3. Await all results concurrently
      4. Calculate USD values: (balance × price) ÷ 10^decimals
      5. Include ckUSDT (price = 1.0)
      6. Calculate percentages of total

    performance: Parallel queries via futures::join_all

  transfer_token:
    input: token, recipient, amount
    output: block index
    method: icrc1_transfer
    use: Redemptions and rebalancing

  get_balance_summary:
    output: BalanceSummary structure
    includes:
      - total_value_usd
      - positions with balances
      - ckusdt_balance
      - timestamp

  get_all_balances:
    output: list of (token, balance) pairs
    use: Debugging and monitoring
```

---

### 11. `index_state.rs` - Portfolio State Calculator

```yaml
purpose: Calculate current state and rebalancing needs

functions:

  calculate_target_allocations:
    input: ICPI total value
    output: target allocations for each token
    steps:
      1. Get locked TVL from kong_locker
      2. Calculate percentage each token represents
      3. Apply percentages to ICPI total value
      4. Normalize to sum exactly 100%

    example:
      - ALEX locked TVL = 50% of total
      - ICPI value = $1000
      - ALEX target = $500

  calculate_deviations:
    input: current positions, target allocations
    output: deviation list sorted by urgency
    calculation:
      - deviation_pct = target_pct - current_pct
      - usd_difference = target_value - current_value
      - trade_size_usd = abs(usd_difference) × 0.1

    sorting: By absolute deviation (largest first)

  get_index_state:
    output: complete IndexState structure
    steps:
      1. Get current positions (parallel queries)
      2. Calculate total value
      3. Get target allocations from locked TVL
      4. Calculate deviations
      5. Get ckUSDT balance
      6. Return complete state

  get_rebalancing_action:
    input: deviations, ckusdt_balance
    output: RebalanceAction or None
    logic:

      if ckusdt_balance >= $10:
        find most underweight token (deviation > 1%)
        return Buy action

      else:
        find most overweight token (deviation < -1%)
        return Sell action

      if no tokens exceed 1% threshold:
        return None

    trade_size: 10% of deviation per hour

  get_rebalancing_recommendation:
    output: recommended action
    flow:
      - Get index state
      - Analyze deviations
      - Return action or None

validation:
  validate_index_state:
    - Current percentages sum to ~100%
    - Target percentages sum to ~100%
    - All tracked tokens present

formatting:
  format_index_state:
    output: human-readable string
    includes: positions, targets, deviations
```

---

### 12. `tvl_calculator.rs` - Total Value Locked

```yaml
purpose: Calculate locked liquidity TVL and canister TVL

caching:
  cache_location: thread-local
  ttl: 300 seconds (5 minutes)

functions:

  calculate_locked_tvl:
    output: HashMap of (token, USD value)
    steps:
      1. Get all lock canisters from kong_locker
      2. Query LP positions in parallel
      3. For each LP position:
         - Pool value is 50/50 split
         - Attribute half to each token
         - Sum up for each tracked token
      4. Return totals by token

    example:
      - ALEX/ckUSDT pool = $1000
      - ALEX TVL += $500
      - ckUSDT TVL += $500 (ignored if not tracked)

  calculate_tvl_percentages:
    input: TVL by token
    output: percentage each represents
    formula: (token_tvl ÷ total_tvl) × 100

  get_tvl_summary:
    output: formatted TVL summary
    includes:
      - total_tvl_usd
      - list of tokens with tvl and percentage
      - timestamp
    sorting: By TVL descending

  get_cached_or_calculate_tvl:
    output: TVL by token (from cache or fresh)
    flow:
      - Check cache
      - If expired, recalculate
      - Update cache
      - Return result

  calculate_tvl_in_ckusdt:
    output: total canister holdings in ckUSDT units
    purpose: For mint calculations
    steps:
      1. Get ckUSDT balance directly
      2. For each tracked token in parallel:
         - Get balance
         - Get price in USDT
         - Calculate value = (balance ÷ 10^decimals) × price
      3. Sum all values
      4. Convert to ckUSDT e6 format

    note: Different from locked TVL (this is canister holdings)
```

---

### 13. `minting.rs` - ICPI Token Minting

```yaml
purpose: Two-phase secure minting with refund protection

constants:
  TIMEOUT_NANOS: 180,000,000,000 (3 minutes)

storage:
  PENDING_MINTS: thread-local HashMap of mint operations

mint_status_states:
  - Pending: mint created, awaiting user action
  - CollectingFee: charging 0.1 ckUSDT fee
  - CollectingDeposit: receiving ckUSDT from user
  - Calculating: determining ICPI amount
  - Refunding: returning deposit after failure
  - Minting: creating ICPI tokens
  - Complete: finished successfully
  - Failed: generic failure
  - FailedRefunded: failed but deposit returned
  - FailedNoRefund: failed and refund also failed
  - Expired: timeout exceeded

functions:

  initiate_mint:
    input: ckusdt_amount
    output: mint_id (tracking string)
    validation: minimum 1 ckUSDT
    steps:
      - Generate unique mint_id
      - Create PendingMint record
      - Store in PENDING_MINTS map
      - Return mint_id to user

    note: User must approve backend for ckUSDT spending

  complete_mint:
    input: mint_id
    output: ICPI amount minted
    steps:

      1. Verify mint exists and ownership
      2. Check not expired

      3. Collect fee:
         - Transfer 0.1 ckUSDT to fee recipient
         - Fail if insufficient approval

      4. CRITICAL: Calculate BEFORE collecting deposit
         - Get current ICPI supply
         - Get current canister TVL
         - This ensures fair pricing

      5. Collect deposit:
         - Transfer ckUSDT from user to backend
         - Fail if insufficient funds

      6. Calculate ICPI to mint:
         - If initial: 1:1 ratio (adjusted for decimals)
         - Otherwise: (deposit × supply) ÷ tvl

      7. Mint ICPI via ledger:
         - Call icrc1_transfer on ICPI ledger
         - Backend is minting authority

      8. Mark complete

      error_handling:
        - Fee collected but not refunded on failure
        - Deposit refunded on any post-fee failure
        - Status tracks refund attempts

  check_mint_status:
    input: mint_id
    output: current MintStatus
    type: query (no state changes)

  cleanup_expired_mints:
    trigger: every 5 minutes
    action:
      - Mark expired pending mints
      - Remove old completed/failed mints (24hr+)

  start_cleanup_timer:
    action: Start 5-minute interval timer

security_features:
  - Two-phase prevents race conditions
  - TVL snapshot before deposit
  - Automatic refunds on failure
  - Timeout protection
  - Status tracking for debugging
```

---

### 14. `burning.rs` - ICPI Token Burning

```yaml
purpose: Atomic burn operation (single-call, secure)

constants:
  MIN_BURN_AMOUNT: 11,000 units (0.00011 ICPI)
  CKUSDT_TRANSFER_FEE: 10,000 (ckUSDT)
  ICRC1_TRANSFER_FEE: 10,000 (standard tokens)

function:

  burn_icpi:
    input: amount to burn
    output: BurnResult with transfer details

    prerequisites:
      - User must first transfer ICPI to backend
      - Transfer to backend = automatic burn (backend is minting account)

    steps:

      1. Validate minimum amount

      2. Collect 0.1 ckUSDT fee

      3. Get current supply atomically:
         - No race condition
         - Supply reflects user's burn already

      4. Calculate proportional redemption:

         for ckUSDT:
           amount = (burn_amount × ckusdt_balance) ÷ supply
           if amount > fee + buffer:
             redemptions.add(ckUSDT, amount - fee)

         for each tracked token:
           amount = (burn_amount × token_balance) ÷ supply
           if amount > fee + buffer:
             redemptions.add(token, amount - fee)

      5. Execute transfers to user:
         - Transfer each token back to user
         - Track successful and failed transfers
         - Continue even if some fail

      6. Return BurnResult:
         - successful_transfers: list
         - failed_transfers: list with errors
         - icpi_burned: amount

security_features:
  - Single atomic operation
  - No two-phase vulnerability
  - User burns before calling
  - Proportional to exact current supply
  - Continues on partial failure

burn_result_structure:
  successful_transfers:
    - (token_symbol, amount)
  failed_transfers:
    - (token_symbol, amount, error_message)
  icpi_burned: total burned
```

---

### 15. `ledger_client.rs` - ICPI Ledger Interaction

```yaml
purpose: Interact with the real ICPI token ledger canister

constants:
  ICPI_LEDGER_CANISTER: l6lep-niaaa-aaaap-qqeda-cai

functions:

  mint_icpi_tokens:
    input: recipient, amount
    output: block index or error
    method: icrc1_transfer on ledger
    authority: Backend canister is minting account
    flow:
      - Create transfer from minting account to user
      - Minting account = backend canister
      - Call ledger's icrc1_transfer
      - Return block index

  get_icpi_balance:
    input: owner principal
    output: balance (Nat)
    method: icrc1_balance_of on ledger
    use: Query user balances

  get_icpi_total_supply:
    output: total supply (Nat)
    method: icrc1_total_supply on ledger
    use: Mint and burn calculations

critical_note:
  - Backend doesn't store balances
  - Ledger is source of truth
  - Backend only has minting authority
  - All balance queries go to ledger
```

---

### 16. `rebalancer.rs` - Automated Rebalancing

```yaml
purpose: Hourly timer to maintain portfolio allocations

constants:
  REBALANCE_INTERVAL: 3600 seconds (1 hour)
  MIN_TIME_BETWEEN: 3550 seconds (allow 50s drift)

storage:
  REBALANCE_TIMER: thread-local timer ID
  LAST_REBALANCE: thread-local timestamp
  REBALANCE_HISTORY: thread-local list of records (max 100)

functions:

  start_rebalancing:
    action:
      - Create 1-hour interval timer
      - Store timer ID
      - Each tick calls perform_rebalance

  stop_rebalancing:
    action:
      - Clear timer
      - Stop automatic rebalancing

  perform_rebalance:
    output: success message or error
    steps:

      1. Check time since last rebalance
         - Must be at least ~1 hour

      2. Get rebalancing recommendation
         - Query index state
         - Analyze deviations

      3. Execute action:

         if Buy action:
           - Convert USD to ckUSDT amount
           - Execute swap: ckUSDT → token
           - Max 2% slippage

         if Sell action:
           - Calculate token amount needed
           - Execute swap: token → ckUSDT
           - Max 2% slippage

         if None:
           - Log "no action needed"

      4. Record in history

      5. Update last rebalance timestamp

  execute_buy:
    input: token, usdt_amount
    output: swap details
    steps:
      - Convert USD to ckUSDT (6 decimals)
      - Call kongswap execute_swap
      - Return result

  execute_sell:
    input: token, usdt_value
    output: swap details
    steps:
      - Get current token price
      - Calculate token amount for USD value
      - Call kongswap execute_swap
      - Return result

  calculate_token_amount_for_usd:
    input: token, usd_value
    output: token amount (Nat)
    formula: (usd_value ÷ price) × 10^decimals

  trigger_manual_rebalance:
    action:
      - Clear last rebalance time
      - Force immediate rebalance
      - Use: Testing and emergencies

  get_rebalancer_status:
    output: RebalancerStatus
    includes:
      - timer_active: boolean
      - last_rebalance: timestamp
      - next_rebalance: calculated time
      - recent_history: last 10 records

rebalance_record:
  timestamp: when it occurred
  action: Buy/Sell/None details
  success: boolean
  details: human-readable message

history_management:
  - Keep last 100 records
  - Older records automatically removed
  - Available via get_rebalancer_status
```

---

### 17. `icpi_backend.did` - Candid Interface

```yaml
purpose: Public API definition for frontend and CLI tools

service_interface:

  # ICRC-1 Standard
  icrc1_name: query → text
  icrc1_symbol: query → text
  icrc1_decimals: query → nat8
  icrc1_total_supply: update → nat
  icrc1_balance_of: update → nat
  icrc1_fee: query → nat
  icrc1_metadata: query → vec(text, MetadataValue)
  icrc1_supported_standards: query → vec(StandardRecord)

  # Minting
  initiate_mint: update → Result(text, text)
  complete_mint: update → Result(nat, text)
  check_mint_status: query → Result(MintStatus, text)

  # Burning
  burn_icpi: update → Result(BurnResult, text)

  # Index State
  get_index_state: update → Result(IndexState, text)
  get_index_state_cached: query → Result(IndexState, text)
  get_tvl_summary: update → Result(TVLSummary, text)
  get_token_metadata: query → Result(vec(TokenMetadata), text)

  # Rebalancer
  get_rebalancer_status: query → RebalancerStatus
  debug_rebalancer: update → text

  # System
  get_canister_id: query → principal
  get_cycles_balance: query → nat

type_definitions:
  - Account: {owner, subaccount}
  - TrackedToken: enum of ALEX/ZERO/KONG/BOB/ckUSDT
  - CurrentPosition: token, balance, usd_value, percentage
  - TargetAllocation: token, target_percentage, target_usd_value
  - AllocationDeviation: detailed deviation info
  - IndexState: complete portfolio state
  - RebalanceAction: Buy/Sell/None
  - MintStatus: state machine for minting
  - BurnResult: transfer results
  - TVLSummary: locked liquidity breakdown

note: All update methods can make inter-canister calls
note: Query methods are fast but can't make external calls
```

---

## Key Architecture Principles

### 1. No Persistent Storage for Balances
```yaml
reason: Balances queried live from token canisters
benefit: Always accurate, no sync issues
trade_off: Slower but safer
```

### 2. Caching for Performance
```yaml
cached_data:
  - Lock canisters (1 hour TTL)
  - TVL calculations (5 minutes TTL)
  - Index state (5 minutes TTL)

refresh:
  - Automatic background timers
  - Manual clear_cache functions
  - TTL-based expiration
```

### 3. Parallel Queries
```yaml
use: futures::join_all
examples:
  - All token balances in parallel
  - All token prices in parallel
  - All lock canister LP positions in parallel

benefit: Faster data gathering
```

### 4. Sequential Trading
```yaml
reason: Kongswap design requirement
implementation: One trade per hour via timer
trades: Either buy or sell, never both
```

### 5. Safety in Minting/Burning
```yaml
minting:
  - Two-phase with refund protection
  - TVL snapshot before deposit
  - Atomic supply queries

burning:
  - Single atomic operation
  - User burns before calling
  - Proportional redemption
  - Continue on partial failure
```

### 6. Rebalancing Logic
```yaml
priority_1: If ckUSDT available (≥$10), buy underweight
priority_2: Otherwise, sell overweight to ckUSDT
threshold: 1% deviation required
trade_size: 10% of deviation
frequency: Hourly automatic + manual trigger
```

---

## Important Constants Reference

```yaml
decimals:
  ICPI: 8 decimals
  ckUSDT: 6 decimals
  ALEX/ZERO/KONG/BOB: 8 decimals

fees:
  operation_fee: 0.1 ckUSDT (100,000 with 6 decimals)
  transfer_fees: 10,000 units per token
  approval_gas: included in approval amount

minimums:
  mint: 1 ckUSDT (1,000,000 with 6 decimals)
  burn: 11,000 units (0.00011 ICPI)

intervals:
  rebalancing: 3600 seconds (1 hour)
  cache_refresh: 300 seconds (5 minutes)
  mint_cleanup: 300 seconds (5 minutes)

timeouts:
  mint_operation: 180 seconds (3 minutes)

cache_ttls:
  lock_canisters: 3600 seconds
  tvl: 300 seconds
  index_state: 300 seconds
```

---

## Data Flow Examples

### Minting Flow
```yaml
1. User approves backend for ckUSDT spending
2. User calls initiate_mint(amount)
   → Backend creates pending mint
   → Returns mint_id

3. User calls complete_mint(mint_id)
   → Collect 0.1 ckUSDT fee
   → Snapshot: supply = 1000, TVL = 10,000
   → Collect deposit: 100 ckUSDT
   → Calculate: (100 × 1000) ÷ 10,000 = 10 ICPI
   → Mint 10 ICPI to user via ledger
   → Return 10 ICPI
```

### Burning Flow
```yaml
1. User transfers 10 ICPI to backend
   → Automatically burned (backend is minting account)

2. User calls burn_icpi(10 ICPI)
   → Collect 0.1 ckUSDT fee
   → Query supply: now 990 (user's 10 already burned)
   → Calculate redemptions:
      - ckUSDT: (10 × 150) ÷ 990 = 1.515 ckUSDT
      - ALEX: (10 × 500) ÷ 990 = 5.05 ALEX
      - etc. for each token
   → Transfer all tokens to user
   → Return BurnResult with details
```

### Rebalancing Flow
```yaml
1. Timer triggers (hourly)

2. Query index state:
   → Current: ALEX=40%, ZERO=30%, KONG=20%, BOB=10%
   → Target: ALEX=50%, ZERO=25%, KONG=20%, BOB=5%
   → Deviations: ALEX=+10%, ZERO=-5%, KONG=0%, BOB=-5%

3. Check ckUSDT balance:
   → If ≥$10: Buy most underweight (ALEX)
   → Otherwise: Sell most overweight (ZERO)

4. Execute trade:
   → Trade size = 10% of deviation
   → ALEX deviation = $100, trade = $10
   → execute_swap(ckUSDT, $10, ALEX, 2% slippage)

5. Record result in history
```

---

## End of Document

This pseudocode reference provides a high-level understanding of the ICPI backend architecture. For implementation details, refer to the actual Rust source code.

**Generated**: 2025-10-02
**Version**: Backend v0.1.0
**Purpose**: Offline code review and understanding
