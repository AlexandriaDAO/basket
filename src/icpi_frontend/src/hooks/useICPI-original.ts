import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Actor, HttpAgent } from '@dfinity/agent'
import { Principal } from '@dfinity/principal'
import { ICPI_BACKEND_CANISTER_ID, ICPI_TOKEN_CANISTER_ID, CKUSDT_CANISTER_ID } from '../constants/canisters'

// Types
export interface UserTokenBalance {
  symbol: string
  balance: string          // Raw balance as string
  balanceFormatted: number // Human-readable
  decimals: number
  canisterId: string
  usdValue?: number        // USD value if price is available
  error?: string           // Error message if balance query failed
}

// Define the query keys
export const QUERY_KEYS = {
  INDEX_STATE: 'indexState',
  REBALANCER_STATUS: 'rebalancerStatus',
  TVL_DATA: 'tvlData',
  USER_BALANCE: 'userBalance',
  HOLDINGS: 'holdings',
  ALLOCATION: 'allocation',
  TOKEN_METADATA: 'tokenMetadata',
  ACTUAL_ALLOCATIONS: 'actualAllocations',
  TRACKED_TOKENS: 'trackedTokens',
  USER_WALLET_BALANCES: 'userWalletBalances',
  TOTAL_SUPPLY: 'totalSupply',
} as const

// ICRC1/ICRC2 IDL for balance queries, transfers, and approvals
const ICRC1_IDL = ({ IDL }: any) => {
  const Account = IDL.Record({
    owner: IDL.Principal,
    subaccount: IDL.Opt(IDL.Vec(IDL.Nat8)),
  })

  // Transfer types
  const TransferArg = IDL.Record({
    to: Account,
    fee: IDL.Opt(IDL.Nat),
    memo: IDL.Opt(IDL.Vec(IDL.Nat8)),
    from_subaccount: IDL.Opt(IDL.Vec(IDL.Nat8)),
    created_at_time: IDL.Opt(IDL.Nat64),
    amount: IDL.Nat,
  })

  const TransferError = IDL.Variant({
    BadFee: IDL.Record({ expected_fee: IDL.Nat }),
    BadBurn: IDL.Record({ min_burn_amount: IDL.Nat }),
    InsufficientFunds: IDL.Record({ balance: IDL.Nat }),
    TooOld: IDL.Null,
    CreatedInFuture: IDL.Record({ ledger_time: IDL.Nat64 }),
    TemporarilyUnavailable: IDL.Null,
    Duplicate: IDL.Record({ duplicate_of: IDL.Nat }),
    GenericError: IDL.Record({ error_code: IDL.Nat, message: IDL.Text }),
  })

  const TransferResult = IDL.Variant({
    Ok: IDL.Nat,
    Err: TransferError,
  })

  // ICRC2 Approve types
  const ApproveArgs = IDL.Record({
    from_subaccount: IDL.Opt(IDL.Vec(IDL.Nat8)),
    spender: Account,
    amount: IDL.Nat,
    expected_allowance: IDL.Opt(IDL.Nat),
    expires_at: IDL.Opt(IDL.Nat64),
    fee: IDL.Opt(IDL.Nat),
    memo: IDL.Opt(IDL.Vec(IDL.Nat8)),
    created_at_time: IDL.Opt(IDL.Nat64),
  })

  const ApproveError = IDL.Variant({
    BadFee: IDL.Record({ expected_fee: IDL.Nat }),
    InsufficientFunds: IDL.Record({ balance: IDL.Nat }),
    AllowanceChanged: IDL.Record({ current_allowance: IDL.Nat }),
    Expired: IDL.Record({ ledger_time: IDL.Nat64 }),
    TooOld: IDL.Null,
    CreatedInFuture: IDL.Record({ ledger_time: IDL.Nat64 }),
    Duplicate: IDL.Record({ duplicate_of: IDL.Nat }),
    TemporarilyUnavailable: IDL.Null,
    GenericError: IDL.Record({ error_code: IDL.Nat, message: IDL.Text }),
  })

  const ApproveResult = IDL.Variant({
    Ok: IDL.Nat,
    Err: ApproveError,
  })

  return IDL.Service({
    icrc1_balance_of: IDL.Func([Account], [IDL.Nat], ['query']),
    icrc1_fee: IDL.Func([], [IDL.Nat], ['query']),
    icrc1_transfer: IDL.Func([TransferArg], [TransferResult], []),
    icrc2_approve: IDL.Func([ApproveArgs], [ApproveResult], []),
  })
}

