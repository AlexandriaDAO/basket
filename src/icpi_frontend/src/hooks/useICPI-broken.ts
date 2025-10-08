import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { HttpAgent, Actor } from '@dfinity/agent'
import { Principal } from '@dfinity/principal'
import { useInternetIdentity } from 'ic-use-internet-identity'
import { useICPIBackend } from './actors'
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

// Helper to create authenticated agent
function useAuthenticatedAgent(): HttpAgent | null {
  const { identity } = useInternetIdentity()

  if (!identity) return null

  const isLocal = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'
  const host = isLocal ? 'http://localhost:4943' : 'https://icp-api.io'

  const agent = new HttpAgent({
    identity,
    host,
    ingressExpiryMs: 5 * 60 * 1000,
  })

  if (isLocal) {
    agent.fetchRootKey().catch(console.error)
  }

  return agent
}

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

// ===== QUERY HOOKS =====

export const useIndexState = () => {
  const { actor } = useICPIBackend()

  return useQuery({
    queryKey: [QUERY_KEYS.INDEX_STATE],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')

      const result = await actor.get_index_state_cached()

      if ('Ok' in result) {
        return result.Ok
      } else if ('Err' in result) {
        console.error('get_index_state_cached returned error:', result.Err)
        throw new Error(result.Err)
      }
      throw new Error('Unexpected result format')
    },
    enabled: !!actor,
    refetchInterval: 60_000,
    staleTime: 30_000,
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  })
}

export const useRebalancerStatus = () => {
  const { actor } = useICPIBackend()

  return useQuery({
    queryKey: [QUERY_KEYS.REBALANCER_STATUS],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')

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
    refetchInterval: 60_000,
    staleTime: 30_000,
    retry: 0,
    retryDelay: 1000,
  })
}

export const useTVLData = () => {
  const { actor } = useICPIBackend()

  return useQuery({
    queryKey: [QUERY_KEYS.TVL_DATA],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      const result = await actor.get_tvl_summary()

      if ('Ok' in result) {
        return result.Ok
      } else if ('Err' in result) {
        throw new Error(result.Err)
      }
      throw new Error('Unexpected result format')
    },
    enabled: !!actor,
    staleTime: 10 * 60_000,
    refetchInterval: 5 * 60_000,
  })
}

export const useHoldings = () => {
  const { actor } = useICPIBackend()

  return useQuery({
    queryKey: [QUERY_KEYS.HOLDINGS],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      return await actor.get_token_holdings()
    },
    enabled: !!actor,
    refetchInterval: 2 * 60_000,
    staleTime: 60_000,
  })
}

export const useAllocation = () => {
  const { actor } = useICPIBackend()

  return useQuery({
    queryKey: [QUERY_KEYS.ALLOCATION],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      const stateResult = await actor.get_index_state()
      const tvlResult = await actor.get_tvl_summary()

      if (!('Ok' in stateResult) || !('Ok' in tvlResult)) {
        throw new Error(
          'Err' in stateResult ? stateResult.Err :
          'Err' in tvlResult ? tvlResult.Err :
          'Failed to fetch allocation data'
        )
      }

      return calculateAllocations(stateResult.Ok, tvlResult.Ok)
    },
    enabled: !!actor,
    refetchInterval: 2 * 60_000,
    staleTime: 60_000,
  })
}

export const useTotalSupply = () => {
  const { actor } = useICPIBackend()

  return useQuery({
    queryKey: [QUERY_KEYS.TOTAL_SUPPLY],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      const supply = await actor.icrc1_total_supply()
      return Number(supply) / 100_000_000
    },
    enabled: !!actor,
    staleTime: 10_000,
    refetchInterval: 30_000,
  })
}

export const useTokenMetadata = () => {
  const { actor } = useICPIBackend()

  return useQuery({
    queryKey: [QUERY_KEYS.TOKEN_METADATA],
    queryFn: async () => {
      if (!actor) throw new Error('Actor not initialized')
      const result = await actor.get_token_metadata()

      if ('Ok' in result) {
        return result.Ok
      } else if ('Err' in result) {
        throw new Error(result.Err)
      }
      throw new Error('Unexpected result format')
    },
    enabled: !!actor,
    staleTime: Infinity,
  })
}

