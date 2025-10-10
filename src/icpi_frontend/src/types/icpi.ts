// ICPI Type Definitions

export type MintStatus =
  | { Pending: null }
  | { CollectingFee: null }
  | { CollectingDeposit: null }
  | { Calculating: null }
  | { Refunding: null }              // NEW: Refund in progress
  | { Minting: null }
  | { Complete: bigint }
  | { Failed: string }
  | { FailedRefunded: string }       // NEW: Failed but deposit refunded
  | { FailedNoRefund: string }       // NEW: Failed and refund also failed
  | { Expired: null }

export interface BurnResult {
  successful_transfers: Array<[string, bigint]>
  failed_transfers: Array<[string, bigint, string]>
  icpi_burned: bigint
}

export interface TokenTransfer {
  symbol: string
  amount: bigint
  error?: string
}

// Constants
export const CKUSDT_DECIMALS = 6
export const ICPI_DECIMALS = 8
export const TOKEN_DECIMALS = 8  // ALEX, ZERO, KONG, BOB

export const MINT_BURN_FEE = 1_000_000n  // 1.0 ckUSDT (6 decimals)

// Canister IDs (Mainnet)
export const ICPI_CANISTER = 'ehyav-lqaaa-aaaap-qqc2a-cai'
export const CKUSDT_CANISTER = 'cngnf-vqaaa-aaaar-qag4q-cai'
export const FEE_RECIPIENT = 'e454q-riaaa-aaaap-qqcyq-cai'

// Token Canisters
export const ALEX_CANISTER = 'ysy5f-2qaaa-aaaap-qkmmq-cai'      // 8 decimals
export const ZERO_CANISTER = 'b3d2q-ayaaa-aaaap-qqcfq-cai'      // 8 decimals
export const KONG_CANISTER = 'xnjld-hqaaa-aaaar-qah4q-cai'      // 8 decimals
export const BOB_CANISTER = '7pail-xaaaa-aaaas-aabmq-cai'       // 8 decimals

// ===== TRADE HISTORY TYPES =====

export type TrackedToken =
  | { ALEX: null }
  | { ZERO: null }
  | { KONG: null }
  | { BOB: null }
  | { ckUSDT: null }

export type RebalanceAction =
  | { None: null }
  | { Buy: { token: TrackedToken; usdt_amount: number } }
  | { Sell: { token: TrackedToken; usdt_value: number } }

export interface RebalanceRecord {
  timestamp: bigint
  action: RebalanceAction
  success: boolean
  details: string
}

export interface RebalancerStatus {
  timer_active: boolean
  last_rebalance: [] | [bigint]
  next_rebalance: [] | [bigint]
  recent_history: RebalanceRecord[]
}

// ===== HELPER FUNCTIONS =====

/** Format TrackedToken for display */
export function formatTrackedToken(token: TrackedToken): string {
  if ('ALEX' in token) return 'ALEX'
  if ('ZERO' in token) return 'ZERO'
  if ('KONG' in token) return 'KONG'
  if ('BOB' in token) return 'BOB'
  if ('ckUSDT' in token) return 'ckUSDT'
  return 'UNKNOWN'
}

/** Format RebalanceAction for display */
export function formatRebalanceAction(action: RebalanceAction): {
  type: 'buy' | 'sell' | 'none'
  token: string
  amount: number
} {
  if ('Buy' in action) {
    return {
      type: 'buy',
      token: formatTrackedToken(action.Buy.token),
      amount: action.Buy.usdt_amount,
    }
  }
  if ('Sell' in action) {
    return {
      type: 'sell',
      token: formatTrackedToken(action.Sell.token),
      amount: action.Sell.usdt_value,
    }
  }
  return {
    type: 'none',
    token: '',
    amount: 0,
  }
}

/** Format trade timestamp to readable string */
export function formatTradeTimestamp(timestamp: bigint): string {
  const ms = Number(timestamp / 1_000_000n)
  return new Date(ms).toLocaleString()
}