// Custom hooks for ICPI data
export const useIndexState = (actor: Actor | null) => {
  return useQuery({
    queryKey: [QUERY_KEYS.INDEX_STATE],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')

      // Try cached version first (fast query)
      const result = await actor.get_index_state_cached()

      // Handle Result type - unwrap Ok/Err variant
      if ('Ok' in result) {
        return result.Ok
      } else if ('Err' in result) {
        console.error('get_index_state_cached returned error:', result.Err)
        throw new Error(result.Err)
      }
      throw new Error('Unexpected result format')
    },
    enabled: !!actor,
    refetchInterval: 60_000, // Refetch every 60 seconds (cache refreshes every 5 min on backend)
    staleTime: 30_000, // Consider stale after 30 seconds
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
    onError: (error: any) => console.error('useIndexState error:', error),
  })
}

export const useRebalancerStatus = (actor: Actor | null) => {
  return useQuery({
    queryKey: [QUERY_KEYS.REBALANCER_STATUS],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')

      // Add timeout wrapper to prevent hanging
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => {
          console.warn('⚠️ get_rebalancer_status timed out after 10s')
          reject(new Error('get_rebalancer_status call timed out after 10s'))
        }, 10000)
      })

      const result = await Promise.race([
        actor.get_rebalancer_status(),
        timeoutPromise
      ])

      return result
    },
    enabled: !!actor,
    refetchInterval: 60_000, // Refetch every 1 minute
    staleTime: 30_000,
    retry: 0, // Don't retry on timeout
    retryDelay: 1000, // Wait 1s between retries
    onError: (error: any) => console.error('useRebalancerStatus error:', error),
  })
}

export const useTVLData = (actor: Actor | null) => {
  return useQuery({
    queryKey: [QUERY_KEYS.TVL_DATA],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      const result = await actor.get_tvl_summary()

      // Handle Result type - unwrap Ok/Err variant
      if ('Ok' in result) {
        return result.Ok
      } else if ('Err' in result) {
        throw new Error(result.Err)
      }
      throw new Error('Unexpected result format')
    },
    enabled: !!actor,
    staleTime: 10 * 60_000, // 10 minutes
    refetchInterval: 5 * 60_000, // Refetch every 5 minutes
  })
}

export const useUserBalance = (actor: Actor | null, principal: string | null) => {
  return useQuery({
    queryKey: [QUERY_KEYS.USER_BALANCE, principal],
    queryFn: async () => {
      if (!actor || !principal) throw new Error('Actor or principal not available')
      const icpiBalance = await actor.balance_of({ owner: principal, subaccount: [] })
      // Also fetch ckUSDT balance if needed
      return {
        icpi: icpiBalance,
        ckusdt: 0, // This would come from ckUSDT canister
      }
    },
    enabled: !!actor && !!principal,
    refetchInterval: 2 * 60_000, // Refetch every 2 minutes
    staleTime: 60_000,
  })
}

export const useHoldings = (actor: Actor | null) => {
  return useQuery({
    queryKey: [QUERY_KEYS.HOLDINGS],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      return await actor.get_token_holdings()
    },
    enabled: !!actor,
    refetchInterval: 2 * 60_000, // Refetch every 2 minutes
    staleTime: 60_000,
  })
}

