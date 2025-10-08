import { HttpAgent, Actor } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { kongSwapService, type LPReply } from './kongSwapDirect';
import { alexandriaUsersService } from './alexandriaUsers';

// Kong Locker canister ID
const KONG_LOCKER_ID = 'eazgb-giaaa-aaaap-qqc2q-cai';

// Types for analytics data
export interface AnalyticsOverview {
  total_lock_canisters: bigint;
  participants: Array<[Principal, Principal]>; // [user, lock_canister]
  last_updated: bigint;
}

export interface CanisterListItem {
  userPrincipal: Principal;
  lockCanister: Principal;
  userDisplay: string;        // Truncated for display
  canisterDisplay: string;    // Truncated for display
  username?: string;          // Alexandria username if available
  status: 'unknown' | 'loading' | 'active' | 'empty' | 'error';
  lastQueried?: number;       // Timestamp when last checked
  cachedPositions?: LPReply[];
  cachedValue?: number;       // USD value
  cachedVotingPower?: number;
}

export interface SystemStats {
  totalCanisters: number;
  totalParticipants: number;
  estimatedTotalValue: number;
  lastUpdated: Date;
  tokenBreakdown?: Map<string, {
    totalValue: number;
    lpPools: Set<string>;
    totalTokenAmount: number;
  }>;
}

// Cache management
interface CacheEntry {
  data: LPReply[];
  timestamp: number;
  votingPower: number;
  totalUSD: number;
}

class AnalyticsCache {
  private cache = new Map<string, CacheEntry>();
  private readonly TTL = 10 * 60 * 1000; // 10 minutes

  get(canisterId: string): CacheEntry | null {
    const entry = this.cache.get(canisterId);
    if (!entry) return null;

    if (Date.now() - entry.timestamp > this.TTL) {
      this.cache.delete(canisterId);
      return null;
    }

    return entry;
  }

  set(canisterId: string, positions: LPReply[]): void {
    const totalUSD = positions.reduce((sum, pos) => sum + pos.usd_balance, 0);
    const votingPower = Math.floor(totalUSD * 100);
    
    this.cache.set(canisterId, {
      data: positions,
      timestamp: Date.now(),
      votingPower,
      totalUSD
    });
  }

  clear(): void {
    this.cache.clear();
  }

  getStats(): { entries: number; totalSize: number } {
    return {
      entries: this.cache.size,
      totalSize: Array.from(this.cache.values()).reduce((sum, entry) => 
        sum + JSON.stringify(entry).length, 0
      )
    };
  }
  
  getAllEntries(): Map<string, CacheEntry> {
    return new Map(this.cache);
  }
}

// Kong Locker IDL for analytics
const kongLockerIdl = ({ IDL }: any) => {
  const AnalyticsOverview = IDL.Record({
    'total_lock_canisters': IDL.Nat64,
    'participants': IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Principal)),
    'last_updated': IDL.Nat64,
  });

  return IDL.Service({
    'get_analytics_overview': IDL.Func([], [AnalyticsOverview], ['query']),
  });
};

export class AnalyticsService {
  private actor: any;
  private cache = new AnalyticsCache();

  constructor() {
    const host = 'https://icp0.io';
    const agent = new HttpAgent({ host });
    
    this.actor = Actor.createActor(kongLockerIdl, {
      agent,
      canisterId: KONG_LOCKER_ID,
    });
  }

  // Get basic analytics overview (fast - single query)
  async getAnalyticsOverview(): Promise<AnalyticsOverview> {
    try {
      const result = await this.actor.get_analytics_overview();
      return result;
    } catch (error) {
      console.error('Failed to fetch analytics overview:', error);
      throw error;
    }
  }

