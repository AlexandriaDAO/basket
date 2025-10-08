import { HttpAgent, Actor } from '@dfinity/agent';
import type { Identity } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { writable } from 'svelte/store';
import { kongSwapService } from './kongSwapDirect';

// LP Locking Canister ID
const LP_LOCKING_CANISTER_ID = 'eazgb-giaaa-aaaap-qqc2q-cai';

// IDL for LP Locking canister (updated to match new candid)
const lpLockingIdl = ({ IDL }: any) => {
  const DetailedCanisterStatus = IDL.Record({
    'memory_size': IDL.Nat,
    'is_blackholed': IDL.Bool,
    'canister_id': IDL.Principal,
    'controller_count': IDL.Nat32,
    'cycle_balance': IDL.Nat,
    'module_hash': IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  
  return IDL.Service({
    // Query calls
    'get_total_positions_count': IDL.Func([], [IDL.Nat64], ['query']),
    'get_my_lock_canister': IDL.Func([], [IDL.Opt(IDL.Principal)], ['query']),
    'get_all_lock_canisters': IDL.Func([], [IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Principal))], ['query']),
    
    // Update calls for canister management
    'create_lock_canister': IDL.Func([], [IDL.Variant({ 'Ok': IDL.Principal, 'Err': IDL.Text })], []),
    'get_detailed_canister_status': IDL.Func([], [IDL.Variant({ 'Ok': DetailedCanisterStatus, 'Err': IDL.Text })], []),
    'complete_my_canister_setup': IDL.Func([], [IDL.Variant({ 'Ok': IDL.Text, 'Err': IDL.Text })], []),
  });
};

export interface LockingStats {
  totalPositions: number;
  totalValueLocked: number; // in USD (converted from cents)
  isLoading: boolean;
  error: string | null;
  lastUpdated: Date | null;
}

const initialStats: LockingStats = {
  totalPositions: 0,
  totalValueLocked: 0,
  isLoading: false,
  error: null,
  lastUpdated: null,
};

export const lockingStatsStore = writable<LockingStats>(initialStats);

export class LPLockingService {
  private agent: HttpAgent;
  private actor: any;

  constructor(identity?: Identity) {
    const host = 'https://icp0.io';
    this.agent = new HttpAgent({
      identity,
      host,
    });

    this.actor = Actor.createActor(lpLockingIdl, {
      agent: this.agent,
      canisterId: LP_LOCKING_CANISTER_ID,
    });
  }

  async fetchStats(): Promise<LockingStats> {
    lockingStatsStore.update(state => ({ ...state, isLoading: true, error: null }));

    try {
      // Fetch total positions (this is a query call)
      const positionsCount = await this.actor.get_total_positions_count();
      
      // Fetch all lock canisters
      const lockCanisters = await this.actor.get_all_lock_canisters();
      
      // Use KongSwap direct service to calculate TVL
      const canisterPrincipals = lockCanisters.map(([_, canister]) => canister);
      const totalValueCents = await kongSwapService.getTotalValueLocked(canisterPrincipals);
      const totalValueUsd = totalValueCents / 100;

      const stats: LockingStats = {
        totalPositions: Number(positionsCount),
        totalValueLocked: totalValueUsd,
        isLoading: false,
        error: null,
        lastUpdated: new Date(),
      };

      lockingStatsStore.set(stats);
      return stats;
    } catch (error) {
      console.error('Error fetching locking stats:', error);
      const errorMessage = error instanceof Error ? error.message : 'Failed to fetch locking statistics';
      
      const errorStats: LockingStats = {
        ...initialStats,
        isLoading: false,
        error: errorMessage,
      };

      lockingStatsStore.set(errorStats);
      return errorStats;
    }
  }

  async fetchUserLockCanister(userPrincipal?: Principal): Promise<Principal | null> {
    try {
      const result = await this.actor.get_my_lock_canister();
      return result && result[0] ? result[0] : null;
    } catch (error) {
      console.error('Error fetching user lock canister:', error);
      return null;
    }
  }