export const useAllocation = (actor: Actor | null) => {
  return useQuery({
    queryKey: [QUERY_KEYS.ALLOCATION],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      const stateResult = await actor.get_index_state()
      const tvlResult = await actor.get_tvl_summary()

      // Unwrap Result types
      if (!('Ok' in stateResult) || !('Ok' in tvlResult)) {
        throw new Error(
          'Err' in stateResult ? stateResult.Err :
          'Err' in tvlResult ? tvlResult.Err :
          'Failed to fetch allocation data'
        )
      }

      // Calculate allocations based on state and TVL
      return calculateAllocations(stateResult.Ok, tvlResult.Ok)
    },
    enabled: !!actor,
    refetchInterval: 2 * 60_000, // Refetch every 2 minutes
    staleTime: 60_000,
  })
}

// Mutation hooks
export const useMintICPI = (actor: Actor | null, agent: HttpAgent | null) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (ckusdtAmount: number) => {
      if (!actor || !agent) throw new Error('Actor or agent not initialized')

      const amountRaw = BigInt(Math.floor(ckusdtAmount * 1e6)) // ckUSDT has 6 decimals
      const feeAmount = BigInt(100_000) // 0.1 ckUSDT backend fee
      const ckusdtTransferFee = BigInt(10_000) // 0.01 ckUSDT ICRC1 transfer fee

      // Total approval: mint amount + backend fee + buffer for 2 transfers (fee collection + deposit)
      const totalApproval = amountRaw + feeAmount + (ckusdtTransferFee * BigInt(2))

      // Step 0: Approve ICPI backend to spend ckUSDT
      const ckusdtActor = Actor.createActor(ICRC1_IDL, {
        agent,
        canisterId: CKUSDT_CANISTER_ID,
      })

      const icpiBackendPrincipal = Principal.fromText(ICPI_BACKEND_CANISTER_ID)

      const approveArgs = {
        from_subaccount: [],
        spender: {
          owner: icpiBackendPrincipal,
          subaccount: [],
        },
        amount: totalApproval,
        expected_allowance: [],
        expires_at: [],
        fee: [ckusdtTransferFee],
        memo: [],
        created_at_time: [],
      }

      const approveResult = await ckusdtActor.icrc2_approve(approveArgs)

      if ('Err' in approveResult) {
        throw new Error(`Approval failed: ${JSON.stringify(approveResult.Err)}`)
      }

      // Phase 1: Initiate mint (returns mint_id)
      const initResult = await actor.initiate_mint(amountRaw)

      if ('Err' in initResult) {
        throw new Error(initResult.Err)
      }
      const mintId = initResult.Ok

      // Phase 2: Complete mint (returns ICPI amount)
      const completeResult = await actor.complete_mint(mintId)

      if ('Err' in completeResult) {
        throw new Error(completeResult.Err)
      }

      return completeResult.Ok
    },
    onSuccess: () => {
      // Invalidate relevant queries
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.INDEX_STATE] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_BALANCE] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.HOLDINGS] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_WALLET_BALANCES] })
    },
  })
}

