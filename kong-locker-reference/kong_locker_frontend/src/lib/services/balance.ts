import { HttpAgent, Actor } from '@dfinity/agent';
import type { Identity } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { writable } from 'svelte/store';

// ICP Ledger Canister ID
const ICP_LEDGER_CANISTER_ID = 'ryjl3-tyaaa-aaaaa-aaaba-cai';

// IDL for ICRC1 and ICRC2 (includes approve for spending)
const icrc1Idl = ({ IDL }: any) => {
  const Account = IDL.Record({
    'owner': IDL.Principal,
    'subaccount': IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  
  const TransferArgs = IDL.Record({
    'from_subaccount': IDL.Opt(IDL.Vec(IDL.Nat8)),
    'to': Account,
    'amount': IDL.Nat,
    'fee': IDL.Opt(IDL.Nat),
    'memo': IDL.Opt(IDL.Vec(IDL.Nat8)),
    'created_at_time': IDL.Opt(IDL.Nat64),
  });
  
  const ApproveArgs = IDL.Record({
    'from_subaccount': IDL.Opt(IDL.Vec(IDL.Nat8)),
    'spender': Account,
    'amount': IDL.Nat,
    'expected_allowance': IDL.Opt(IDL.Nat),
    'expires_at': IDL.Opt(IDL.Nat64),
    'fee': IDL.Opt(IDL.Nat),
    'memo': IDL.Opt(IDL.Vec(IDL.Nat8)),
    'created_at_time': IDL.Opt(IDL.Nat64),
  });
  
  const TransferError = IDL.Variant({
    'BadFee': IDL.Record({ 'expected_fee': IDL.Nat }),
    'BadBurn': IDL.Record({ 'min_burn_amount': IDL.Nat }),
    'InsufficientFunds': IDL.Record({ 'balance': IDL.Nat }),
    'TooOld': IDL.Null,
    'CreatedInFuture': IDL.Record({ 'ledger_time': IDL.Nat64 }),
    'TemporarilyUnavailable': IDL.Null,
    'Duplicate': IDL.Record({ 'duplicate_of': IDL.Nat }),
    'GenericError': IDL.Record({ 'error_code': IDL.Nat, 'message': IDL.Text }),
  });
  
  const ApproveError = IDL.Variant({
    'BadFee': IDL.Record({ 'expected_fee': IDL.Nat }),
    'InsufficientFunds': IDL.Record({ 'balance': IDL.Nat }),
    'AllowanceChanged': IDL.Record({ 'current_allowance': IDL.Nat }),
    'Expired': IDL.Record({ 'ledger_time': IDL.Nat64 }),
    'TooOld': IDL.Null,
    'CreatedInFuture': IDL.Record({ 'ledger_time': IDL.Nat64 }),
    'Duplicate': IDL.Record({ 'duplicate_of': IDL.Nat }),
    'TemporarilyUnavailable': IDL.Null,
    'GenericError': IDL.Record({ 'error_code': IDL.Nat, 'message': IDL.Text }),
  });
  
  const TransferResult = IDL.Variant({
    'Ok': IDL.Nat,
    'Err': TransferError,
  });
  
  const ApproveResult = IDL.Variant({
    'Ok': IDL.Nat,
    'Err': ApproveError,
  });
  
  return IDL.Service({
    'icrc1_balance_of': IDL.Func([Account], [IDL.Nat], ['query']),
    'icrc1_transfer': IDL.Func([TransferArgs], [TransferResult], []),
    'icrc1_fee': IDL.Func([], [IDL.Nat], ['query']),
    'icrc2_approve': IDL.Func([ApproveArgs], [ApproveResult], []),
  });
};

export interface BalanceState {
  icpBalance: bigint;
  isLoading: boolean;
  error: string | null;
  lastUpdated: Date | null;
}

const initialBalanceState: BalanceState = {
  icpBalance: 0n,
  isLoading: false,
  error: null,
  lastUpdated: null,
};

export const balanceStore = writable<BalanceState>(initialBalanceState);

export class BalanceService {
  private agent: HttpAgent | null = null;
  private identity: Identity | null = null;

  constructor(identity: Identity) {
    this.identity = identity;
    this.setupAgent();
  }

  private setupAgent(): void {
    if (!this.identity) return;

    const host = 'https://icp0.io';
    this.agent = new HttpAgent({
      identity: this.identity,
      host,
    });
  }

  async fetchIcpBalance(principal: Principal): Promise<bigint> {
    if (!this.agent) {
      throw new Error('Agent not initialized');
    }

    balanceStore.update(state => ({ ...state, isLoading: true, error: null }));

    try {
      const actor = Actor.createActor(icrc1Idl, {
        agent: this.agent,
        canisterId: ICP_LEDGER_CANISTER_ID,
      });

      const account = {
        owner: principal,
        subaccount: [],
      };

      const balance = await actor.icrc1_balance_of(account);
      
      balanceStore.update(state => ({
        ...state,
        icpBalance: balance,
        isLoading: false,
        error: null,
        lastUpdated: new Date(),
      }));

      return balance;
    } catch (error) {
      console.error('Error fetching ICP balance:', error);
      const errorMessage = error instanceof Error ? error.message : 'Failed to fetch ICP balance';
      
      balanceStore.update(state => ({
        ...state,
        isLoading: false,
        error: errorMessage,
      }));

      return 0n;
    }
  }

  async transferIcp(toPrincipal: Principal, amount: bigint, memo?: Uint8Array): Promise<bigint> {
    if (!this.agent) {
      throw new Error('Agent not initialized');
    }

    try {
      const actor = Actor.createActor(icrc1Idl, {
        agent: this.agent,
        canisterId: ICP_LEDGER_CANISTER_ID,
      });

      // Get current fee
      const fee = await actor.icrc1_fee();

      const transferArgs = {
        from_subaccount: [],
        to: {
          owner: toPrincipal,
          subaccount: [],
        },
        amount: amount,
        fee: [fee],
        memo: memo ? [Array.from(memo)] : [],
        created_at_time: [],
      };

      const result = await actor.icrc1_transfer(transferArgs);
      
      if ('Ok' in result) {
        // Refresh balance after successful transfer
        if (this.identity) {
          const userPrincipal = this.identity.getPrincipal();
          await this.fetchIcpBalance(userPrincipal);
        }
        return result.Ok;
      } else {
        // Handle transfer errors
        const error = result.Err;
        let errorMessage = 'Transfer failed';
        
        if ('InsufficientFunds' in error) {
          const balance = Number(error.InsufficientFunds.balance) / 100_000_000;
          errorMessage = `Insufficient funds. Balance: ${balance} ICP`;
        } else if ('BadFee' in error) {
          const expectedFee = Number(error.BadFee.expected_fee) / 100_000_000;
          errorMessage = `Bad fee. Expected: ${expectedFee} ICP`;
        } else if ('GenericError' in error) {
          errorMessage = error.GenericError.message;
        } else if ('TooOld' in error) {
          errorMessage = 'Transaction too old';
        } else if ('TemporarilyUnavailable' in error) {
          errorMessage = 'Ledger temporarily unavailable';
        }
        
        throw new Error(errorMessage);
      }
    } catch (error) {
      console.error('Error transferring ICP:', error);
      throw error;
    }
  }

  async sendIcp(toPrincipal: Principal, amount: bigint): Promise<bigint> {
    return this.transferIcp(toPrincipal, amount);
  }

  async getIcpBalance(principal: Principal): Promise<bigint> {
    if (!this.agent) {
      throw new Error('Agent not initialized');
    }

    try {
      const actor = Actor.createActor(icrc1Idl, {
        agent: this.agent,
        canisterId: ICP_LEDGER_CANISTER_ID,
      });

      const account = {
        owner: principal,
        subaccount: [],
      };

      const balance = await actor.icrc1_balance_of(account);
      return balance;
    } catch (error) {
      console.error('Error fetching ICP balance:', error);
      throw error;
    }
  }

  async getAlexBalance(principal: Principal): Promise<bigint> {
    if (!this.agent) {
      throw new Error('Agent not initialized');
    }

    const ALEX_CANISTER_ID = Principal.fromText('ysy5f-2qaaa-aaaap-qkmmq-cai');

    try {
      const actor = Actor.createActor(icrc1Idl, {
        agent: this.agent,
        canisterId: ALEX_CANISTER_ID,
      });

      const account = {
        owner: principal,
        subaccount: [],
      };

      const balance = await actor.icrc1_balance_of(account);
      return balance;
    } catch (error) {
      console.error('Error fetching ALEX balance:', error);
      throw error;
    }
  }

  async approveIcp(spenderPrincipal: Principal, amount: bigint): Promise<bigint> {
    if (!this.agent) {
      throw new Error('Agent not initialized');
    }

    try {
      const actor = Actor.createActor(icrc1Idl, {
        agent: this.agent,
        canisterId: ICP_LEDGER_CANISTER_ID,
      });

      // Get current fee
      const fee = await actor.icrc1_fee();

      const approveArgs = {
        from_subaccount: [],
        spender: {
          owner: spenderPrincipal,
          subaccount: [],
        },
        amount: amount,
        expected_allowance: [], // No expected allowance check
        expires_at: [], // No expiration
        fee: [fee],
        memo: [],
        created_at_time: [],
      };

      const result = await actor.icrc2_approve(approveArgs);
      
      if ('Ok' in result) {
        return result.Ok; // Returns the block index
      } else {
        // Handle approve errors
        const error = result.Err;
        let errorMessage = 'Approval failed';
        
        if ('InsufficientFunds' in error) {
          const balance = Number(error.InsufficientFunds.balance) / 100_000_000;
          errorMessage = `Insufficient funds for approval. Balance: ${balance} ICP`;
        } else if ('BadFee' in error) {
          const expectedFee = Number(error.BadFee.expected_fee) / 100_000_000;
          errorMessage = `Bad fee. Expected: ${expectedFee} ICP`;
        } else if ('AllowanceChanged' in error) {
          const currentAllowance = Number(error.AllowanceChanged.current_allowance) / 100_000_000;
          errorMessage = `Allowance changed. Current: ${currentAllowance} ICP`;
        } else if ('GenericError' in error) {
          errorMessage = error.GenericError.message;
        }
        
        throw new Error(errorMessage);
      }
    } catch (error) {
      console.error('Error approving ICP spend:', error);
      throw error;
    }
  }

  clearError(): void {
    balanceStore.update(state => ({ ...state, error: null }));
  }
}

// Utility function to convert e8s (10^8) to ICP display format
export const e8sToIcp = (e8s: bigint): string => {
  const icpAmount = Number(e8s) / 100_000_000;
  return icpAmount.toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 8,
  });
};

// Utility function to format ICP balance for display
export const formatIcpBalance = (balance: bigint): string => {
  const icp = e8sToIcp(balance);
  return `${icp} ICP`;
};