  // Convert overview to display-friendly list
  async getCanisterList(): Promise<CanisterListItem[]> {
    const overview = await this.getAnalyticsOverview();

    // Extract all user principals
    const userPrincipals = overview.participants.map(([user, _]) => user);

    // Fetch usernames in parallel
    const usernameMap = await alexandriaUsersService.getUsernames(userPrincipals);

    return overview.participants.map(([user, lockCanister]) => {
      const cached = this.cache.get(lockCanister.toText());
      const username = usernameMap.get(user.toText());

      return {
        userPrincipal: user,
        lockCanister: lockCanister,
        userDisplay: this.truncatePrincipal(user.toText()),
        canisterDisplay: this.truncatePrincipal(lockCanister.toText()),
        username: username || undefined,
        status: cached ? 'active' : 'unknown',
        lastQueried: cached?.timestamp,
        cachedPositions: cached?.data,
        cachedValue: cached?.totalUSD,
        cachedVotingPower: cached?.votingPower
      } as CanisterListItem;
    });
  }

  // Get detailed breakdown for specific canister (slower - KongSwap query)
  async getCanisterDetails(lockCanisterId: Principal): Promise<{
    positions: LPReply[];
    totalUSD: number;
    votingPower: number;
    summary: {
      poolCount: number;
      lastUpdated: Date;
    }
  }> {
    // Check cache first
    const cached = this.cache.get(lockCanisterId.toText());
    if (cached) {
      console.log('Using cached LP positions for:', lockCanisterId.toText());
      return {
        positions: cached.data,
        totalUSD: cached.totalUSD,
        votingPower: cached.votingPower,
        summary: {
          poolCount: cached.data.length,
          lastUpdated: new Date(cached.timestamp)
        }
      };
    }

    // Query KongSwap for fresh data
    try {
      const positions = await kongSwapService.getLPPositions(lockCanisterId);
      
      // Cache the results
      this.cache.set(lockCanisterId.toText(), positions);
      
      const totalUSD = positions.reduce((sum, pos) => sum + pos.usd_balance, 0);
      const votingPower = kongSwapService.calculateVotingPower(positions);
      
      return {
        positions,
        totalUSD,
        votingPower,
        summary: {
          poolCount: positions.length,
          lastUpdated: new Date()
        }
      };
    } catch (error) {
      console.error('Failed to fetch canister details:', error);
      throw error;
    }
  }

  // Calculate system-wide statistics (load all canisters for accuracy)
  async calculateSystemStats(limit: number = 20): Promise<SystemStats> {
    const overview = await this.getAnalyticsOverview();
    const totalCanisters = Number(overview.total_lock_canisters);

    // Load ALL canisters for accurate data (no more extrapolation)
    const allCanisters = overview.participants;

    let totalValue = 0;
    let successfulQueries = 0;
    const tokenBreakdown = new Map<string, {
      totalValue: number;
      lpPools: Set<string>;
      totalTokenAmount: number;
    }>();

    // Query ALL canisters in parallel
    const queries = allCanisters.map(async ([user, canister]) => {
      try {
        const details = await this.getCanisterDetails(canister);
        return {
          totalUSD: details.totalUSD,
          positions: details.positions
        };
      } catch {
        return { totalUSD: 0, positions: [] }; // Failed queries don't contribute
      }
    });

    const results = await Promise.allSettled(queries);

    results.forEach((result) => {
      if (result.status === 'fulfilled') {
        totalValue += result.value.totalUSD;
        if (result.value.totalUSD > 0) {
          successfulQueries++;
        }

        // Aggregate token data from positions
        result.value.positions.forEach(position => {
          // Process token 0 - use actual USD value from KongSwap
          if (position.symbol_0) {
            if (!tokenBreakdown.has(position.symbol_0)) {
              tokenBreakdown.set(position.symbol_0, {
                totalValue: 0,
                lpPools: new Set(),
                totalTokenAmount: 0
              });
            }
            const token0Data = tokenBreakdown.get(position.symbol_0)!;
            // Use the actual USD amount for token 0 from KongSwap
            token0Data.totalValue += position.usd_amount_0 || 0;
            token0Data.lpPools.add(position.symbol);
            token0Data.totalTokenAmount += position.amount_0;
          }

          // Process token 1 - use actual USD value from KongSwap
          if (position.symbol_1) {
            if (!tokenBreakdown.has(position.symbol_1)) {
              tokenBreakdown.set(position.symbol_1, {
                totalValue: 0,
                lpPools: new Set(),
                totalTokenAmount: 0
              });
            }
            const token1Data = tokenBreakdown.get(position.symbol_1)!;
            // Use the actual USD amount for token 1 from KongSwap
            token1Data.totalValue += position.usd_amount_1 || 0;
            token1Data.lpPools.add(position.symbol);
            token1Data.totalTokenAmount += position.amount_1;
          }
        });
      }
    });

    // No extrapolation - use actual total value
    const actualTotalValue = totalValue;

    return {
      totalCanisters,
      totalParticipants: totalCanisters, // 1:1 mapping currently
      estimatedTotalValue: actualTotalValue,
      lastUpdated: new Date(Number(overview.last_updated) / 1_000_000), // Convert nanoseconds to milliseconds
      tokenBreakdown
    };
  }