export const useRedeemICPI = (actor: Actor | null, agent: HttpAgent | null) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (icpiAmount: number) => {
      if (!actor || !agent) throw new Error('Actor or agent not initialized')

      const amountRaw = BigInt(Math.floor(icpiAmount * 1e8)) // ICPI has 8 decimals
      const feeAmount = BigInt(100_000) // 0.1 ckUSDT backend fee (6 decimals)
      const ckusdtTransferFee = BigInt(10_000) // 0.01 ckUSDT ICRC1 transfer fee
      const icpiTransferFee = BigInt(10_000) // ICPI transfer fee (8 decimals, but numerically 10000)

      // Step 1: Approve backend to collect fee (ckUSDT)
      const ckusdtActor = Actor.createActor(ICRC1_IDL, {
        agent,
        canisterId: CKUSDT_CANISTER_ID,
      })

      const icpiBackendPrincipal = Principal.fromText(ICPI_BACKEND_CANISTER_ID)

      const approveArgs = {
        from_subaccount: [],
        spender: {
          owner: icpiBackendPrincipal,
          subaccount: [],
        },
        amount: feeAmount + ckusdtTransferFee, // Fee + transfer fee
        expected_allowance: [],
        expires_at: [],
        fee: [ckusdtTransferFee],
        memo: [],
        created_at_time: [],
      }

      const approveResult = await ckusdtActor.icrc2_approve(approveArgs)

      if ('Err' in approveResult) {
        throw new Error(`Fee approval failed: ${JSON.stringify(approveResult.Err)}`)
      }

      // Step 2: Transfer ICPI to backend (this burns it since backend is minting account)
      const icpiActor = Actor.createActor(ICRC1_IDL, {
        agent,
        canisterId: ICPI_TOKEN_CANISTER_ID,
      })

      const transferArgs = {
        to: {
          owner: icpiBackendPrincipal,
          subaccount: [],
        },
        amount: amountRaw,
        fee: [icpiTransferFee],
        memo: [],
        from_subaccount: [],
        created_at_time: [],
      }

      const transferResult = await icpiActor.icrc1_transfer(transferArgs)

      if ('Err' in transferResult) {
        throw new Error(`ICPI transfer failed: ${JSON.stringify(transferResult.Err)}`)
      }

      // Step 3: Call atomic burn_icpi (backend will verify burn and send redemption tokens)
      const burnResult = await actor.burn_icpi(amountRaw)

      if ('Err' in burnResult) {
        throw new Error(burnResult.Err)
      }

      return burnResult.Ok
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.INDEX_STATE] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_BALANCE] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.HOLDINGS] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_WALLET_BALANCES] })
    },
  })
}

export const useManualRebalance = (actor: Actor | null) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      return await actor.execute_rebalance()
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.INDEX_STATE] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.REBALANCER_STATUS] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.HOLDINGS] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.ALLOCATION] })
    },
  })
}

// Hook for transferring ICRC1 tokens
export const useTransferToken = (agent: HttpAgent | null) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({
      tokenCanisterId,
      recipient,
      amount,
      decimals
    }: {
      tokenCanisterId: string
      recipient: string
      amount: string
      decimals: number
    }) => {
      if (!agent) throw new Error('Agent not initialized')

      // STEP 1: Create actor for the token canister
      const tokenActor = Actor.createActor(ICRC1_IDL, {
        agent,
        canisterId: tokenCanisterId,
      })

      // STEP 2: Query fee from token canister
      const fee = await tokenActor.icrc1_fee()

      // STEP 3: Convert amount to raw units using string manipulation
      // to avoid floating-point precision loss
      const [wholePart, decimalPart = ''] = amount.split('.')
      const paddedDecimal = decimalPart.padEnd(decimals, '0').slice(0, decimals)
      const amountStr = wholePart + paddedDecimal
      const amountRaw = BigInt(amountStr)

      // STEP 4: Prepare transfer args (ICRC1 standard)
      const transferArgs = {
        to: {
          owner: Principal.fromText(recipient),
          subaccount: [],
        },
        amount: amountRaw,
        fee: [fee], // Always include fee
        memo: [], // Optional
        from_subaccount: [], // Optional
        created_at_time: [], // Optional
      }

      // STEP 5: Call icrc1_transfer
      const result = await tokenActor.icrc1_transfer(transferArgs)

      // STEP 6: Handle Result type (Ok/Err)
      if ('Ok' in result) {
        return result.Ok // Block index
      } else if ('Err' in result) {
        // Parse error for user-friendly message
        const errMsg = JSON.stringify(result.Err)
        if (errMsg.includes('InsufficientFunds')) {
          throw new Error('Insufficient funds (remember to account for transfer fee)')
        } else if (errMsg.includes('BadFee')) {
          throw new Error('Incorrect fee amount')
        }
        throw new Error(`Transfer failed: ${errMsg}`)
      }
      throw new Error('Unexpected result format')
    },
    onSuccess: () => {
      // Invalidate user wallet balances to trigger refresh
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_WALLET_BALANCES] })
    },
  })
}