export const useActualAllocations = () => {
  const { actor } = useICPIBackend()
  const agent = useAuthenticatedAgent()
  const queryClient = useQueryClient()

  return useQuery({
    queryKey: [QUERY_KEYS.ACTUAL_ALLOCATIONS, ICPI_BACKEND_CANISTER_ID],
    queryFn: async () => {
      if (!actor || !agent) {
        throw new Error('Actor or agent not initialized')
      }

      const [tokenMetadataResult, tvlDataResult] = await Promise.all([
        actor.get_token_metadata(),
        actor.get_tvl_summary(),
      ])

      if (!('Ok' in tokenMetadataResult) || !('Ok' in tvlDataResult)) {
        throw new Error(
          'Err' in tokenMetadataResult ? tokenMetadataResult.Err :
          'Err' in tvlDataResult ? tvlDataResult.Err :
          'Failed to fetch data'
        )
      }
      const tokenMetadata = tokenMetadataResult.Ok
      const tvlData = tvlDataResult.Ok

      const balancePromises = tokenMetadata.map(async (token: any) => {
        try {
          const tokenActor = Actor.createActor(ICRC1_IDL, {
            agent,
            canisterId: token.canister_id.toString(),
          })

          const balance = await tokenActor.icrc1_balance_of({
            owner: Principal.fromText(ICPI_BACKEND_CANISTER_ID),
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

      let indexState = queryClient.getQueryData([QUERY_KEYS.INDEX_STATE])

      if (!indexState) {
        const indexStateResult = await actor.get_index_state()
        if (!('Ok' in indexStateResult)) {
          throw new Error('Err' in indexStateResult ? indexStateResult.Err : 'Failed to get index state')
        }
        indexState = indexStateResult.Ok
      }

      const TRACKED_TOKENS = ['ALEX', 'ZERO', 'KONG', 'BOB']

      const allocations = balances
        .filter((bal) => TRACKED_TOKENS.includes(bal.symbol))
        .map((bal) => {
          const balance = BigInt(bal.balance)
          const decimals = BigInt(10) ** BigInt(bal.decimals)
          const balanceFloat = Number(balance) / Number(decimals)

          const position = indexState.current_positions.find(
            (p: any) => p.token[Object.keys(p.token)[0]] !== undefined &&
                        Object.keys(p.token)[0] === bal.symbol
          )

          const usdValue = position ? position.usd_value : 0

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

      const ckusdtPosition = indexState.current_positions.find(
        (p: any) => p.token.ckUSDT !== undefined
      )

      if (ckusdtPosition) {
        allocations.push({
          token: 'ckUSDT',
          balance: Number(ckusdtPosition.balance) / 1_000_000,
          value: ckusdtPosition.usd_value,
          decimals: 6,
          targetPercent: 0,
        })
      }

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
    enabled: !!actor && !!agent,
    refetchInterval: 2 * 60_000,
    staleTime: 60_000,
  })
}

export const useUserWalletBalances = (userPrincipal: string) => {
  const { actor } = useICPIBackend()
  const agent = useAuthenticatedAgent()
  const queryClient = useQueryClient()

  return useQuery({
    queryKey: [QUERY_KEYS.USER_WALLET_BALANCES, userPrincipal],
    queryFn: async () => {
      if (!actor || !userPrincipal || !agent) {
        throw new Error('Actor, principal, or agent not initialized')
      }

      const tokenMetadataResult = await actor.get_token_metadata()

      if (!('Ok' in tokenMetadataResult)) {
        throw new Error('Err' in tokenMetadataResult ? tokenMetadataResult.Err : 'Failed to fetch token metadata')
      }
      const trackedTokensMetadata = tokenMetadataResult.Ok

      const icpiMetadata = {
        symbol: 'ICPI',
        canister_id: Principal.fromText(ICPI_TOKEN_CANISTER_ID),
        decimals: 8,
      }

      const ckusdtMetadata = {
        symbol: 'ckUSDT',
        canister_id: Principal.fromText(CKUSDT_CANISTER_ID),
        decimals: 6,
      }

      const allTokensMetadata = [
        icpiMetadata,
        ckusdtMetadata,
        ...trackedTokensMetadata,
      ]

      const balancePromises = allTokensMetadata.map(async (token: any) => {
        try {
          const tokenActor = Actor.createActor(ICRC1_IDL, {
            agent,
            canisterId: token.canister_id.toString(),
          })

          const balance = await tokenActor.icrc1_balance_of({
            owner: Principal.fromText(userPrincipal),
            subaccount: [],
          })

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

      const balances = await Promise.all(balancePromises)

      let enrichedBalances = balances
      try {
        let indexState = queryClient.getQueryData([QUERY_KEYS.INDEX_STATE])

        if (!indexState) {
          const indexStateResult = await actor.get_index_state()
          if ('Ok' in indexStateResult) {
            indexState = indexStateResult.Ok
          }
        }

        if (indexState) {
          enrichedBalances = balances.map(balance => {
            const position = indexState.current_positions.find((p: any) => {
              const tokenKey = Object.keys(p.token)[0]
              return tokenKey === balance.symbol
            })

            if (position && balance.balanceFormatted > 0) {
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
      }

      enrichedBalances.sort((a, b) => {
        const aValue = a.usdValue || 0
        const bValue = b.usdValue || 0
        if (aValue !== bValue) return bValue - aValue
        return b.balanceFormatted - a.balanceFormatted
      })

      return enrichedBalances as UserTokenBalance[]
    },
    enabled: !!actor && !!userPrincipal && !!agent,
    refetchInterval: 30_000,
    staleTime: 15_000,
    retry: 2,
  })
}

// ===== MUTATION HOOKS =====

export const useMintICPI = () => {
  const { actor } = useICPIBackend()
  const agent = useAuthenticatedAgent()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (ckusdtAmount: number) => {
      if (!actor || !agent) throw new Error('Actor or agent not initialized')

      const amountRaw = BigInt(Math.floor(ckusdtAmount * 1e6))
      const feeAmount = BigInt(100_000)
      const ckusdtTransferFee = BigInt(10_000)

      const totalApproval = amountRaw + feeAmount + (ckusdtTransferFee * BigInt(2))

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

      const initResult = await actor.initiate_mint(amountRaw)

      if ('Err' in initResult) {
        throw new Error(initResult.Err)
      }
      const mintId = initResult.Ok

      const completeResult = await actor.complete_mint(mintId)

      if ('Err' in completeResult) {
        throw new Error(completeResult.Err)
      }

      return completeResult.Ok
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.INDEX_STATE] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_BALANCE] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.HOLDINGS] })
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_WALLET_BALANCES] })
    },
  })
}

export const useRedeemICPI = () => {
  const { actor } = useICPIBackend()
  const agent = useAuthenticatedAgent()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (icpiAmount: number) => {
      if (!actor || !agent) throw new Error('Actor or agent not initialized')

      const amountRaw = BigInt(Math.floor(icpiAmount * 1e8))
      const feeAmount = BigInt(100_000)
      const ckusdtTransferFee = BigInt(10_000)
      const icpiTransferFee = BigInt(10_000)

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
        amount: feeAmount + ckusdtTransferFee,
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

export const useManualRebalance = () => {
  const { actor } = useICPIBackend()
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

export const useTransferToken = () => {
  const agent = useAuthenticatedAgent()
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

      const tokenActor = Actor.createActor(ICRC1_IDL, {
        agent,
        canisterId: tokenCanisterId,
      })

      const fee = await tokenActor.icrc1_fee()

      const [wholePart, decimalPart = ''] = amount.split('.')
      const paddedDecimal = decimalPart.padEnd(decimals, '0').slice(0, decimals)
      const amountStr = wholePart + paddedDecimal
      const amountRaw = BigInt(amountStr)

      const transferArgs = {
        to: {
          owner: Principal.fromText(recipient),
          subaccount: [],
        },
        amount: amountRaw,
        fee: [fee],
        memo: [],
        from_subaccount: [],
        created_at_time: [],
      }

      const result = await tokenActor.icrc1_transfer(transferArgs)

      if ('Ok' in result) {
        return result.Ok
      } else if ('Err' in result) {
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
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_WALLET_BALANCES] })
    },
  })
}

// Helper function
function calculateAllocations(state: any, tvl: any) {
  const tokens = ['ALEX', 'ZERO', 'KONG', 'BOB']
  return tokens.map(token => {
    return {
      token,
      value: 0,
      currentPercent: 0,
      targetPercent: 0,
      deviation: 0,
    }
  })
}