  // Utility function to truncate principals for display
  private truncatePrincipal(principal: string): string {
    if (principal.length <= 20) return principal;
    return `${principal.substring(0, 8)}...${principal.substring(principal.length - 6)}`;
  }

  // Batch preload popular canisters (optional optimization)
  async preloadTopCanisters(limit: number = 10): Promise<void> {
    try {
      const overview = await this.getAnalyticsOverview();
      const topCanisters = overview.participants.slice(0, limit);
      
      // Load in background, don't await all
      topCanisters.forEach(async ([user, canister]) => {
        try {
          await this.getCanisterDetails(canister);
          console.log('Preloaded:', canister.toText());
        } catch (error) {
          // Silent failure for background preloading
          console.debug('Preload failed for:', canister.toText(), error);
        }
      });
    } catch (error) {
      console.warn('Failed to preload canisters:', error);
    }
  }

  // Get token breakdown from currently cached data
  getTokenBreakdownFromCache(): Map<string, {
    totalValue: number;
    lpPools: Set<string>;
    totalTokenAmount: number;
  }> {
    const tokenBreakdown = new Map<string, {
      totalValue: number;
      lpPools: Set<string>;
      totalTokenAmount: number;
    }>();
    
    // Aggregate from all cached entries
    this.cache.getAllEntries().forEach((entry) => {
      entry.data.forEach(position => {
        // Process token 0 - use actual USD value from KongSwap
        if (position.symbol_0) {
          if (!tokenBreakdown.has(position.symbol_0)) {
            tokenBreakdown.set(position.symbol_0, {
              totalValue: 0,
              lpPools: new Set(),
              totalTokenAmount: 0
            });
          }
          const token0Data = tokenBreakdown.get(position.symbol_0)!;
          // Use the actual USD amount for token 0 from KongSwap
          token0Data.totalValue += position.usd_amount_0 || 0;
          token0Data.lpPools.add(position.symbol);
          token0Data.totalTokenAmount += position.amount_0;
        }

        // Process token 1 - use actual USD value from KongSwap
        if (position.symbol_1) {
          if (!tokenBreakdown.has(position.symbol_1)) {
            tokenBreakdown.set(position.symbol_1, {
              totalValue: 0,
              lpPools: new Set(),
              totalTokenAmount: 0
            });
          }
          const token1Data = tokenBreakdown.get(position.symbol_1)!;
          // Use the actual USD amount for token 1 from KongSwap
          token1Data.totalValue += position.usd_amount_1 || 0;
          token1Data.lpPools.add(position.symbol);
          token1Data.totalTokenAmount += position.amount_1;
        }
      });
    });
    
    return tokenBreakdown;
  }

  // Cache management
  clearCache(): void {
    this.cache.clear();
  }

  getCacheStats() {
    return this.cache.getStats();
  }
}

// Export singleton instance
export const analyticsService = new AnalyticsService();