// Get token metadata from backend
export const useTokenMetadata = (actor: Actor | null) => {
  return useQuery({
    queryKey: [QUERY_KEYS.TOKEN_METADATA],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      const result = await actor.get_token_metadata()

      // Handle Result type - unwrap Ok/Err variant
      if ('Ok' in result) {
        return result.Ok
      } else if ('Err' in result) {
        throw new Error(result.Err)
      }
      throw new Error('Unexpected result format')
    },
    enabled: !!actor,
    staleTime: Infinity, // Token metadata never changes
  })
}

// Get ICPI total supply
export const useTotalSupply = (actor: Actor | null) => {
  return useQuery({
    queryKey: [QUERY_KEYS.TOTAL_SUPPLY],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      const supply = await actor.icrc1_total_supply()
      // Convert from e8 to human-readable (8 decimals)
      return Number(supply) / 100_000_000
    },
    enabled: !!actor,
    staleTime: 10_000, // 10 seconds
    refetchInterval: 30_000, // Refetch every 30s
  })
}

// Query token balances directly from token canisters
export const useActualAllocations = (
  icpiActor: Actor | null,
  icpiCanisterId: string | null,
  agent: HttpAgent | null
) => {
  const queryClient = useQueryClient()
  return useQuery({
    queryKey: [QUERY_KEYS.ACTUAL_ALLOCATIONS, icpiCanisterId],
    queryFn: async () => {
      if (!icpiActor || !icpiActor || !icpiCanisterId || !agent) {
        throw new Error('Actor, canister ID, or agent not initialized')
      }

      // Get token metadata and TVL data
      const [tokenMetadataResult, tvlDataResult] = await Promise.all([
        icpiActor.get_token_metadata(),
        icpiActor.get_tvl_summary(),
      ])

      // Unwrap Result types
      if (!('Ok' in tokenMetadataResult) || !('Ok' in tvlDataResult)) {
        throw new Error(
          'Err' in tokenMetadataResult ? tokenMetadataResult.Err :
          'Err' in tvlDataResult ? tvlDataResult.Err :
          'Failed to fetch data'
        )
      }
      const tokenMetadata = tokenMetadataResult.Ok
      const tvlData = tvlDataResult.Ok

      // Query balances directly from token canisters (true query calls!)
      const balancePromises = tokenMetadata.map(async (token: any) => {
        try {
          const tokenActor = Actor.createActor(ICRC1_IDL, {
            agent,
            canisterId: token.canister_id.toString(),
          })

          const balance = await tokenActor.icrc1_balance_of({
            owner: Principal.fromText(icpiCanisterId),
            subaccount: [],
          })

          return {
            symbol: token.symbol,
            balance: balance.toString(),
            decimals: token.decimals,
          }
        } catch (error) {
          console.error(`Error querying ${token.symbol} balance:`, error)
          return {
            symbol: token.symbol,
            balance: '0',
            decimals: token.decimals,
          }
        }
      })

      const balances = await Promise.all(balancePromises)

      // Calculate USD values and percentages
      // Get current positions from backend for prices
      let indexState = queryClient.getQueryData([QUERY_KEYS.INDEX_STATE])

      if (!indexState) {
        // Fallback: call it if somehow not cached yet
        const indexStateResult = await icpiActor.get_index_state()
        if (!('Ok' in indexStateResult)) {
          throw new Error('Err' in indexStateResult ? indexStateResult.Err : 'Failed to get index state')
        }
        indexState = indexStateResult.Ok
      }

      // Track index tokens + ckUSDT (cash position for rebalancing)
      const TRACKED_TOKENS = ['ALEX', 'ZERO', 'KONG', 'BOB']

      const allocations = balances
        .filter((bal) => TRACKED_TOKENS.includes(bal.symbol))
        .map((bal) => {
          const balance = BigInt(bal.balance)
          const decimals = BigInt(10) ** BigInt(bal.decimals)
          const balanceFloat = Number(balance) / Number(decimals)

          // Find matching position for price
          const position = indexState.current_positions.find(
            (p: any) => p.token[Object.keys(p.token)[0]] !== undefined &&
                        Object.keys(p.token)[0] === bal.symbol
          )

          const usdValue = position ? position.usd_value : 0

          // Find target from TVL data
          const tvlEntry = tvlData.tokens.find(
            (t: any) => Object.keys(t.token)[0] === bal.symbol
          )
          const targetPercent = tvlEntry ? tvlEntry.percentage : 0

          return {
            token: bal.symbol,
            balance: balanceFloat,
            value: usdValue,
            decimals: bal.decimals,
            targetPercent,
          }
        })

      // Add ckUSDT position (cash held for rebalancing)
      const ckusdtPosition = indexState.current_positions.find(
        (p: any) => p.token.ckUSDT !== undefined
      )

      if (ckusdtPosition) {
        allocations.push({
          token: 'ckUSDT',
          balance: Number(ckusdtPosition.balance) / 1_000_000, // 6 decimals
          value: ckusdtPosition.usd_value,
          decimals: 6,
          targetPercent: 0, // ckUSDT is cash, not a target allocation
        })
      }

      // Calculate actual percentages (including ckUSDT to show total portfolio composition)
      const totalValue = allocations.reduce((sum, a) => sum + a.value, 0)

      const result = allocations.map((a) => ({
        token: a.token,
        value: a.value,
        currentPercent: totalValue > 0 ? (a.value / totalValue) * 100 : 0,
        targetPercent: a.targetPercent ?? 0,
        deviation: totalValue > 0 ? (a.targetPercent ?? 0) - (a.value / totalValue) * 100 : 0,
      }))

      return result
    },
    enabled: !!icpiActor && !!icpiCanisterId && !!agent,
    refetchInterval: 2 * 60_000, // Refetch every 2 minutes (query calls are fast but no need to spam)
    staleTime: 60_000,
  })
}

