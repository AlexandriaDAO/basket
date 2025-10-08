# Frontend Refactor - BLOCKED by ic-use-internet-identity

## 🔴 Critical Runtime Error

**Error**: `TypeError: e.getTimeDiffMsecs is not a function`
**Library**: ic-use-internet-identity
**Impact**: Login completely broken, page crashes on load
**Action Taken**: ✅ Rolled back to working version

---

## What We Tried

### Goal
Fix 20-second delays and missing data by using ic-use-actor pattern from alex_frontend.

### Implementation
- ✅ Installed ic-use-actor and ic-use-internet-identity
- ✅ Created actor hooks (useICPIBackend, useICPIToken)
- ✅ Refactored all 15 hooks in useICPI.ts
- ✅ Refactored App.tsx with InternetIdentityProvider
- ✅ Build succeeded locally
- ❌ **Runtime error on mainnet** - library incompatible

### Evidence
```javascript
// Console error on login button click:
TypeError: e.getTimeDiffMsecs is not a function
    at https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io/assets/index-57626d14.js:1
```

---

## Why alex_frontend Pattern Doesn't Work Here

**Theory**: Library/build incompatibility
- alex_frontend uses different build tooling
- May use different library versions
- Vite vs Webpack configuration differences
- The ic-use-internet-identity library has undocumented requirements

**Investigation Needed**:
1. Check exact alex_frontend library versions
2. Check if alex_frontend uses custom build config
3. Test if ic-use-actor works WITHOUT ic-use-internet-identity

---

## Alternative Approaches to Fix 20-Second Delays

### Option 1: Manual Actor with Better Initialization ⭐ RECOMMENDED
Keep current auth pattern but fix the race condition:

```typescript
function AppContent() {
  const [actor, setActor] = useState<Actor | null>(null);
  const [agent, setAgent] = useState<HttpAgent | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);

  useEffect(() => {
    if (isAuthenticated && identity && !isInitialized) {
      const newAgent = createAgent(identity);
      const newActor = createActor(identity);
      setAgent(newAgent);
      setActor(newActor);
      setIsInitialized(true);  // ✅ Mark as initialized
    }
  }, [isAuthenticated, identity, isInitialized]);

  // ✅ Don't render hooks until initialized
  if (!isInitialized || !actor || !agent) {
    return <FullPageSkeleton />;
  }

  // Now hooks fire ONCE with valid actor, not repeatedly with null
  const { data: indexState } = useIndexState(actor);
  // ...
}
```

**Pros**: No new dependencies, minimal changes
**Cons**: Still uses prop drilling

### Option 2: Use ic-use-actor WITHOUT ic-use-internet-identity
Keep manual auth, use ic-use-actor for backend only:

```typescript
// Keep current AuthClient setup
const [identity, setIdentity] = useState<Identity | null>(null);

// Use ic-use-actor for backend (provide identity manually)
const backend = useICPIBackend({ agentOptions: { identity } });

// Hooks use backend actor internally
const { data: indexState } = useIndexState();  // Uses useICPIBackend() internally
```

**Pros**: Cleaner hook API, better actor lifecycle
**Cons**: Need to configure ic-use-actor with manual identity

### Option 3: Context-Based Actor Management
Create ActorContext to avoid prop drilling:

```typescript
// ActorContext.tsx
export const ActorProvider = ({ identity, children }) => {
  const actor = useMemo(() => createActor(identity), [identity]);
  const agent = useMemo(() => createAgent(identity), [identity]);

  return (
    <ActorContext.Provider value={{ actor, agent }}>
      {children}
    </ActorContext.Provider>
  );
};

// useICPI.ts
export const useIndexState = () => {
  const { actor } = useContext(ActorContext);  // ✅ No props needed
  return useQuery({ ... });
};
```

**Pros**: Clean hooks, no prop drilling, simple
**Cons**: Custom solution, not using proven library

### Option 4: Lazy Query Initialization
Make queries truly lazy until actor ready:

```typescript
export const useIndexState = (actor: Actor | null) => {
  return useQuery({
    queryKey: [QUERY_KEYS.INDEX_STATE],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      return await actor.get_index_state_cached()
    },
    enabled: !!actor,
    // ✅ Add this to prevent automatic refetch storms
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  })
}
```

**Pros**: Minimal changes
**Cons**: Doesn't fully solve initialization race

---

## Current Status

### Working on Mainnet ✅
- Original version deployed
- Login functional
- All features working (with 20-second delays)
- URL: https://qhlmp-5aaaa-aaaam-qd4jq-cai.icp0.io

### Backend API Fixes ✅ (KEPT)
These fixes from commit `25a05ea` are still in place and working:
- ✅ Added `get_index_state_cached` method
- ✅ Added 15+ missing API endpoints
- ✅ Fixed .did file

### Refactor Attempt 🔴 (ROLLED BACK)
- Code exists in `*-broken.ts` files
- Not deployed
- Blocked by library compatibility

---

## Recommendation

**For Now**: Accept the 20-second delays and fix incrementally with **Option 1** (manual actor with better initialization).

**For Long Term**: Investigate why ic-use-internet-identity works in alex_frontend but not here, or use **Option 3** (Context-based solution).

The refactored code is preserved in `*-broken.ts` files for future reference.

---

## Files Status

### Deployed (Working)
- `src/icpi_frontend/src/App.tsx` - Original version
- `src/icpi_frontend/src/hooks/useICPI.ts` - Original version

### Preserved (Not Deployed)
- `src/icpi_frontend/src/App-broken.tsx` - Refactored version (has error)
- `src/icpi_frontend/src/hooks/useICPI-broken.ts` - Refactored version
- `src/icpi_frontend/src/hooks/actors/*` - Actor hooks (unused)

### Dependencies Added
- ic-use-actor (installed but not used)
- ic-use-internet-identity (installed but broken)
- Can be removed if not pursuing this approach

---

**Next Steps**: Choose one of the 4 alternative approaches above and implement it.
