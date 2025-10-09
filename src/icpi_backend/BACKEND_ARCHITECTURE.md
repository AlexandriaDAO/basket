# ICPI Backend - Complete Architecture Documentation
**Generated**: 2025-10-09
**Version**: 0.1.0
**Purpose**: Comprehensive documentation of all backend functionality

---

## Table of Contents
1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Module Details](#module-details)
4. [Data Flow Examples](#data-flow-examples)
5. [Security Features](#security-features)
6. [Constants Reference](#constants-reference)

---

## Overview

The ICPI (Internet Computer Portfolio Index) backend manages a token basket that tracks locked liquidity from Kong Locker. It implements:

- **Two-phase minting** with atomic TVL snapshots
- **Atomic burning** with proportional redemption
- **Hourly rebalancing** to maintain target allocations
- **Kong Liquidity integration** for TVL and pricing
- **Kongswap DEX trading** for rebalancing

### Three Canisters

1. **ICPI Token** (`l6lep-niaaa-aaaap-qqeda-cai`) - ICRC-1 ledger
2. **Backend** (`ev6xm-haaaa-aaaap-qqcza-cai`) - Business logic (this canister)
3. **Frontend** (`qhlmp-5aaaa-aaaam-qd4jq-cai`) - React UI

Backend is the minting authority - tokens transferred to it are automatically burned.

---

## Architecture

### Zone-Based Organization

```
6_INFRASTRUCTURE     Foundation layer (math, errors, constants, admin)
  ↓
2_CRITICAL_DATA      Data queries and validation (supply, balances, TVL)
  ↓
3_KONG_LIQUIDITY     External data sources (locker, pools, TVL)
  ↓
4_TRADING_EXECUTION  Swap execution (approvals, swaps, slippage)
  ↓
1_CRITICAL_OPERATIONS State-changing ops (mint, burn, rebalance)
  ↓
5_INFORMATIONAL      Display and monitoring
```

### File Structure

```
icpi_backend/
├── Cargo.toml                    # Dependencies
├── icpi_backend.did              # Candid API
└── src/
    ├── lib.rs                    # Entry point
    ├── types/                    # Type definitions
    │   ├── mod.rs
    │   ├── common.rs             # HealthStatus, ServiceError
    │   ├── tokens.rs             # TrackedToken enum
    │   ├── portfolio.rs          # CurrentPosition, IndexState
    │   ├── rebalancing.rs        # TargetAllocation, AllocationDeviation
    │   ├── kongswap.rs           # SwapArgs, SwapReply, LPBalancesReply
    │   └── icrc.rs               # ICRC-1/ICRC-2 types
    ├── 1_CRITICAL_OPERATIONS/
    │   ├── mod.rs
    │   ├── minting/
    │   │   ├── mod.rs
    │   │   ├── mint_state.rs     # PendingMint, MintStatus
    │   │   ├── mint_validator.rs # Input validation
    │   │   ├── mint_orchestrator.rs  # Main minting logic
    │   │   ├── fee_handler.rs    # Fee collection
    │   │   └── refund_handler.rs # Refund on failure
    │   ├── burning/
    │   │   ├── mod.rs            # burn_icpi main function
    │   │   ├── burn_validator.rs # Input validation
    │   │   ├── redemption_calculator.rs  # Proportional calc
    │   │   └── token_distributor.rs      # Token transfers
    │   └── rebalancing/
    │       └── mod.rs            # Hourly timer, buy/sell logic
    ├── 2_CRITICAL_DATA/
    │   ├── mod.rs                # get_supply_and_tvl_atomic()
    │   ├── supply_tracker/
    │   │   └── mod.rs            # ICPI supply queries
    │   ├── token_queries/
    │   │   └── mod.rs            # Token balance queries
    │   ├── portfolio_value/
    │   │   └── mod.rs            # Portfolio valuation
    │   └── validation/
    │       └── mod.rs            # Supply/price validation
    ├── 3_KONG_LIQUIDITY/
    │   ├── mod.rs
    │   ├── locker/
    │   │   └── mod.rs            # Lock canister queries
    │   ├── pools/
    │   │   └── mod.rs            # Token pricing
    │   └── tvl/
    │       └── mod.rs            # TVL calculation (1hr cache)
    ├── 4_TRADING_EXECUTION/
    │   ├── mod.rs
    │   ├── swaps/
    │   │   └── mod.rs            # Swap execution
    │   ├── approvals/
    │   │   └── mod.rs            # ICRC-2 approvals
    │   └── slippage/
    │       └── mod.rs            # Slippage protection
    ├── 5_INFORMATIONAL/
    │   ├── mod.rs
    │   ├── health/
    │   │   └── mod.rs            # Health status
    │   ├── cache/
    │   │   └── mod.rs            # Cache management
    │   └── display/
    │       └── mod.rs            # Index state formatting
    └── 6_INFRASTRUCTURE/
        ├── mod.rs
        ├── constants/
        │   └── mod.rs            # All constants
        ├── errors/
        │   └── mod.rs            # Error types
        ├── math/
        │   ├── mod.rs
        │   └── pure_math.rs      # Pure math functions
        ├── cache/
        │   └── mod.rs            # Infrastructure cache
        ├── logging/
        │   └── mod.rs            # Logging utilities
        ├── admin/
        │   └── mod.rs            # Admin controls
        ├── reentrancy/
        │   └── mod.rs            # Reentrancy guards
        ├── rate_limiting/
        │   └── mod.rs            # Rate limiting
        ├── stable_storage/
        │   └── mod.rs            # Upgrade persistence
        └── types/
            └── mod.rs            # Infrastructure types
```

---

## Module Details

### Zone 6: Infrastructure

#### constants/mod.rs
**Purpose**: Single source of truth for all configuration values

```rust
// Canister IDs
ICPI_CANISTER_ID: "l6lep-niaaa-aaaap-qqeda-cai"
ICPI_BACKEND_ID: "ev6xm-haaaa-aaaap-qqcza-cai"
CKUSDT_CANISTER_ID: "cngnf-vqaaa-aaaar-qag4q-cai"
KONGSWAP_BACKEND_ID: "2ipq2-uqaaa-aaaar-qailq-cai"
KONG_LOCKER_ID: "eazgb-giaaa-aaaap-qqc2q-cai"

// Decimals
ICPI_DECIMALS: 8
CKUSDT_DECIMALS: 6

// Minting
MIN_MINT_AMOUNT: 100_000 (0.1 ckUSDT e6)
MAX_MINT_AMOUNT: 100_000_000_000 (100k ckUSDT)
MINT_TIMEOUT_NANOS: 180_000_000_000 (3 minutes)
MINT_FEE_AMOUNT: 100_000 (0.1 ckUSDT)
FEE_RECIPIENT: "e454q-riaaa-aaaap-qqcyq-cai"

// Burning
MIN_BURN_AMOUNT: 11_000 (0.00011 ICPI e8)
BURN_FEE_BUFFER: 10_000 (transfer fee buffer)

// Rebalancing
REBALANCE_INTERVAL_SECONDS: 3600 (1 hour)
MIN_DEVIATION_PERCENT: 1.0 (1% minimum)
TRADE_INTENSITY: 0.1 (10% of deviation per hour)
MAX_SLIPPAGE_PERCENT: 2.0 (2% max slippage)
MIN_TRADE_SIZE_USD: 1.0 ($1 minimum trade)

// Validation
MAX_SUPPLY_CHANGE_RATIO: 1.1 (10% max supply change)
MAX_PRICE_CHANGE_RATIO: 2.0 (100% max price change)
MIN_REASONABLE_PRICE: 0.0001
MAX_REASONABLE_PRICE: 1_000_000.0

// Cache durations (seconds)
CACHE_DURATION_SHORT: 30
CACHE_DURATION_MEDIUM: 300
CACHE_DURATION_LONG: 3600
```

#### errors/mod.rs
**Purpose**: Structured error types for the entire application

```rust
pub type Result<T> = std::result::Result<T, IcpiError>;

pub enum IcpiError {
    Mint(MintError),
    Burn(BurnError),
    Rebalance(RebalanceError),
    Trading(TradingError),
    Kongswap(KongswapError),
    Validation(ValidationError),
    Calculation(CalculationError),
    System(SystemError),
    Query(QueryError),
    Other(String),
}

// Each error type has detailed variants with context
MintError:
  - InvalidMintId { id }
  - AmountBelowMinimum { amount, minimum }
  - AmountAboveMaximum { amount, maximum }
  - FeeCollectionFailed { user, reason }
  - DepositCollectionFailed { user, amount, reason }
  - RefundFailed { user, amount, reason }
  - InsufficientTVL { tvl, required }
  - LedgerInteractionFailed { operation, details }
  - Unauthorized { principal, mint_id }
  - ProportionalCalculationError { reason }

BurnError:
  - AmountBelowMinimum { amount, minimum }
  - AmountExceedsMaximum { amount, maximum, percentage_limit }
  - InsufficientApproval { required, approved }
  - InsufficientBalance { required, available }
  - InsufficientFeeAllowance { required, approved }
  - NoSupply
  - NoRedemptionsPossible { reason }
  - TokenTransferFailed { token, amount, reason }

RebalanceError:
  - TimerNotActive
  - TooSoonToRebalance { last_time, next_time }
  - AllocationCalculationError { reason }
  - SwapFailed { token, amount, reason }
  - InsufficientBalance { token, available, required }
  - RebalancingInProgress

TradingError:
  - InvalidQuote { reason }
  - SlippageTooHigh { expected, actual, max_allowed }
  - ApprovalFailed { token, amount, reason }
  - InvalidTokenCanister { token, canister_id, reason }
  - KongswapError { operation, message }
  - SlippageExceeded { expected, actual, max_allowed, actual_slippage }
  - SwapFailed { pay_token, receive_token, amount, reason }
  - InvalidSwapAmount { reason }
```

#### math/pure_math.rs
**Purpose**: Deterministic mathematical functions with no side effects

**Key Functions:**

1. **multiply_and_divide**(a: &Nat, b: &Nat, c: &Nat) -> Result<Nat>
   - Formula: (a × b) ÷ c
   - Uses BigUint for arbitrary precision
   - Prevents overflow and division by zero

2. **convert_decimals**(amount: &Nat, from_decimals: u32, to_decimals: u32) -> Result<Nat>
   - Converts between different decimal places
   - Example: ckUSDT e6 ↔ ICPI e8
   - Detects precision loss

3. **calculate_mint_amount**(deposit: &Nat, supply: &Nat, tvl: &Nat) -> Result<Nat>
   - Initial mint: 1:1 ratio (adjusted for decimals)
   - Subsequent: (deposit × supply) ÷ tvl
   - Validates non-zero result

4. **calculate_redemptions**(burn_amount: &Nat, supply: &Nat, balances: &[(String, Nat)]) -> Result<Vec<(String, Nat)>>
   - Proportional: (burn_amount × balance) ÷ supply
   - Returns list of (token, amount) pairs

5. **calculate_trade_size**(deviation_usd: f64, trade_intensity: f64, min_trade_size: f64) -> Result<Nat>
   - Trade size = deviation × intensity
   - Returns 0 if below minimum
   - Converts to e6 decimals (ckUSDT)

#### admin/mod.rs
**Purpose**: Admin controls and emergency functions

**Admin Principals:**
- `ev6xm-haaaa-aaaap-qqcza-cai` - Backend (for timers)
- `67ktx-ln42b-uzmo5-bdiyn-gu62c-cd4h4-a5qt3-2w3rs-cixdl-iaso2-mqe` - Deployer

**Functions:**
- `require_admin()` - Verify caller is admin
- `check_not_paused()` - Verify system not paused
- `set_pause(bool)` - Activate/deactivate emergency pause
- `is_paused()` - Get current pause state
- `log_admin_action(String)` - Record admin actions
- `get_admin_log()` - Retrieve action log

**Emergency Pause:**
- Thread-local state (NOT persisted across upgrades!)
- Blocks minting, burning, rebalancing
- Admin must re-enable after upgrade if needed
- Max 1000 log entries kept

#### reentrancy/mod.rs
**Purpose**: Two-layer protection for concurrent operations

**Layer 1: Per-User Guards**
```rust
MintGuard::acquire(user: Principal) -> Result<Self>
BurnGuard::acquire(user: Principal) -> Result<Self>
```
- Prevents same user from multiple concurrent operations
- Allows different users to operate simultaneously
- Guard automatically released on drop

**Layer 2: Global Operation Coordination**
```rust
enum GlobalOperation {
    Idle,
    Minting,
    Burning,
    Rebalancing,
}

try_start_global_operation(op: GlobalOperation) -> Result<()>
end_global_operation(op: GlobalOperation)
get_current_operation() -> GlobalOperation
has_active_operations() -> bool
```

**Coordination Rules:**
- Rebalancing blocks new mints/burns
- Mints/burns block new rebalancing
- Mints and burns can coexist (per-user guards prevent conflicts)
- 60-second grace period between operation type switches

#### rate_limiting/mod.rs
**Purpose**: Prevent abuse and spam

```rust
check_rate_limit(key: &str, limit_nanos: u64) -> Result<()>
periodic_cleanup()  // Clears old entries
```

**Features:**
- Thread-local HashMap storage
- Per-operation keys (e.g., "mint_{principal}")
- Automatic cleanup at 1000 entries
- Periodic cleanup every hour

#### stable_storage/mod.rs
**Purpose**: Persist pending mints across upgrades

```rust
pub struct StableState {
    pub pending_mints: HashMap<String, PendingMint>,
}

save_state(pending_mints: HashMap<String, PendingMint>)
restore_state() -> HashMap<String, PendingMint>
```

**Behavior:**
- Saves to stable memory in `pre_upgrade`
- Restores in `post_upgrade`
- Drops mints older than 24 hours on restore
- Graceful degradation if serialization fails

---

### Zone 1: Critical Operations

#### minting/mint_state.rs
**Purpose**: Track pending mint operations

```rust
pub enum MintStatus {
    Pending,                // Created, waiting for complete_mint
    CollectingFee,          // Charging 0.1 ckUSDT fee
    Snapshotting,           // Taking supply/TVL snapshot
    CollectingDeposit,      // Receiving ckUSDT from user
    Calculating,            // Computing ICPI amount
    Minting,                // Creating ICPI tokens
    Refunding,              // Returning deposit after failure
    Complete(Nat),          // Finished successfully
    Failed(String),         // Generic failure
    FailedRefunded(String), // Failed but deposit returned
    FailedNoRefund(String), // Failed and refund also failed
    Expired,                // Timeout exceeded
}

pub struct MintSnapshot {
    pub supply: Nat,        // ICPI supply when snapshot taken
    pub tvl: Nat,           // Portfolio TVL when snapshot taken
    pub timestamp: u64,     // When snapshot was taken
}

pub struct PendingMint {
    pub id: String,         // Unique mint ID
    pub user: Principal,    // Who initiated
    pub amount: Nat,        // ckUSDT deposit amount
    pub status: MintStatus,
    pub created_at: u64,
    pub last_updated: u64,
    pub snapshot: Option<MintSnapshot>,  // Pre-deposit snapshot
}
```

**Functions:**
- `store_pending_mint(mint: PendingMint)`
- `get_pending_mint(mint_id: &str) -> Option<PendingMint>`
- `get_mint_status(mint_id: &str) -> Option<MintStatus>`
- `update_mint_status(mint_id: &str, status: MintStatus)`
- `cleanup_expired_mints() -> u32`  // Returns count cleaned
- `get_pending_count() -> usize`
- `export_state()` / `import_state()` - For upgrades

**Cleanup Policy:**
- Pending/failed mints: 3 minutes
- Completed mints: 24 hours
- Runs hourly via timer

#### minting/mint_orchestrator.rs
**Purpose**: Main two-phase minting logic

**Phase 1: initiate_mint**(caller: Principal, amount: Nat) -> Result<String>
1. Validate request (amount, principal, rate limit)
2. Generate unique mint_id
3. Create PendingMint record
4. Store in state
5. Return mint_id

**Phase 2: complete_mint**(caller: Principal, mint_id: String) -> Result<Nat>
1. Check not paused
2. Acquire MintGuard (reentrancy protection)
3. Get pending mint and verify ownership
4. **Step 1**: Collect 0.1 ckUSDT fee
5. **Step 2**: Take atomic snapshot (supply + TVL in parallel)
   - CRITICAL: Snapshot BEFORE collecting deposit
   - Validates TVL is non-zero
   - Checks snapshot age (warning at 30s, error at 60s)
6. **Step 3**: Collect deposit from user
7. **Step 4**: Calculate ICPI amount using pure math
   - Uses snapshot values (pre-deposit)
   - Validates result is non-zero
8. **Step 5**: Mint ICPI on ledger
   - Backend is minting account
   - Transfer creates new tokens
9. **Step 6**: Mark mint as complete
10. **Error Handling**: Attempt refund on any failure after deposit collected

**Why Snapshot Before Deposit:**
```
Example:
  Current: TVL = $100, Supply = 100 ICPI
  User deposits: $10 (should get 10% ownership)

  CORRECT (snapshot before):
    Calculation: (10 × 100) / 100 = 10 ICPI
    Result: 10/110 = 9.09% ownership ✓

  WRONG (snapshot after):
    Calculation: (10 × 100) / 110 = 9.09 ICPI
    Result: 9.09/109.09 = 8.33% ownership ✗
```

**Helper Functions:**
- `handle_mint_failure()` - Refund and update status
- `mint_icpi_on_ledger()` - Call ICPI ledger

#### minting/fee_handler.rs
**Purpose**: Collect fees from users

```rust
collect_mint_fee(user: Principal) -> Result<Nat>
  - Uses ICRC-2 transfer_from
  - Requires prior approval
  - Transfers 0.1 ckUSDT to backend
  - Returns fee amount

collect_deposit(user: Principal, amount: Nat, memo: String) -> Result<Nat>
  - Uses ICRC-2 transfer_from
  - Requires prior approval
  - Transfers ckUSDT to backend
  - Returns deposited amount
```

#### minting/refund_handler.rs
**Purpose**: Return deposits on failure

```rust
refund_deposit(user: Principal, amount: Nat) -> Result<Nat>
  - Uses ICRC-1 transfer
  - Sends ckUSDT back to user
  - Returns block index
  - Called automatically on mint failure
```

#### burning/mod.rs
**Purpose**: Single atomic burn operation

**burn_icpi**(caller: Principal, amount: Nat) -> Result<BurnResult>
1. Check not paused
2. Acquire BurnGuard (reentrancy protection)
3. Validate request (amount, principal, rate limit)
4. Check ckUSDT fee approval (0.1 ckUSDT required)
5. Get current supply (atomically, BEFORE collecting anything)
6. Validate burn limit (max 10% of supply per transaction)
7. Check user has sufficient ICPI balance
8. Collect 0.1 ckUSDT fee
9. **Transfer ICPI to backend** (automatically burns it via ICRC-2)
   - User must have approved backend first
   - Backend is burning account
10. Calculate proportional redemptions for all tokens
11. Distribute tokens to user (parallel transfers)
12. Return BurnResult with success/failure details

```rust
pub struct BurnResult {
    pub successful_transfers: Vec<(String, Nat)>,
    pub failed_transfers: Vec<(String, Nat, String)>,
    pub icpi_burned: Nat,
    pub timestamp: u64,
}
```

**Why Atomic (vs Two-Phase Minting):**
- Burning is instant - ICPI transferred to backend immediately burns
- No race conditions - user's tokens already removed from supply
- Simpler UX - single call instead of two
- Still proportional - calculation uses post-burn supply

#### burning/burn_validator.rs
**Purpose**: Validate burn requests

```rust
validate_burn_request(caller: &Principal, amount: &Nat) -> Result<()>
  - Check not anonymous
  - Check minimum amount (11,000 e8)
  - Rate limiting (1 second)

validate_burn_limit(amount: &Nat, supply: &Nat) -> Result<()>
  - Max 10% of supply per transaction
  - Uses BigUint arithmetic (no u128 ceiling)
  - Formula: amount * 100 > supply * 10 → error
```

#### burning/redemption_calculator.rs
**Purpose**: Calculate proportional redemptions

```rust
calculate_redemptions(burn_amount: &Nat, current_supply: &Nat) -> Result<Vec<(String, Nat)>>
  1. Get all token balances
  2. For each token:
     - Calculate: (burn_amount × balance) ÷ supply
     - Subtract transfer fee (10,000)
     - Skip if below dust threshold
  3. Return list of (token, amount) pairs
  4. Error if no redemptions possible

calculate_proportional_share(burn_amount: &Nat, token_balance: &Nat, total_supply: &Nat) -> Result<Nat>
  - Pure function for single token
  - Uses multiply_and_divide from math module
```

#### burning/token_distributor.rs
**Purpose**: Execute redemption transfers

```rust
distribute_tokens(recipient: Principal, redemptions: Vec<(String, Nat)>, icpi_burned: Nat) -> Result<BurnResult>
  - Executes all transfers in parallel (futures::join_all)
  - Continues even if some transfers fail
  - Returns detailed BurnResult
  - Errors only if ALL transfers fail
```

#### rebalancing/mod.rs
**Purpose**: Hourly rebalancing automation

**Strategy:**
- Run every 3600 seconds (1 hour)
- One trade per hour (Kong limitation)
- 10% of deviation per trade (gradual rebalancing)
- Priority: Buy underweight if ckUSDT available, else sell overweight

**Data Structures:**
```rust
pub enum RebalanceAction {
    None,
    Buy { token: TrackedToken, usdt_amount: f64 },
    Sell { token: TrackedToken, usdt_value: f64 },
}

pub struct RebalanceRecord {
    pub timestamp: u64,
    pub action: RebalanceAction,
    pub success: bool,
    pub details: String,
}

pub struct RebalancerStatus {
    pub timer_active: bool,
    pub last_rebalance: Option<u64>,
    pub next_rebalance: Option<u64>,
    pub recent_history: Vec<RebalanceRecord>,  // Last 10
}
```

**Functions:**

1. **start_rebalancing_timer()**
   - Called in init() and post_upgrade()
   - Sets 1-hour interval timer
   - Each tick spawns hourly_rebalance() async
   - Checks GlobalOperation lock before proceeding

2. **perform_rebalance()** -> Result<String>
   - Manual trigger (admin only)
   - Checks not paused
   - Acquires GlobalOperation lock
   - Calls hourly_rebalance()

3. **hourly_rebalance()** -> Result<String>
   - Get portfolio state from Zone 5
   - Analyze deviations via get_rebalancing_action()
   - Execute buy or sell action
   - Record in history

4. **get_rebalancing_action**(deviations: &[AllocationDeviation], ckusdt_balance: &Nat) -> Result<RebalanceAction>
   - If ckUSDT >= $10: Buy most underweight token
   - Else if overweight exists: Sell most overweight token
   - Else: None (portfolio balanced)
   - Minimum $1 deviation required
   - Trade size = 10% of deviation

5. **execute_buy_action**(token: &TrackedToken, usd_amount: f64) -> Result<String>
   - Convert USD to ckUSDT e6
   - Call Zone 4 execute_swap()
   - Log results and record in history

6. **execute_sell_action**(token: &TrackedToken, usd_value: f64) -> Result<String>
   - Get current token price
   - Calculate token amount to sell
   - Check sufficient balance
   - Call Zone 4 execute_swap()
   - Log results and record in history

7. **record_rebalance**(action, success, details)
   - Adds to history (max 10 entries)
   - Updates last_rebalance timestamp

**Coordination:**
- Local REBALANCING_IN_PROGRESS flag
- Global operation lock (blocks mints/burns)
- Releases locks on completion or error

---

### Zone 2: Critical Data

#### mod.rs
**Purpose**: Atomic data queries

```rust
get_supply_and_tvl_atomic() -> Result<(Nat, Nat)>
  - Queries supply and TVL in parallel (futures::join!)
  - Retries up to 2 times on failure
  - Validates consistency (supply>0 XOR tvl>0 is error)
  - Detects data corruption
  - Returns (supply_e8, tvl_e6)
```

**Consistency Validation:**
```
Valid states:
  - Both zero (initial state)
  - Both positive (normal operation)

Invalid states:
  - Supply > 0, TVL = 0  → Error (data corruption)
  - Supply = 0, TVL > 0  → Error (data corruption)
```

#### supply_tracker/mod.rs
**Purpose**: Query ICPI total supply

```rust
get_icpi_supply_uncached() -> Result<Nat>
  - Calls icrc1_total_supply on ICPI ledger
  - Validates supply < 10^16 (100M ICPI max)
  - No caching (financial accuracy)
  - Returns supply in e8 decimals

get_validated_supply() -> Result<Nat>
  - Convenience wrapper
```

#### token_queries/mod.rs
**Purpose**: Query token balances

```rust
get_token_balance_uncached(token: &TrackedToken) -> Result<Nat>
  - Calls icrc1_balance_of on token canister
  - Queries backend's account
  - No caching (financial accuracy)

get_all_balances_uncached() -> Result<Vec<(String, Nat)>>
  - Queries all tracked tokens in parallel
  - Includes ckUSDT
  - Allows partial failures (logs errors)

get_ckusdt_balance() -> Result<Nat>
  - Specific ckUSDT query
  - Returns in e6 decimals
```

#### portfolio_value/mod.rs
**Purpose**: Calculate portfolio value and state

```rust
calculate_portfolio_value_atomic() -> Result<Nat>
  - Gets all balances in parallel
  - For each token:
    - ckUSDT: 1:1 with USD (e6)
    - Others: balance × price_in_usdt
  - Validates total < $1 trillion
  - Returns value in e6 (ckUSDT units)

get_token_usd_value(token_symbol: &str, amount: &Nat) -> Result<u64>
  - Returns 0 for zero amounts (skip pricing)
  - Gets price from Zone 3 pools
  - Calculates: (amount_e8 × price_e6) / 1e8
  - Prevents overflow with checked arithmetic
  - Returns value in e6 (ckUSDT units)

get_portfolio_state_uncached() -> Result<IndexState>
  - Gets all balances
  - Calculates total value
  - Builds CurrentPosition list with USD values
  - Gets target allocations from Kong Locker TVL
  - Calculates deviations (current vs target)
  - Returns complete IndexState
```

**IndexState Structure:**
```rust
pub struct IndexState {
    pub total_value: f64,                          // Portfolio USD value
    pub current_positions: Vec<CurrentPosition>,   // Actual holdings
    pub target_allocations: Vec<TargetAllocation>, // Desired holdings
    pub deviations: Vec<AllocationDeviation>,      // What needs rebalancing
    pub timestamp: u64,
    pub ckusdt_balance: Nat,                       // Available for trades
}
```

#### validation/mod.rs
**Purpose**: Data validation

```rust
validate_supply(new_supply: &Nat, cached_supply: Option<&Nat>) -> Result<()>
  - Max 10 billion ICPI

validate_price(token: &str, price: f64, cached_price: Option<f64>) -> Result<()>
  - Min: $0.0001
  - Max: $1,000,000
```

---

### Zone 3: Kong Liquidity

#### locker/mod.rs
**Purpose**: Query Kong Locker for lock canisters

```rust
get_all_lock_canisters() -> Result<Vec<(Principal, Principal)>>
  - Calls kong_locker.get_all_lock_canisters()
  - Returns list of (user, lock_canister) pairs
  - No caching
```

#### pools/mod.rs
**Purpose**: Query token prices from Kongswap

```rust
get_token_price_in_usdt(token: &TrackedToken) -> Result<f64>
  - Special case: ckUSDT always returns 1.0
  - Queries kongswap.swap_amounts(symbol, 1.0e8, "ckUSDT")
  - Returns price per token in USDT
  - Validates price is reasonable (0.000001 to 100)
  - Example: ALEX price = 0.0012 means 1 ALEX = 0.0012 ckUSDT
```

#### tvl/mod.rs
**Purpose**: Calculate locked liquidity TVL

**With 1-hour caching:**
```rust
calculate_kong_locker_tvl() -> Result<Vec<(TrackedToken, f64)>>
  - Checks cache (TTL: 3600s)
  - Returns cached data if valid
  - Otherwise calls calculate_kong_locker_tvl_uncached()
  - Updates cache
  - Returns TVL by token in USD

clear_tvl_cache()
  - Manual cache invalidation
```

**Uncached calculation:**
```rust
calculate_kong_locker_tvl_uncached() -> Result<Vec<(TrackedToken, f64)>>
  1. Get all lock canisters from locker module
  2. Query each lock canister's balances from Kongswap (parallel)
  3. For each LP position:
     - CRITICAL: Use usd_amount_0 and usd_amount_1, NOT usd_balance
     - usd_balance = total value of both sides (would double-count)
     - usd_amount_X = value of one side only
     - If symbol_0 is tracked: add usd_amount_0 to that token's TVL
     - If symbol_1 is tracked: add usd_amount_1 to that token's TVL
  4. Sum by token
  5. Validate success rate > 50%
  6. Return [(ALEX, $X), (ZERO, $Y), (KONG, $Z), (BOB, $W)]

  Partial failure handling:
    - Ok(Some(...)) = successful query
    - Ok(None) = query failed (logged, continue)
    - Fails hard if < 50% success rate
```

**Example Output:**
```
Total Kong Locker TVL: $23,191.75
  ALEX: $22,500.00
  ZERO: $640.89
  KONG: $48.71
  BOB: $2.05
```

---

### Zone 4: Trading Execution

#### swaps/mod.rs
**Purpose**: Execute token swaps on Kongswap

```rust
execute_swap(
    pay_token: &TrackedToken,
    pay_amount: Nat,
    receive_token: &TrackedToken,
    max_slippage: f64,
) -> Result<SwapReply>

Process:
  1. Validate inputs (amount > 0, slippage 0-10%, tokens different)
  2. Approve tokens via Zone 4 approvals module
  3. Query expected output via swap_amounts (for slippage check)
  4. Execute swap:
     - pay_tx_id: None (CRITICAL: ICRC-2 flow)
     - receive_address: backend canister
     - max_slippage: as provided
  5. Validate actual slippage vs max_slippage
  6. Log results
  7. Return SwapReply

query_swap_amounts(pay_symbol: &str, pay_amount: Nat, receive_symbol: &str) -> Result<Nat>
  - Calls kongswap.swap_amounts()
  - Returns expected receive amount
  - Used for slippage calculation

validate_swap_params(...) -> Result<()>
  - Pay amount > 0
  - Max slippage 0-10%
  - Tokens are different
```

#### approvals/mod.rs
**Purpose**: ICRC-2 token approvals

```rust
approve_token_for_swap(token: &TrackedToken, amount: Nat) -> Result<Nat>
  - Calls icrc2_approve on token canister
  - Spender: Kongswap backend
  - Expiry: 15 minutes (increased from 5 for network congestion)
  - Memo: "ICPI rebalancing"
  - Returns approval block index

check_kongswap_allowance(token: &TrackedToken) -> Result<Nat>
  - Debugging helper
  - Not used in production flow
```

**Approval Expiry:**
- 15 minutes (900_000_000_000 nanos)
- Balances security and network handling
- Unused approvals expire automatically

#### slippage/mod.rs
**Purpose**: Slippage protection

```rust
calculate_min_receive(expected_amount: &Nat, max_slippage: f64) -> Nat
  - Formula: expected × (1 - max_slippage)
  - Example: 100 tokens, 2% slippage → 98 minimum

validate_swap_result(expected: &Nat, actual: &Nat, max_slippage: f64) -> Result<()>
  - Calculates: actual_slippage = (expected - actual) / expected
  - Positive slippage (got more) is always OK
  - Validates actual_slippage <= max_slippage
  - Errors if zero expected amount
  - Errors if slippage exceeded
```

**Examples:**
```
Good: Expected 100, got 98, max 2% → Pass (2% slippage)
Good: Expected 100, got 105, max 2% → Pass (positive slippage)
Bad:  Expected 100, got 95, max 2% → Error (5% slippage)
Bad:  Expected 0, got anything      → Error (invalid)
```

---

### Zone 5: Informational

#### display/mod.rs
**Purpose**: Format index state for UI

```rust
get_index_state_cached() -> Result<IndexState>
  - Calls Zone 2 get_portfolio_state_uncached()
  - Propagates errors (no silent failures)
  - Returns complete portfolio state

Note: Name is misleading - there's no actual caching yet
TODO: Implement real caching in future phase
```

#### health/mod.rs
**Purpose**: System health monitoring

```rust
get_health_status() -> HealthStatus
  - Version from Cargo.toml
  - List of tracked tokens
  - Last rebalance timestamp
  - Cycles balance

get_tracked_tokens() -> Vec<String>
  - Returns ["ALEX", "ZERO", "KONG", "BOB"]
```

#### cache/mod.rs
**Purpose**: Cache management

```rust
clear_all_caches()
  - Clears thread-local cache map
  - Also clears TVL cache (Zone 3)
  - Called by admin clear_caches() endpoint
```

---

## Data Flow Examples

### Minting Flow

**User Perspective:**
```
1. User calls icrc2_approve on ckUSDT ledger
   - Spender: backend
   - Amount: deposit + fee (e.g., 1.1 ckUSDT)

2. User calls initiate_mint(amount)
   → Returns mint_id

3. User calls complete_mint(mint_id)
   → Returns ICPI amount minted
```

**Backend Execution:**
```
initiate_mint(1_000_000):
  ✓ Validate amount >= 100_000
  ✓ Rate limit check
  ✓ Create PendingMint
  → Return "mint_67ktx_1728518400000"

complete_mint("mint_67ktx_1728518400000"):
  1. Check not paused
  2. Acquire MintGuard
  3. Collect 0.1 ckUSDT fee
     → transfer_from user to backend

  4. Take atomic snapshot (parallel):
     supply_future = get_icpi_supply_uncached()
     tvl_future = calculate_portfolio_value_atomic()
     → supply = 100_000_000 (1 ICPI, e8)
     → tvl = 1_000_000 (1 ckUSDT, e6)

  5. Validate snapshot age < 60s

  6. Collect deposit
     → transfer_from 1_000_000 ckUSDT from user

  7. Calculate ICPI:
     Initial? No (supply > 0)
     Formula: (deposit_e8 × supply) / tvl_e8
              = (100_000_000 × 100_000_000) / 100_000_000
              = 100_000_000 (1 ICPI)

  8. Mint ICPI on ledger
     → transfer 100_000_000 from minting account to user

  9. Mark complete
  → Return 100_000_000
```

**On Error (After Deposit):**
```
  If step 7 or 8 fails:
    → refund_deposit(user, 1_000_000)
    → Update status to FailedRefunded or FailedNoRefund

  Fee is NOT refunded (spam prevention)
```

### Burning Flow

**User Perspective:**
```
1. User calls icrc2_approve on ckUSDT ledger
   - Spender: backend
   - Amount: 100_000 (0.1 ckUSDT fee)

2. User calls icrc2_approve on ICPI ledger
   - Spender: backend
   - Amount: burn_amount (e.g., 10_000_000 = 0.1 ICPI)

3. User calls burn_icpi(amount)
   → Returns BurnResult with token distributions
```

**Backend Execution:**
```
burn_icpi(10_000_000):  // 0.1 ICPI
  1. Check not paused
  2. Acquire BurnGuard
  3. Validate amount >= 11_000
  4. Check ckUSDT fee approval >= 100_000
  5. Get current supply = 1_000_000_000 (10 ICPI)
  6. Validate burn limit (10_000_000 < 10% of supply ✓)
  7. Check user ICPI balance >= 10_000_000
  8. Collect 0.1 ckUSDT fee
  9. Transfer ICPI from user to backend (BURNS IT!)
     → icrc2_transfer_from 10_000_000 ICPI
     → Automatically reduces supply to 990_000_000

  10. Calculate redemptions (supply now 990_000_000):
      get_all_balances_uncached() = [
        ("ALEX", 1_000_000_000),
        ("ZERO", 500_000_000),
        ("KONG", 300_000_000),
        ("BOB", 100_000_000),
        ("ckUSDT", 150_000),
      ]

      For each token:
        redemption = (10_000_000 × balance) / 990_000_000

      ALEX: (10_000_000 × 1_000_000_000) / 990_000_000
          = 10_101_010 - 10_000 fee = 10_091_010

      ZERO: (10_000_000 × 500_000_000) / 990_000_000
          = 5_050_505 - 10_000 fee = 5_040_505

      KONG: (10_000_000 × 300_000_000) / 990_000_000
          = 3_030_303 - 10_000 fee = 3_020_303

      BOB: (10_000_000 × 100_000_000) / 990_000_000
          = 1_010_101 - 10_000 fee = 1_000_101

      ckUSDT: (10_000_000 × 150_000) / 990_000_000
          = 1_515 (too small, skip - below dust threshold)

  11. Distribute tokens (parallel):
      transfer ALEX 10_091_010 → user  ✓
      transfer ZERO 5_040_505 → user   ✓
      transfer KONG 3_020_303 → user   ✓
      transfer BOB 1_000_101 → user    ✓

  12. Return BurnResult:
      successful_transfers: [
        ("ALEX", 10_091_010),
        ("ZERO", 5_040_505),
        ("KONG", 3_020_303),
        ("BOB", 1_000_101),
      ]
      failed_transfers: []
      icpi_burned: 10_000_000
```

### Rebalancing Flow

**Hourly Timer Triggers:**
```
1. Check if rebalancing already in progress → skip if yes
2. Try to acquire GlobalOperation::Rebalancing lock
   - Fails if mints/burns active → skip cycle
   - Fails if in grace period → skip cycle
   - Success → proceed

3. Call hourly_rebalance():

   a. Check not paused

   b. Get portfolio state:
      → current_positions = [
          ALEX: $11,250 (45%),
          ZERO: $6,250 (25%),
          KONG: $5,000 (20%),
          BOB: $2,500 (10%),
        ]
      → ckusdt_balance = $15
      → total_value = $25,000

      → target_allocations (from Kong Locker TVL):
          ALEX: 48.57% → $12,142.50
          ZERO: 1.38% → $345.00
          KONG: 0.11% → $27.50
          BOB: 0.00% → $0.00

      → deviations:
          ALEX: current 45%, target 48.57%, deviation +3.57%
                usd_difference = +$892.50
                trade_size = $89.25 (10%)

          ZERO: current 25%, target 1.38%, deviation -23.62%
                usd_difference = -$5,905.00
                trade_size = $590.50 (10%)

          KONG: current 20%, target 0.11%, deviation -19.89%
                usd_difference = -$4,972.50
                trade_size = $497.25 (10%)

          BOB: current 10%, target 0.00%, deviation -10%
               usd_difference = -$2,500.00
               trade_size = $250.00 (10%)

   c. Determine action via get_rebalancing_action():
      - ckUSDT available: $15 >= $10 ✓
      - Most underweight: ALEX (+3.57%)
      - Action: Buy ALEX with $89.25

   d. Execute buy:
      buy_amount = $89.25 → 89_250_000 ckUSDT e6

      execute_swap(
        TrackedToken::ckUSDT,
        89_250_000,
        TrackedToken::ALEX,
        0.02  // 2% max slippage
      ):
        1. Approve ckUSDT for Kongswap (89_250_000)
        2. Query expected receive: 74_375_000 ALEX
        3. Call kongswap.swap():
           pay_token: "ckUSDT"
           pay_amount: 89_250_000
           pay_tx_id: None  (ICRC-2)
           receive_token: "ALEX"
           max_slippage: 0.02
        4. Validate slippage:
           expected: 74_375_000
           actual: 73_156_250
           slippage: 1.64% < 2% ✓
        5. Log success
        → Return SwapReply

   e. Record in history:
      timestamp: 1728518400
      action: Buy { ALEX, $89.25 }
      success: true
      details: "Bought 73156250 ALEX..."

   f. Update last_rebalance timestamp

4. Release GlobalOperation lock
5. Clear in-progress flag
```

**Next Hour:**
```
New state:
  ALEX: $11,339 (45.36%) - closer to target 48.57%
  ZERO: $6,250 (25%) - still overweight
  ...

Action: Buy more ALEX or start selling ZERO
Gradual rebalancing over multiple hours
```

---

## Security Features

### Reentrancy Protection

**Per-User Guards:**
```rust
let _guard = MintGuard::acquire(user)?;
// User blocked from concurrent mints
// Different users can mint simultaneously
// Guard auto-released on function exit
```

**Global Coordination:**
```rust
try_start_global_operation(GlobalOperation::Rebalancing)?;
// Blocks new mints/burns during rebalancing
// 60-second grace period between operation types
end_global_operation(GlobalOperation::Rebalancing);
```

### Emergency Pause

```rust
// Admin activates pause
set_pause(true);

// All state-changing operations check:
check_not_paused()?;  // Returns error if paused

// Blocks: mint, burn, rebalance
// Allows: queries, status checks
```

**WARNING**: Pause state is thread-local (not persisted). After upgrade, system becomes unpaused automatically!

### Admin Controls

**Authorized Principals:**
- Backend canister (for timers)
- Deployer principal

**Admin Functions:**
- `emergency_pause()` - Stop all operations
- `emergency_unpause()` - Resume operations
- `clear_all_caches()` - Force cache refresh
- `debug_rebalancing_state()` - Diagnostic info
- `perform_rebalance()` - Manual rebalance trigger

**Action Logging:**
```rust
log_admin_action("EMERGENCY_PAUSE_ACTIVATED");
// Stored in thread-local log (max 1000 entries)
// Viewable via get_admin_action_log()
```

### Rate Limiting

```rust
check_rate_limit(&format!("mint_{}", user), 1_000_000_000)?;
// 1 second minimum between operations per user
// Prevents spam and abuse
// Automatic cleanup of old entries
```

### Input Validation

**Minting:**
- Minimum: 0.1 ckUSDT (100,000 e6)
- Maximum: 100k ckUSDT (100,000,000,000 e6)
- Not anonymous principal
- Valid mint_id ownership

**Burning:**
- Minimum: 0.00011 ICPI (11,000 e8)
- Maximum: 10% of current supply
- Not anonymous principal
- Sufficient ICPI balance
- Sufficient fee approval

**Trading:**
- Pay amount > 0
- Max slippage 0-10%
- Tokens are different
- Reasonable prices ($0.000001 to $100)

### Financial Accuracy

**No Caching for Critical Queries:**
- ICPI supply - always query ledger
- Token balances - always query token canisters
- Portfolio TVL - always recalculate

**Only Safe Caching:**
- Kong Locker TVL (changes slowly, 1-hour TTL)
- Token prices (short-lived, validation)

**Atomic Operations:**
- Supply + TVL queried in parallel (minimize time gap)
- Burn uses post-burn supply (atomic)
- Minting uses pre-deposit snapshot (prevents gaming)

### Overflow Protection

**BigUint Arithmetic:**
```rust
// All critical calculations use arbitrary precision
multiply_and_divide(a, b, c)  // Uses BigUint internally
```

**Checked Math:**
```rust
total_value = total_value.checked_add(token_value)
    .ok_or(IcpiError::Overflow)?;
```

**Safe Conversions:**
```rust
let value_u64 = nat.0.to_u64()
    .ok_or_else(|| IcpiError::Overflow)?;
```

### Error Handling

**Structured Errors:**
- Each module has specific error types
- Errors include context (amounts, principals, reasons)
- No silent failures - all errors propagated

**Refund on Failure:**
```rust
// Minting: automatic refund if fails after deposit collected
handle_mint_failure(mint_id, user, amount, reason).await?;
```

**Partial Success:**
```rust
// Burning: continues even if some token transfers fail
BurnResult {
    successful_transfers: [...],
    failed_transfers: [...],
    ...
}
```

---

## Constants Reference

### Canister IDs
```rust
ICPI_CANISTER_ID: "l6lep-niaaa-aaaap-qqeda-cai"
ICPI_BACKEND_ID: "ev6xm-haaaa-aaaap-qqcza-cai"
CKUSDT_CANISTER_ID: "cngnf-vqaaa-aaaar-qag4q-cai"
KONGSWAP_BACKEND_ID: "2ipq2-uqaaa-aaaar-qailq-cai"
KONG_LOCKER_ID: "eazgb-giaaa-aaaap-qqc2q-cai"
FEE_RECIPIENT: "e454q-riaaa-aaaap-qqcyq-cai"
```

### Token Details
```rust
Tracked Tokens:
  ALEX:   ysy5f-2qaaa-aaaap-qkmmq-cai (8 decimals)
  ZERO:   b3d2q-ayaaa-aaaap-qqcfq-cai (8 decimals)
  KONG:   o7oak-iyaaa-aaaaq-aadzq-cai (8 decimals)
  BOB:    7pail-xaaaa-aaaas-aabmq-cai (8 decimals)
  ckUSDT: cngnf-vqaaa-aaaar-qag4q-cai (6 decimals)
  ICPI:   l6lep-niaaa-aaaap-qqeda-cai (8 decimals)
```

### Financial Limits
```rust
Minting:
  MIN_MINT_AMOUNT: 100_000 (0.1 ckUSDT)
  MAX_MINT_AMOUNT: 100_000_000_000 (100k ckUSDT)
  MINT_FEE_AMOUNT: 100_000 (0.1 ckUSDT)
  MINT_TIMEOUT_NANOS: 180_000_000_000 (3 minutes)

Burning:
  MIN_BURN_AMOUNT: 11_000 (0.00011 ICPI)
  MAX_BURN_PERCENT: 10.0 (10% of supply)
  BURN_FEE_BUFFER: 10_000 (transfer fee)

Supply:
  MAX_POSSIBLE_SUPPLY: 10^16 (100M ICPI with 8 decimals)

Prices:
  MIN_REASONABLE_PRICE: 0.0001
  MAX_REASONABLE_PRICE: 1_000_000.0
```

### Rebalancing
```rust
REBALANCE_INTERVAL_SECONDS: 3600 (1 hour)
MIN_DEVIATION_PERCENT: 1.0 (1% minimum to trigger)
TRADE_INTENSITY: 0.1 (trade 10% of deviation)
MAX_SLIPPAGE_PERCENT: 2.0 (2% max slippage)
MIN_TRADE_SIZE_USD: 1.0 ($1 minimum trade)
MAX_REBALANCE_HISTORY: 10 (keep last 10 records)
```

### Timing
```rust
Approvals:
  APPROVAL_EXPIRY_NANOS: 900_000_000_000 (15 minutes)

Snapshots:
  SNAPSHOT_WARNING_AGE_NANOS: 30_000_000_000 (30 seconds)
  SNAPSHOT_MAX_AGE_NANOS: 60_000_000_000 (60 seconds)

Caching:
  CACHE_DURATION_SHORT: 30 (seconds)
  CACHE_DURATION_MEDIUM: 300 (5 minutes)
  CACHE_DURATION_LONG: 3600 (1 hour)
  TVL_CACHE_DURATION_NANOS: 3_600_000_000_000 (1 hour)

Cleanup:
  MINT_CLEANUP_INTERVAL: 3600 (seconds)
  MINT_TIMEOUT: 180 (seconds)
  COMPLETED_MINT_RETENTION: 86400 (seconds / 24 hours)

Grace Periods:
  OPERATION_GRACE_PERIOD_NANOS: 60_000_000_000 (60 seconds)
```

### Validation
```rust
MAX_SUPPLY_CHANGE_RATIO: 1.1 (10% max)
MAX_PRICE_CHANGE_RATIO: 2.0 (100% max)
MAX_REASONABLE_VALUE_E6: 1_000_000_000_000 * 1_000_000 ($1 trillion)
```

---

## End of Document

This document provides a complete reference of the ICPI backend architecture as of 2025-10-09. For implementation details, refer to the actual Rust source code in the repository.

**Version**: 0.1.0
**Last Updated**: 2025-10-09
**Purpose**: Complete functionality reference for offline review and development
