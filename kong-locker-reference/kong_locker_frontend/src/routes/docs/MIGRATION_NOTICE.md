# Kong Locker API Migration Notice

## 🚨 Important Architecture Change

As of the latest update, Kong Locker has been refactored to remove all inter-canister calls to KongSwap. This change provides:

- **Better Performance**: Direct query calls instead of update calls
- **Lower Costs**: No cycles consumed for balance queries  
- **More Detail**: Full LP pool breakdowns instead of just USD totals
- **Simpler Backend**: Kong Locker now only manages user→canister mappings

## Removed Functions

The following functions have been removed from the Kong Locker backend:

- ❌ `get_voting_power(principal)` 
- ❌ `get_all_voting_powers()`
- ❌ `get_total_value_locked()`

## New Query Pattern

Instead of calling Kong Locker for balance data, query KongSwap directly:

```typescript
import { kongSwapService } from './kongSwapDirect';
import { lpLockingService } from './lpLocking';

// Step 1: Get lock canister from Kong Locker
const lockCanister = await lpLockingService.fetchUserLockCanister();

// Step 2: Query KongSwap directly for detailed LP positions
const positions = await kongSwapService.getLPPositions(lockCanister);

// Step 3: Process the detailed data
positions.forEach(position => {
  console.log(`Pool: ${position.symbol}`);
  console.log(`  ${position.symbol_0}: ${position.amount_0} ($${position.usd_amount_0})`);
  console.log(`  ${position.symbol_1}: ${position.amount_1} ($${position.usd_amount_1})`);
  console.log(`  Total Value: $${position.usd_balance}`);
});

// Step 4: Calculate voting power client-side
const votingPower = kongSwapService.calculateVotingPower(positions);
console.log(`Voting Power: ${votingPower} cents`);
```

## Migration Examples

### Old Way (Inter-canister calls)
```typescript
// ❌ OLD - Required update call, no pool details
const votingPower = await kongLocker.get_voting_power(userPrincipal);
// Returns: { Ok: 125050n } // Just total in cents
```

### New Way (Direct queries)
```typescript
// ✅ NEW - Query call, full pool breakdown
const lockCanister = await lpLockingService.fetchUserLockCanister();
const positions = await kongSwapService.getLPPositions(lockCanister);
// Returns detailed array:
[
  {
    symbol: "ICP_ckUSDT",
    balance: 100.5,
    usd_balance: 2500.75,
    symbol_0: "ICP",
    amount_0: 50.25,
    usd_amount_0: 1250.50,
    symbol_1: "ckUSDT", 
    amount_1: 1250.25,
    usd_amount_1: 1250.25,
    // ... more fields
  }
]
```

## Batch Operations

For operations involving multiple users:

```typescript
// Get all lock canisters from Kong Locker
const allCanisters = await lpLockingService.fetchAllLockCanisters();

// Query all positions in parallel (much faster!)
const allVotingPowers = await kongSwapService.getAllVotingPowers(allCanisters);

// Calculate total value locked
const canisterPrincipals = allCanisters.map(([_, c]) => c);
const totalValueCents = await kongSwapService.getTotalValueLocked(canisterPrincipals);
```

## Available Kong Locker Functions

Kong Locker now provides these simple query functions:

| Function | Type | Purpose |
|----------|------|---------|
| `get_my_lock_canister()` | Query | Get caller's lock canister |
| `get_all_lock_canisters()` | Query | Get all user→canister mappings |
| `get_total_positions_count()` | Query | Count of unique positions |
| `get_detailed_canister_status()` | Update | Canister health info |
| `create_lock_canister()` | Update | Create new lock |
| `complete_my_canister_setup()` | Update | Fix partial setup |

## KongSwap Direct Service

The new `kongSwapDirect.ts` service provides:

```typescript
class KongSwapDirectService {
  // Get detailed LP positions for any lock canister
  getLPPositions(lockCanister: Principal): Promise<LPReply[]>
  
  // Calculate voting power from positions
  calculateVotingPower(positions: LPReply[]): number
  
  // Batch operations
  getAllVotingPowers(canisters: [Principal, Principal][]): Promise<[Principal, number][]>
  getTotalValueLocked(canisters: Principal[]): Promise<number>
}
```

## Benefits Summary

| Metric | Before | After |
|--------|--------|-------|
| Query Type | Update calls | Query calls |
| Cost | ~13M cycles each | Free |
| Speed | 2-5 seconds | <500ms |
| Data | USD totals only | Full pool details |
| Parallel | No | Yes |

## Questions?

For help with the migration, check:
- `src/lib/services/kongSwapDirect.ts` - Direct KongSwap service
- `src/lib/services/lpLocking.ts` - Updated Kong Locker service
- This documentation page for examples