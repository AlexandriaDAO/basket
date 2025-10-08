# ICPI Protocol Documentation

*Technical reference for the Internet Computer Portfolio Index*

---

## What is ICPI?

ICPI (Internet Computer Portfolio Index) is an ICRC-1 token that provides direct, redeemable ownership of a basket of Internet Computer tokens. Unlike traditional ETFs that charge management fees on paper representations requiring proof-of-reserves, ICPI maintains 1:1 backing that's always verifiable on-chain - you can redeem your ICPI tokens for the underlying assets at any time. The index automatically tracks and rebalances to match the dollar value distribution of locked liquidity in Kongswap pools.

## Fees

**No management fees.** Unlike traditional ETFs, ICPI charges no ongoing management fees or expenses.

**Operation Fee (0.1 ckUSDT):**
- Charged on every mint and burn operation
- Goes directly to ALEX token stakers (recipient: `e454q-riaaa-aaaap-qqcyq-cai`)
- Equivalent to ~$0.10 USD per transaction

**Token Transfer Fees:**
- When burning ICPI, you receive the underlying tokens minus standard ICRC-1 transfer fees
- Typical transfer fee: 0.0001 tokens (10,000 e8s)
- Dollar value varies by token price, but generally negligible (fractions of a cent)
- Fee paid to each token's respective ledger canister

**Example:** Burning 10 ICPI that represents holdings in 4 tokens means you pay 4 transfer fees (one per token) plus the 0.1 ckUSDT operation fee. Total cost: ~$0.10-0.12.

## Mint Price

The price per ICPI is simply:

```
price = current_tvl_usd / total_supply
```

Where `current_tvl_usd` is the dollar value of all tokens held by the canister (ckUSDT + ALEX + ZERO + KONG + BOB).

When you deposit ckUSDT to mint ICPI, you receive:

```
new_icpi = usdt_deposit / price
```

This ensures exact proportional ownership - if you contribute 10% of the total value, you receive 10% of the new supply.

### Example

**Given:**
- Current TVL: $1,000 (all canister holdings)
- Current Supply: 100 ICPI
- Price: $1,000 / 100 = $10 per ICPI

**You deposit:** $100 ckUSDT

**You receive:** $100 / $10 = **10 ICPI**

You now own 10 out of 110 total ICPI (9.09% of supply), representing exactly 9.09% of all index holdings.

**Note:** Minimum mint is 1 ckUSDT. If TVL calculation fails or is zero, deposit is refunded (0.1 ckUSDT fee kept).

## Burn Price

### Inverse Proportional Formula

Burning uses the **inverse proportional formula**:

```
token_redemption = (burn_amount / total_supply) × token_balance
```

Burning 1% of ICPI supply returns exactly 1% of each token holding.

### Redemption Process (burning.rs)

1. User initiates burn with ICPI amount
2. Fee collected (0.1 ckUSDT)
3. For each token (ALEX, ZERO, KONG, BOB, ckUSDT):
   - Calculate proportional share
   - Transfer tokens to user (skips amounts ≤ transfer_fee + 1,000 units, i.e., ≤11,000 e8s)
4. ICPI burned from user balance (timeout: 180 seconds)
5. Returns list of successful/failed token transfers

### Example: Burning 10% of Supply

**Given:**
- Total Supply: 100 ICPI
- User Burns: 10 ICPI (10%)
- Index Holdings:
  - 1,000 ALEX
  - 500 ZERO
  - 200 KONG
  - 50 BOB

**User Receives (10% of each):**
- 100 ALEX
- 50 ZERO
- 20 KONG
- 5 BOB

## Rebalancing

### Hourly Automated Rebalancing (rebalancer.rs)

**Hourly automated rebalancing** via ic-cdk-timers (one trade per hour):

- **Deviation Threshold:** Only triggers if any token deviates >1% from target allocation
- **If ckUSDT available (>$10):** Buy token with largest deficit, trade size = 10% of deficit
- **Else if tokens over-allocated:** Sell most overweight token to ckUSDT, trade size = 10% of excess
- **Slippage Protection:** All swaps enforce 2% maximum slippage
- All trades go through ckUSDT as intermediary
- Sequential execution only (no batching) - required by Kongswap design

### Target Allocation Calculation (tvl_calculator.rs)

```
target% = token_locked_liquidity / total_locked_liquidity
```

1. Query kong_locker_backend for all lock canisters
2. Query each lock canister via kongswap_backend user_balances for LP positions
3. For each LP position: 50/50 pools attribute half of USD value to each token
4. Only count [ALEX, ZERO, KONG, BOB] tokens
5. Calculate percentages: Each token's TVL / Total TVL × 100%
6. Cache for 5 minutes to reduce inter-canister calls

### Example Distribution

**From project docs (example locked TVL):**
- ALEX: 48.57% ($22.5K locked)
- ZERO: 1.38% ($640)
- KONG: 0.11% ($48.71)
- BOB: 0.00% ($2.05)

## Index Weighting

**Dynamic weighting based on locked liquidity:**

Index weights are determined by the dollar value of locked liquidity in Kongswap pools. The system queries kong_locker to find all lock canisters, then queries each for their LP positions.

For each 50/50 liquidity pool, half the USD value is attributed to each token. Only [ALEX, ZERO, KONG, BOB] tokens are tracked.

Percentages are calculated as: Each token's TVL / Total TVL × 100%

This data is cached for 5 minutes to optimize performance.

## Key Features & Design Principles

### Safety-First Design

- No persistent storage for balances - queries actual token canisters
- In-memory caching with 5-minute TTL
- Refunds on calculation failures
- Two-phase mint/burn (initiate → complete)

### Gas Efficiency

- Parallel inter-canister calls (futures::join_all)
- Operation timeouts: 60 seconds (mint), 180 seconds (burn)
- Cleanup timers for expired operations

### Price Discovery

- All prices queried from Kongswap swap_amounts endpoint in real-time
- Portfolio value: Current canister holdings (not locked liquidity) determine mint/burn calculations

---

**Disclaimer:** This documentation describes the technical implementation of the ICPI protocol. It is not financial advice. Digital assets carry risk.

*Protocol Version: 1.0.1 | Last Updated: 2025-10-02*
