# Frontend Loading Failure - Post-Mortem & Fix Guide

## 🔴 Critical Acknowledgment of Failure

**I was wrong.** The previous agent (me) made a critical error in diagnosis and wasted effort on the wrong problem.

### What I Did Wrong

1. **Misdiagnosed the problem** - Assumed backend API methods were missing/broken
2. **Fixed the wrong thing** - Addressed PR review feedback about type mismatches in backend
3. **Failed to verify** - Tested backend with `dfx canister call` but didn't check if frontend actually loads data
4. **Got distracted** - Focused on code quality issues instead of user-facing bug
5. **Declared success prematurely** - Merged PR #7 thinking the issue was fixed

### What Actually Happened

After the merge, the user reports:
- ❌ Skeleton screens after hard refresh
- ❌ No wallet balances showing
- ❌ No portfolio allocations
- ❌ Same exact problem as the beginning
- ❌ "Your wallet is empty" even when it's not

**The frontend still doesn't work.**

---

## 🔬 Actual Root Cause Analysis

### The Real Problem: Frontend Actor Initialization Race Condition

**File**: `src/icpi_frontend/src/App.tsx`

**Lines 62-105**: The problematic flow

```typescript
// Line 62-68: Hooks are called with actor=null BEFORE actor exists
const { data: indexState, isLoading: indexLoading } = useIndexState(actor);
const { data: rebalancerStatus } = useRebalancerStatus(actor);
const { data: tvlData } = useTVLData(actor);
const { data: holdings } = useHoldings(actor);
const { data: allocations } = useAllocation(actor);
const { data: actualAllocations } = useActualAllocations(actor, icpiCanisterId, agent);
const { data: totalSupply } = useTotalSupply(actor);

// Line 75-79: Wallet balances also wait for actor
const { data: walletBalances, isLoading: balancesLoading } = useUserWalletBalances(
    actor,
    principal,
    agent
);

// Line 82-99: Auth client created AFTER hooks are called
useEffect(() => {
    AuthClient.create({...}).then(async (client) => {
      // Sets isAuthenticated later...
    });
}, []);

// Line 101-105: Actor created even LATER in second useEffect
useEffect(() => {
    if (isAuthenticated && identity) {
      createActor();  // ← Actor finally created here
    }
}, [isAuthenticated, identity]);
```

### What Happens on Page Load (Timeline)

1. **T=0ms**: Component mounts
2. **T=1ms**: All hooks fire with `actor=null`, `agent=null`, `principal=''`
3. **T=2ms**: All queries disabled because `enabled: !!actor` is false
4. **T=3ms**: Component renders `<FullPageSkeleton />` because `!indexState` (line 189)
5. **T=100ms**: First useEffect creates AuthClient
6. **T=500ms**: If authenticated, identity retrieved, `isAuthenticated=true`
7. **T=501ms**: Second useEffect fires, `createActor()` called
8. **T=502ms**: `actor` and `agent` finally set to valid values
9. **T=503ms**: All queries become enabled simultaneously
10. **T=504ms**: ALL queries fire at once (thundering herd)
11. **T=5000-20000ms**: Queries complete (update calls are slow)
12. **T=20000ms**: Data finally available, skeleton disappears

### Why Skeleton Screens Persist

**File**: `src/icpi_frontend/src/App.tsx:189-191`

```typescript
if (!indexState || indexLoading) {
    return <FullPageSkeleton />;
}
```

