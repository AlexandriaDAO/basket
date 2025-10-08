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
