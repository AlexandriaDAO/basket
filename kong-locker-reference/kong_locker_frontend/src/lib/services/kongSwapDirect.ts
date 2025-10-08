import { HttpAgent, Actor } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';

// KongSwap canister ID
const KONGSWAP_BACKEND_ID = '2ipq2-uqaaa-aaaar-qailq-cai';

// Types matching Kong Locker backend (preserve these!)
export interface LPReply {
  name: string;           // "ICP/ckUSDT LP Token"
  symbol: string;         // "ICP_ckUSDT"
  lp_token_id: bigint;   // LP token ID
  balance: number;        // LP token balance
  usd_balance: number;    // Total USD value
  chain_0: string;        // "IC"
  symbol_0: string;       // "ICP"
  address_0: string;      // "ryjl3-tyaaa-aaaaa-aaaba-cai"
  amount_0: number;       // 50.25
  usd_amount_0: number;   // 1250.50
  chain_1: string;        // "IC"
  symbol_1: string;       // "ckUSDT"
  address_1: string;      // Token canister
  amount_1: number;       // Amount of second token
  usd_amount_1: number;   // USD value of second token
  ts: bigint;            // Timestamp
}

export type UserBalancesReply = { 'LP': LPReply };

// KongSwap IDL factory
const kongSwapIdl = ({ IDL }: any) => {
  const LPReply = IDL.Record({
    'name': IDL.Text,
    'symbol': IDL.Text,
    'lp_token_id': IDL.Nat64,
    'balance': IDL.Float64,
    'usd_balance': IDL.Float64,
    'chain_0': IDL.Text,
    'symbol_0': IDL.Text,
    'address_0': IDL.Text,
    'amount_0': IDL.Float64,
    'usd_amount_0': IDL.Float64,
    'chain_1': IDL.Text,
    'symbol_1': IDL.Text,
    'address_1': IDL.Text,
    'amount_1': IDL.Float64,
    'usd_amount_1': IDL.Float64,
    'ts': IDL.Nat64,
  });
  
  const UserBalancesReply = IDL.Variant({ 'LP': LPReply });
  
  return IDL.Service({
    'user_balances': IDL.Func([IDL.Text], [IDL.Variant({
      'Ok': IDL.Vec(UserBalancesReply),
      'Err': IDL.Text
    })], ['query']),
  });
};

// Direct KongSwap query functions
export class KongSwapDirectService {
  private actor: any;
  
  constructor() {
    const host = 'https://icp0.io';
    const agent = new HttpAgent({ host });
    
    this.actor = Actor.createActor(kongSwapIdl, {
      agent,
      canisterId: KONGSWAP_BACKEND_ID,
    });
  }
  
  // Get detailed LP positions for a lock canister
  async getLPPositions(lockCanisterPrincipal: Principal): Promise<LPReply[]> {
    try {
      const result = await this.actor.user_balances(lockCanisterPrincipal.toText());
      
      if ('Ok' in result) {
        return result.Ok.map((reply: UserBalancesReply) => reply.LP);
      } else if (result.Err.includes("User not found")) {
        return []; // Not registered yet
      } else {
        throw new Error(result.Err);
      }
    } catch (error) {
      console.error('Failed to fetch LP positions:', error);
      throw error;
    }
  }
  
  // Calculate voting power from LP positions (client-side)
  calculateVotingPower(positions: LPReply[]): number {
    const totalUSD = positions.reduce((sum, pos) => sum + pos.usd_balance, 0);
    return Math.floor(totalUSD * 100); // Convert to cents
  }
  
  // Get all voting powers (batch query)
  async getAllVotingPowers(lockCanisters: Array<[Principal, Principal]>): Promise<Array<[Principal, number]>> {
    const promises = lockCanisters.map(async ([user, canister]) => {
      try {
        const positions = await this.getLPPositions(canister);
        const votingPower = this.calculateVotingPower(positions);
        return [user, votingPower] as [Principal, number];
      } catch {
        return [user, 0] as [Principal, number];
      }
    });
    
    return Promise.all(promises);
  }
  
  // Calculate total value locked
  async getTotalValueLocked(lockCanisters: Principal[]): Promise<number> {
    const promises = lockCanisters.map(canister => this.getLPPositions(canister));
    const allPositions = await Promise.all(promises);
    
    const totalUSD = allPositions.flat().reduce((sum, pos) => sum + pos.usd_balance, 0);
    return Math.floor(totalUSD * 100); // Convert to cents
  }
  
  // Get detailed breakdown for a single user
  async getUserLPBreakdown(userPrincipal: Principal, kongLocker: any): Promise<{
    lockCanister: Principal | null;
    positions: LPReply[];
    totalUSD: number;
    votingPower: number;
  }> {
    // First get the lock canister from Kong Locker
    const lockCanisterResult = await kongLocker.get_my_lock_canister();
    
    if (!lockCanisterResult[0]) {
      return {
        lockCanister: null,
        positions: [],
        totalUSD: 0,
        votingPower: 0
      };
    }
    
    const lockCanister = lockCanisterResult[0];
    const positions = await this.getLPPositions(lockCanister);
    const totalUSD = positions.reduce((sum, pos) => sum + pos.usd_balance, 0);
    const votingPower = this.calculateVotingPower(positions);
    
    return {
      lockCanister,
      positions,
      totalUSD,
      votingPower
    };
  }
}

// Export singleton instance for convenience
export const kongSwapService = new KongSwapDirectService();