The page shows skeleton when:
- `indexState` is undefined (query hasn't completed)
- OR `indexLoading` is true

**Problem**: `indexState` stays undefined for 5-20 seconds because:
1. Query is disabled until `actor` exists (lines 62, enabled: !!actor)
2. When actor becomes available, query fires
3. `get_index_state_cached` is an **UPDATE call** (slow, makes consensus calls)
4. User waits 5-20 seconds staring at skeletons

### Why Wallet Balances Don't Show

**File**: `src/icpi_frontend/src/hooks/useICPI.ts:670-769`

```typescript
export const useUserWalletBalances = (
  actor: Actor | null,
  userPrincipal: string | null,
  agent: HttpAgent | null
) => {
  return useQuery({
    queryKey: [QUERY_KEYS.USER_WALLET_BALANCES, userPrincipal],
    queryFn: async () => {
      if (!actor || !userPrincipal || !agent) {  // ← All null initially
        throw new Error('Actor, principal, or agent not initialized')
      }

      // STEP 1: Get token metadata
      const tokenMetadataResult = await actor.get_token_metadata()

      // STEP 2-7: Query balances for all tokens...
      const balances = await Promise.all(balancePromises)

      return balances
    },
    enabled: !!actor && !!userPrincipal && !!agent,  // ← Disabled until all ready
    refetchInterval: 30_000,
    staleTime: 10_000,
  })
}
```

**Problem**: Hook requires `actor`, `userPrincipal`, AND `agent` to all be ready:
1. Initially all are null/empty
2. Query disabled, returns `data: undefined`
3. UI shows "Your wallet is empty" (no balances)
4. After 0.5s, all become ready simultaneously
5. Query fires, but takes 2-5s to complete (multiple inter-canister calls)
6. User sees empty wallet for 2-5 seconds minimum

### Why Portfolio Allocations Don't Show

**File**: `src/icpi_frontend/src/hooks/useICPI.ts:517-666`

```typescript
export const useActualAllocations = (
  icpiActor: Actor | null,
  icpiCanisterId: string | null,
  agent: HttpAgent | null
) => {
  return useQuery({
    queryKey: [QUERY_KEYS.ACTUAL_ALLOCATIONS, icpiCanisterId],
    queryFn: async () => {
      if (!icpiActor || !icpiCanisterId || !agent) {
        throw new Error('Actor, canisterId, or agent not initialized')
      }

      // Query each token balance...
      const trackedTokensResult = await icpiActor.get_tracked_tokens()
      // ... multiple async calls per token

      return allocations
    },
    enabled: !!icpiActor && !!icpiCanisterId && !!agent,  // ← Same issue
    refetchInterval: 2 * 60_000,
    staleTime: 60_000,
  })
}
```

**Problem**: Same pattern - waits for actor, then fires slow query.

---

## 💔 Why My Backend Fixes Didn't Help

### What I Fixed (Useless)

1. ✅ Type mismatches: `token_tvls` → `tokens`, `locked_value_usd` → `tvl_usd`
2. ✅ Hardcoded canister IDs → constants
3. ✅ Error logging in ICRC-1 endpoints
4. ✅ Removed backup files

### Why It Didn't Matter

**The backend API methods work fine.** Evidence:

```bash
# These all succeed
dfx canister call --network ic ev6xm-haaaa-aaaap-qqcza-cai get_tvl_summary
dfx canister call --network ic ev6xm-haaaa-aaaap-qqcza-cai get_token_metadata
dfx canister call --network ic ev6xm-haaaa-aaaap-qqcza-cai get_index_state_cached
```

**The problem is the FRONTEND never calls them** (or calls them too late/too slowly).

---

## 🛠️ How to Actually Fix This

### Option 1: Manual Actor with Better Initialization ⭐ RECOMMENDED

**Concept**: Don't render hooks until actor is ready.

**File**: `src/icpi_frontend/src/App.tsx`

```typescript
function AppContent() {
  const [actor, setActor] = useState<Actor | null>(null);
  const [agent, setAgent] = useState<HttpAgent | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [principal, setPrincipal] = useState<string>('');

  useEffect(() => {
    async function initialize() {
      const client = await AuthClient.create({...});
      const isAuth = await client.isAuthenticated();

      if (isAuth) {
        const identity = client.getIdentity();
        const newPrincipal = identity.getPrincipal().toString();
        const newAgent = createAgent(identity);
        const newActor = createActor(identity);

        // Set everything atomically
        setPrincipal(newPrincipal);
        setAgent(newAgent);
        setActor(newActor);
        setIsInitialized(true);  // ✅ Mark ready
      } else {
        setIsInitialized(true);  // ✅ Mark ready even if not authenticated
      }
    }
    initialize();
  }, []);

  // ✅ Don't render dashboard until initialized
  if (!isInitialized) {
    return <FullPageSkeleton />;
  }

  // ✅ For authenticated users, don't render until actor ready
  if (principal && (!actor || !agent)) {
    return <FullPageSkeleton />;
  }

  // Now hooks fire ONCE with valid actor, not multiple times with null
  return <DashboardContent actor={actor} agent={agent} principal={principal} />;
}

function DashboardContent({ actor, agent, principal }) {
  // ✅ Hooks only called when actor is guaranteed to exist
  const { data: indexState } = useIndexState(actor);
  const { data: walletBalances } = useUserWalletBalances(actor, principal, agent);
  // ... render dashboard
}
```

**Pros**:
- No new dependencies
- Minimal changes to existing code
- Hooks only fire when actor is ready
- No race conditions

**Cons**:
- Still uses prop drilling
- Still shows skeleton while waiting for actor creation (~500ms)

**Estimated Time**: 30 minutes
**Risk**: Low

---

### Option 2: Context-Based Actor Management

**Concept**: Use React Context to provide actor to all hooks without prop drilling.

**File**: `src/icpi_frontend/src/contexts/ActorContext.tsx` (new)

```typescript
import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { Actor, HttpAgent, Identity } from '@dfinity/agent';
import { AuthClient } from '@dfinity/auth-client';

interface ActorContextValue {
  actor: Actor | null;
  agent: HttpAgent | null;
  principal: string;
  isAuthenticated: boolean;
  isInitialized: boolean;
  login: () => Promise<void>;
  logout: () => Promise<void>;
}

const ActorContext = createContext<ActorContextValue | null>(null);

export const useActor = () => {
  const context = useContext(ActorContext);
  if (!context) throw new Error('useActor must be used within ActorProvider');
  return context;
};

export const ActorProvider = ({ children }: { children: ReactNode }) => {
  const [actor, setActor] = useState<Actor | null>(null);
  const [agent, setAgent] = useState<HttpAgent | null>(null);
  const [principal, setPrincipal] = useState<string>('');
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);
  const [authClient, setAuthClient] = useState<AuthClient | null>(null);

  useEffect(() => {
    async function initialize() {
      const client = await AuthClient.create({...});
      setAuthClient(client);

      const isAuth = await client.isAuthenticated();
      if (isAuth) {
        const identity = client.getIdentity();
        const newPrincipal = identity.getPrincipal().toString();
        const newAgent = createAgent(identity);
        const newActor = createActor(identity);

        setActor(newActor);
        setAgent(newAgent);
        setPrincipal(newPrincipal);
        setIsAuthenticated(true);
      }

      setIsInitialized(true);
    }
    initialize();
  }, []);

  const login = async () => { /* ... */ };
  const logout = async () => { /* ... */ };

  const value = {
    actor,
    agent,
    principal,
    isAuthenticated,
    isInitialized,
    login,
    logout,
  };

  return (
    <ActorContext.Provider value={value}>
      {children}
    </ActorContext.Provider>
  );
};
```

**Update hooks**: `src/icpi_frontend/src/hooks/useICPI.ts`

```typescript
import { useActor } from '../contexts/ActorContext';

// ✅ No more actor parameter needed
export const useIndexState = () => {
  const { actor } = useActor();

  return useQuery({
    queryKey: [QUERY_KEYS.INDEX_STATE],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      return await actor.get_index_state_cached()
    },
    enabled: !!actor,
    // ...
  })
}

// ✅ All hooks updated similarly
export const useUserWalletBalances = () => {
  const { actor, agent, principal } = useActor();

  return useQuery({
    queryKey: [QUERY_KEYS.USER_WALLET_BALANCES, principal],
    queryFn: async () => {
      if (!actor || !principal || !agent) {
        throw new Error('Actor, principal, or agent not initialized')
      }
      // ... existing code
    },
    enabled: !!actor && !!principal && !!agent,
  })
}
```

**Update App**: `src/icpi_frontend/src/App.tsx`

```typescript
function App() {
  return (
    <ActorProvider>
      <QueryClientProvider client={queryClient}>
        <ErrorBoundary>
          <AppContent />
        </ErrorBoundary>
      </QueryClientProvider>
    </ActorProvider>
  );
}

function AppContent() {
  const { isInitialized, actor, agent, principal } = useActor();

  if (!isInitialized) {
    return <FullPageSkeleton />;
  }

  // ✅ No more prop drilling - hooks get actor from context
  const { data: indexState } = useIndexState();
  const { data: walletBalances } = useUserWalletBalances();

  // ... rest of component
}
```

**Pros**:
- Clean hook API (no parameters)
- No prop drilling
- Centralized actor management
- Easy to add auth state

**Cons**:
- More files to create
- More refactoring needed
- Custom solution (not using proven library)

**Estimated Time**: 2 hours
**Risk**: Medium

---

### Option 3: Fix ic-use-actor Integration

**Concept**: Debug why `ic-use-internet-identity` failed and fix it.

**Previous Error**:
```
TypeError: e.getTimeDiffMsecs is not a function
```

**Possible Causes**:
1. Library version mismatch
2. Build tooling incompatibility (Vite vs Webpack)
3. Missing peer dependencies
4. Incorrect usage

**Investigation Steps**:
1. Check alex_frontend versions:
   ```bash
   cd ../core/src/alex_frontend
   grep "ic-use-" package.json
   npm list ic-use-actor ic-use-internet-identity
   ```

2. Compare build configs:
   ```bash
   diff ../core/src/alex_frontend/vite.config.ts src/icpi_frontend/vite.config.ts
   ```

3. Try alternative auth pattern:
   ```typescript
   // Don't use InternetIdentityProvider
   // Use manual AuthClient with ic-use-actor

   const [identity, setIdentity] = useState<Identity | null>(null);

   const backend = useICPIBackend({
     agentOptions: identity ? { identity } : undefined
   });
   ```

**Pros**:
- Would match alex_frontend architecture
- Proven library pattern
- Best long-term solution

**Cons**:
- Requires debugging library incompatibility
- Time-consuming investigation
- May not be fixable

**Estimated Time**: 3-4 hours
**Risk**: High (might not work)

---

### Option 4: Progressive Enhancement (Incremental Fix)

**Concept**: Make queries work better with the existing pattern.

**Changes**:

1. **Change `get_index_state_cached` to query method** (backend fix)

   **File**: `src/icpi_backend/src/lib.rs:86-89`

   ```rust
   // Before (slow update call)
   #[update]
   #[candid_method(update)]
   async fn get_index_state_cached() -> Result<types::portfolio::IndexState>

   // After (fast query call)
   #[query]
   #[candid_method(query)]
   fn get_index_state_cached() -> Result<types::portfolio::IndexState>
   ```

2. **Add optimistic rendering** (frontend fix)

   **File**: `src/icpi_frontend/src/App.tsx:189-191`

   ```typescript
   // Show partial data instead of full skeleton
   if (!indexState) {
     return (
       <Dashboard
         portfolioData={{ portfolioValue: 0, indexPrice: 0, apy: 0 }}
         allocations={[]}
         actualAllocations={[]}
         // ... show loading states inline
       />
     );
   }
   ```

3. **Stagger query execution**

   **File**: `src/icpi_frontend/src/hooks/useICPI.ts`

   ```typescript
   // Priority 1: Fast, critical data
   export const useIndexState = (actor: Actor | null) => {
     return useQuery({
       queryKey: [QUERY_KEYS.INDEX_STATE],
       queryFn: async () => { /* ... */ },
       enabled: !!actor,
       refetchInterval: 60_000,
       staleTime: 0,  // ✅ Fetch immediately when enabled
     })
   }

   // Priority 2: Wait for indexState before fetching balances
   export const useUserWalletBalances = (actor, principal, agent) => {
     const { data: indexState } = useQuery([QUERY_KEYS.INDEX_STATE]);

     return useQuery({
       queryKey: [QUERY_KEYS.USER_WALLET_BALANCES, principal],
       queryFn: async () => { /* ... */ },
       enabled: !!actor && !!principal && !!agent && !!indexState,  // ✅ Wait for indexState
       refetchInterval: 30_000,
     })
   }
   ```

**Pros**:
- Incremental improvements
- Each change adds value
- Lower risk

**Cons**:
- Doesn't fully solve race condition
- Still has some delays
- Partial solution

**Estimated Time**: 1 hour per improvement
**Risk**: Low

---

## ✅ Proper Verification Steps

### CRITICAL: Verify Frontend, Not Backend

**Don't do this** ❌:
```bash
# Testing backend methods proves nothing about frontend
dfx canister call --network ic ev6xm-haaaa-aaaap-qqcza-cai get_index_state_cached
```

**Do this instead** ✅:

1. **Open browser dev tools** (F12)

2. **Navigate to frontend**:
   ```
   https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io
   ```

3. **Hard refresh** (Ctrl+Shift+R or Cmd+Shift+R)

4. **Check Network tab**:
   - Look for calls to `get_index_state_cached`
   - Check timing: How long does it take?
   - Check response: Is data returned?

5. **Check Console tab**:
   - Look for errors
   - Look for "Actor not initialized" warnings
   - Check React Query devtools (if enabled)

6. **Check UI**:
   - Do wallet balances appear in <5 seconds?
   - Does "Your wallet is empty" change to actual balances?
   - Do portfolio allocations show up?
   - Is TVL visible in top stats bar?

7. **Use console debugging**:
   ```javascript
   // In browser console, check React Query state
   window.__REACT_QUERY_DEVTOOLS_GLOBAL_HOOK__?.getQueryClientInstance?.()?.getQueryCache()?.getAll()

   // Or add to App.tsx:
   useEffect(() => {
     console.log('🔍 Debug State:', {
       actor: !!actor,
       agent: !!agent,
       principal,
       indexState,
       walletBalances,
       actualAllocations,
     });
   }, [actor, agent, principal, indexState, walletBalances, actualAllocations]);
   ```

### Success Criteria

**Must achieve ALL of these**:

1. ✅ After login, wallet balances visible in **<5 seconds** (not 20s)
2. ✅ Portfolio allocations visible in **<5 seconds**
3. ✅ No "Your wallet is empty" when wallet has tokens
4. ✅ No skeleton screens lasting >2 seconds
5. ✅ Console shows no "Actor not initialized" errors
6. ✅ Network tab shows `get_index_state_cached` completes in <3s

**Test both scenarios**:
- ✅ Fresh page load (hard refresh)
- ✅ Returning user (already authenticated)

---

## 📋 Implementation Checklist

### Before Starting

- [ ] Read this entire document
- [ ] Understand the actor initialization race condition
- [ ] Open browser dev tools and reproduce the issue
- [ ] Verify you can see the skeleton screens
- [ ] Check console for errors

### During Implementation

- [ ] Choose Option 1, 2, 3, or 4 (or combine)
- [ ] Make changes to frontend code
- [ ] Build frontend: `cd src/icpi_frontend && npm run build`
- [ ] Deploy to mainnet: `dfx deploy --network ic icpi_frontend`
- [ ] **VERIFY IN BROWSER** (not with dfx commands)
- [ ] Test with hard refresh
- [ ] Test with existing auth
- [ ] Check all success criteria

### After Fixing

- [ ] Document what approach was used
- [ ] Record timing improvements (before/after)
- [ ] Update this document with results
- [ ] Commit with clear message
- [ ] Create PR with frontend verification evidence

---

## 🎯 Key Takeaways

1. **Backend API methods were never the problem** - They work fine via dfx
2. **Frontend actor initialization is the problem** - Race condition causes delays
3. **Always verify in the browser** - Don't trust backend tests alone
4. **Skeleton screens = queries not completing** - Not a backend issue
5. **Fix the frontend, not the backend** - That's where the bug actually is

---

## 📚 Reference Files

### Primary Files to Modify

1. `src/icpi_frontend/src/App.tsx` - Actor initialization
2. `src/icpi_frontend/src/hooks/useICPI.ts` - Query hooks
3. (Optional) `src/icpi_backend/src/lib.rs` - Change update→query

### Files to Read for Context

1. `FRONTEND_REFACTOR_BLOCKED.md` - Previous attempt with ic-use-actor
2. `FRONTEND_REFACTOR_STATUS.md` - Full context of what was tried
3. `src/icpi_frontend/src/hooks/actors/useICPIBackend.ts` - Actor hook (unused)

### Don't Modify These

1. Backend type definitions (already fixed, not the issue)
2. `.did` files (consistent with Rust types)
3. Backend API methods (they work)

---

## 💬 What to Tell the Next Agent

> "The frontend has an actor initialization race condition. Hooks are called with `actor=null` before the actor is created, causing all queries to wait and fire simultaneously. This creates 5-20 second delays and skeleton screens. The backend API methods work fine - the problem is purely frontend actor lifecycle management. Fix the frontend, not the backend. Verify in browser, not with dfx. Success = wallet balances visible in <5 seconds after login."

---

**Status**: Ready for next agent to implement fix
**Recommended Approach**: Option 1 (Manual Actor with Better Initialization)
**Estimated Time**: 30-60 minutes
**Verification**: Must test in browser at https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io
