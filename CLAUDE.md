This project is the "Internet Computer Portfolio Index" (ICPI). It's a token that represents a basket of assets that are kept proportional to the dollar value of locked liquidity that the token has.

## Three Canisters

1. **ICPI Token** (`l6lep-niaaa-aaaap-qqeda-cai`) - Standard ICRC-1 ledger, stores all ICPI balances
2. **Backend** (`ev6xm-haaaa-aaaap-qqcza-cai`) - Business logic: minting, burning, rebalancing. Calls token ledger to mint/burn
3. **Frontend** (`qhlmp-5aaaa-aaaam-qd4jq-cai`) - React UI

Backend is minting authority for the token. Query ICPI balances from the token canister, not the backend.

**CRITICAL: Backend as Minting/Burning Account**
The backend canister (`ev6xm-haaaa-aaaap-qqcza-cai`) is the minting and burning account for ICPI, which carries heavy responsibility:
- **Burning:** Any ICPI tokens transferred TO the backend are automatically removed from circulation (burned). Users transfer ICPI directly to backend via `icrc1_transfer`.
- **Minting:** The backend can mint any amount of ICPI by calling the ledger. Only authorized minting functions should exist.
- **Security implication:** Protect minting functions carefully. Any bug allowing unauthorized mints or miscalculated burns directly affects token supply and user funds.

The amount of 'Locked Liquidity' I'm referencing is queried from the kong_locker project (../kong_locker/). Query kong_locker_backend (eazgb-giaaa-aaaap-qqc2q-cai) for lock canisters, then kongswap_backend (2ipq2-uqaaa-aaaar-qailq-cai) for user_balances. Filter results for [ALEX, ZERO, KONG, BOB] tokens.

The amount included in this index is limited to approved tokens right now, e.g., right now the distribution looks like this:

 ```

Token Distribution (7 tokens)

Token	TVL Contribution	Token Value	% of Total	LP Pools	Distribution

⭐ ALEX

	$22.5K	$11.2K	48.57%	2	

⭐ ICP

	$11.9K	$6.0K	25.74%	4	

⭐ ckUSDT

	$11.2K	$5.6K	24.20%	2	

⚡ ZERO

	$640.89	$320.45	1.38%	1	

🔥 KONG

	$48.71	$24.35	0.11%	1	

💎 BOB

	$2.05	$1.02	0.00%	1	

💎 dvinity

	$0.00	$0.00	0.00%	1	

```

But for now the index is only going to track [ALEX, ZERO, KONG, BOB]. So from this we'll divide up the percentage allocation of each.

ICPI is an ICRC1 token that can be minted with ckUSDT. Minting formula: `new_icpi = (usdt_amount * current_supply) / current_tvl`. This ensures depositors get exact proportional ownership - contribute 9.09% of value, get 9.09% of tokens. Initial mint: 1 ICPI = 1 USDT when supply is zero. Minted ckUSDT is held until rebalancing triggers.

Rebalancing happens every hour via ic-cdk-timers (one trade per hour):
- If ckUSDT available (>$10): buy token with largest deficit, trade size = 10% of deficit
- Else if tokens over-allocated: sell most overweight token to ckUSDT, trade size = 10% of excess
- All trades go through ckUSDT as intermediary
- Sequential execution only (no batching) - required by Kongswap design

Redemption burns ICPI and returns proportional tokens - burn 1% of ICPI supply, receive 1% of each token holding.

Some guiding principals:

We shouldn't use stable structures or persistent storage anywhere unless absolutely necessary because we're working with real tokens and real balances and it's much safer if we just query token balances when needed so it's always perfectly accurate.

Kongswap's pools always denominate liquidity in 50/50 pools that either denominate in ckUSDT or ICP, which is great because we know locked liquidity dollar values are always pegged to a 50/50 ratio of hard assets as a even standard. It's also great because we make all trades with ckUSDT and auto-routing is well handled. 

We should never guess types. Reference ./kong-reference/ for Kongswap source code and ../kong_locker/ for lock canister details. Always test with dfx commands first before implementing.

Development workflow - always deploy and test on mainnet:
- **CRITICAL: ALWAYS deploy to mainnet after ANY frontend changes** - run `./deploy.sh --network ic` immediately after updating frontend files (docs, UI, components, etc.)
- After implementing backend features, deploy with: `./deploy.sh --network ic`
- Test all functions directly on mainnet using dfx commands
- No need for local testing - we're experimenting with small amounts
- This enables real integration testing with kong_locker and Kongswap
- Frontend changes are NOT visible until deployed - local changes don't affect the live site

Debugging principles:
- Candid .did files must match Rust structs exactly (field names, types) - test with `dfx canister call --network ic` to catch deserialization errors before deploying frontend
- Frontend must unwrap Result types: `if ('Ok' in result) { const data = result.Ok }` - never access fields on the Result wrapper directly
- Parallelize independent inter-canister calls with `futures::future::join_all` - never await in loops
- When debugging hangs/failures, add logging at each step to isolate the exact failure point instead of guessing

## Canister IDs (Mainnet)

The ICPI project consists of three canisters on the Internet Computer mainnet:

1. **icpi_frontend** (User Interface)
   - Canister ID: `qhlmp-5aaaa-aaaam-qd4jq-cai`
   - Purpose: React frontend for interacting with ICPI
   - URL: https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io

2. **icpi_backend** (Business Logic)
   - Canister ID: `ev6xm-haaaa-aaaap-qqcza-cai`
   - Purpose: Minting, burning, rebalancing, TVL calculation
   - Acts as minting authority for ICPI token
   - Manages portfolio rebalancing via Kongswap

3. **ICPI** (Token Ledger)
   - Canister ID: `l6lep-niaaa-aaaap-qqeda-cai`
   - Purpose: Standard ICRC-1 token ledger
   - Symbol: ICPI
   - Decimals: 8 (not specified, defaults to standard)
   - Minting Authority: icpi_backend (`ev6xm-haaaa-aaaap-qqcza-cai`)
   - Located on same subnet as backend (using --next-to flag)

**Architecture:**
```
User → Frontend (qhlmp...) → Backend (ev6xm...) → ICPI Ledger (l6lep...)
                                  ↓
                          Kongswap/Kong Locker
```

The backend is the ONLY canister that should interact with the ICPI token ledger for minting/burning operations. All other token operations (transfers, balance queries) go directly to the ICPI ledger following ICRC-1 standards.