// Hook to get tracked tokens
export const useTrackedTokens = (actor: Actor | null) => {
  return useQuery({
    queryKey: [QUERY_KEYS.TRACKED_TOKENS],
    queryFn: async () => {
      if (!actor) return ['ALEX', 'ZERO', 'KONG', 'BOB'] // Fallback
      return await actor.get_tracked_tokens()
    },
    staleTime: 5 * 60_000, // Cache for 5 minutes
  })
}

// Hook to get user wallet balances for all tokens
export const useUserWalletBalances = (
  actor: Actor | null,
  userPrincipal: string | null,
  agent: HttpAgent | null
) => {
  const queryClient = useQueryClient()
  return useQuery({
    queryKey: [QUERY_KEYS.USER_WALLET_BALANCES, userPrincipal],
    queryFn: async () => {
      if (!actor || !userPrincipal || !agent) {
        throw new Error('Actor, principal, or agent not initialized')
      }

      // STEP 1: Get token metadata for all tracked tokens
      const tokenMetadataResult = await actor.get_token_metadata()

      if (!('Ok' in tokenMetadataResult)) {
        throw new Error('Err' in tokenMetadataResult ? tokenMetadataResult.Err : 'Failed to fetch token metadata')
      }
      const trackedTokensMetadata = tokenMetadataResult.Ok

      // STEP 2: Add ICPI token metadata (the index token itself)
      const icpiMetadata = {
        symbol: 'ICPI',
        canister_id: Principal.fromText(ICPI_TOKEN_CANISTER_ID),
        decimals: 8, // ICPI has 8 decimals
      }

      // STEP 3: Add ckUSDT metadata (special case - rebalancing currency)
      const ckusdtMetadata = {
        symbol: 'ckUSDT',
        canister_id: Principal.fromText(CKUSDT_CANISTER_ID),
        decimals: 6, // ckUSDT has 6 decimals
      }

      // STEP 4: Combine all token metadata
      const allTokensMetadata = [
        icpiMetadata,
        ckusdtMetadata,
        ...trackedTokensMetadata,
      ]

      // STEP 5: Query balances for all tokens in parallel
      const balancePromises = allTokensMetadata.map(async (token: any) => {
        try {
          // Create ICRC1 actor for each token canister
          const tokenActor = Actor.createActor(ICRC1_IDL, {
            agent,
            canisterId: token.canister_id.toString(),
          })

          // Query balance using ICRC1 standard
          const balance = await tokenActor.icrc1_balance_of({
            owner: Principal.fromText(userPrincipal),
            subaccount: [],
          })

          // Convert to human-readable format
          const balanceStr = balance.toString()
          const decimals = token.decimals
          const balanceFormatted = Number(balanceStr) / Math.pow(10, decimals)

          return {
            symbol: token.symbol,
            balance: balanceStr,
            balanceFormatted,
            decimals,
            canisterId: token.canister_id.toString(),
          }
        } catch (error) {
          console.error(`Error querying ${token.symbol} balance:`, error)
          // Return zero balance with error flag so UI can show warning
          return {
            symbol: token.symbol,
            balance: '0',
            balanceFormatted: 0,
            decimals: token.decimals,
            canisterId: token.canister_id.toString(),
            error: error instanceof Error ? error.message : 'Query failed',
          }
        }
      })

      // STEP 6: Await all balance queries
      const balances = await Promise.all(balancePromises)

      // STEP 7: Get USD values from index state
      let enrichedBalances = balances
      try {
        // Try cache first
        let indexState = queryClient.getQueryData([QUERY_KEYS.INDEX_STATE])

        if (!indexState) {
          // Fallback only if cache miss
          const indexStateResult = await actor.get_index_state()
          if ('Ok' in indexStateResult) {
            indexState = indexStateResult.Ok
          }
        }

        if (indexState) {

          // Enrich balances with USD values
          enrichedBalances = balances.map(balance => {
            // Find matching position for USD value
            const position = indexState.current_positions.find((p: any) => {
              const tokenKey = Object.keys(p.token)[0]
              return tokenKey === balance.symbol
            })

            if (position && balance.balanceFormatted > 0) {
              // Calculate USD value: (user_balance / canister_balance) * canister_usd_value
              const canisterBalance = Number(position.balance) / Math.pow(10, balance.decimals)
              if (canisterBalance > 0) {
                const pricePerToken = position.usd_value / canisterBalance
                const usdValue = balance.balanceFormatted * pricePerToken
                return { ...balance, usdValue }
              }
            }

            return balance
          })
        }
      } catch (error) {
        console.error('Failed to fetch USD values:', error)
        // Continue without USD values if this fails
      }

      // STEP 8: Sort by USD value (highest first), then by balance
      enrichedBalances.sort((a, b) => {
        const aValue = a.usdValue || 0
        const bValue = b.usdValue || 0
        if (aValue !== bValue) return bValue - aValue
        return b.balanceFormatted - a.balanceFormatted
      })

      return enrichedBalances as UserTokenBalance[]
    },
    enabled: !!actor && !!userPrincipal && !!agent,
    refetchInterval: 30_000, // Refetch every 30 seconds (query calls are fast)
    staleTime: 15_000,       // Consider stale after 15 seconds
    retry: 2,
  })
}

// Helper function to calculate allocations
function calculateAllocations(state: any, tvl: any) {
  // This would process the raw data into the format needed by the UI
  const tokens = ['ALEX', 'ZERO', 'KONG', 'BOB']
  return tokens.map(token => {
    // Calculate based on actual holdings vs TVL targets
    return {
      token,
      value: 0, // Calculate from state
      currentPercent: 0, // Calculate
      targetPercent: 0, // From TVL data
      deviation: 0, // Current - Target
    }
  })
}