  async fetchAllLockCanisters(): Promise<Array<[Principal, Principal]>> {
    try {
      return await this.actor.get_all_lock_canisters();
    } catch (error) {
      console.error('Error fetching all lock canisters:', error);
      return [];
    }
  }

  // NEW: Fetch voting power using direct KongSwap query
  async fetchVotingPower(userPrincipal: Principal): Promise<number> {
    try {
      // First get the user's lock canister
      const lockCanisters = await this.fetchAllLockCanisters();
      const userCanister = lockCanisters.find(([user, _]) => user.toText() === userPrincipal.toText());
      
      if (!userCanister) {
        return 0;
      }
      
      // Query KongSwap directly for LP positions
      const positions = await kongSwapService.getLPPositions(userCanister[1]);
      return kongSwapService.calculateVotingPower(positions);
    } catch (error) {
      console.error('Error fetching voting power:', error);
      return 0;
    }
  }

  // NEW: Get detailed LP positions for a user
  async fetchUserLPPositions(userPrincipal?: Principal) {
    try {
      const lockCanister = await this.fetchUserLockCanister(userPrincipal);
      if (!lockCanister) {
        return {
          lockCanister: null,
          positions: [],
          totalUSD: 0,
          votingPower: 0
        };
      }
      
      const positions = await kongSwapService.getLPPositions(lockCanister);
      const totalUSD = positions.reduce((sum, pos) => sum + pos.usd_balance, 0);
      const votingPower = kongSwapService.calculateVotingPower(positions);
      
      return {
        lockCanister,
        positions,
        totalUSD,
        votingPower
      };
    } catch (error) {
      console.error('Error fetching user LP positions:', error);
      throw error;
    }
  }

  async fetchDetailedCanisterStatus(): Promise<any> {
    try {
      const result = await this.actor.get_detailed_canister_status();
      return result;
    } catch (error) {
      console.error('Error fetching detailed canister status:', error);
      throw error;
    }
  }

  async completeMyCanisterSetup(): Promise<string> {
    try {
      const result = await this.actor.complete_my_canister_setup();
      if (result && 'Ok' in result) {
        return result.Ok;
      } else {
        throw new Error(result && 'Err' in result ? result.Err : 'Failed to complete setup');
      }
    } catch (error) {
      console.error('Error completing canister setup:', error);
      throw error;
    }
  }

  /**
   * Call register_if_funded directly on a lock canister
   * This is useful for retrying registration if it failed initially
   */
  async registerLockCanister(canisterId: Principal): Promise<string> {
    try {
      // Create an actor for the lock canister directly
      const lockCanisterIDL = ({ IDL }: any) => {
        return IDL.Service({
          register_if_funded: IDL.Func([], [IDL.Variant({ Ok: IDL.Text, Err: IDL.Text })], []),
        });
      };

      const lockActor = Actor.createActor(lockCanisterIDL, {
        agent: this.agent,
        canisterId: canisterId,
      });

      const result = await lockActor.register_if_funded();
      if (result && 'Ok' in result) {
        return result.Ok;
      } else {
        throw new Error(result && 'Err' in result ? result.Err : 'Registration failed');
      }
    } catch (error) {
      console.error('Error registering lock canister:', error);
      throw error;
    }
  }

  clearError(): void {
    lockingStatsStore.update(state => ({ ...state, error: null }));
  }

  async createLockCanister(): Promise<Principal> {
    try {
      const result = await this.actor.create_lock_canister();
      if (result && 'Ok' in result) {
        return result.Ok;
      } else {
        throw new Error(result && 'Err' in result ? result.Err : 'Failed to create lock canister');
      }
    } catch (error) {
      console.error('Error creating lock canister:', error);
      throw error;
    }
  }
  
  // Create an authenticated service for user-specific calls
  createAuthenticatedService(identity: Identity): LPLockingService {
    return new LPLockingService(identity);
  }

}

// Create a singleton instance without authentication for public stats
export const lpLockingService = new LPLockingService();

// Utility functions for formatting
export const formatUsd = (amount: number): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount);
};

export const formatCount = (count: number): string => {
  return new Intl.NumberFormat('en-US').format(count);
};