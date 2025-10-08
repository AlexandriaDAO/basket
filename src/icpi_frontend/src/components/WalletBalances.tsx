import React, { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from './ui/card'
import { Button } from './ui/button'
import { Badge } from './ui/badge'
import { formatNumber } from '@/lib/utils'
import { Copy, Send, ChevronDown, ChevronUp, RefreshCw } from 'lucide-react'
import { UserTokenBalance } from '@/hooks/useICPI'

interface WalletBalancesProps {
  balances: UserTokenBalance[]
  userPrincipal: string
  isLoading?: boolean
  onSendToken?: (symbol: string) => void
  onRefresh?: () => void
}

export const WalletBalances: React.FC<WalletBalancesProps> = ({
  balances,
  userPrincipal,
  isLoading = false,
  onSendToken,
  onRefresh,
}) => {
  const [isExpanded, setIsExpanded] = useState(true)
  const [copiedAddress, setCopiedAddress] = useState(false)

  const handleCopyPrincipal = async () => {
    try {
      await navigator.clipboard.writeText(userPrincipal)
      setCopiedAddress(true)
      setTimeout(() => setCopiedAddress(false), 2000)
    } catch (error) {
      console.error('Failed to copy:', error)
    }
  }

  // Categorize balances into groups (already sorted by balance)
  const indexToken = balances.find(b => b.symbol === 'ICPI')
  const stablecoin = balances.find(b => b.symbol === 'ckUSDT')
  const portfolioTokens = balances.filter(b =>
    b.symbol !== 'ICPI' && b.symbol !== 'ckUSDT'
  )

  return (
    <Card className="border-[#1f1f1f]">
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">WALLET</CardTitle>
          <div className="flex items-center gap-1">
            {onRefresh && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onRefresh}
                className="h-6 w-6 p-0"
                title="Refresh balances"
              >
                <RefreshCw className="h-3 w-3" />
              </Button>
            )}
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsExpanded(!isExpanded)}
              className="h-6 w-6 p-0"
            >
              {isExpanded ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />}
            </Button>
          </div>
        </div>
      </CardHeader>

      {isExpanded && (
        <CardContent className="p-3 space-y-3">
          {/* SECTION 1: Receive Address */}
          <div className="space-y-1">
            <div className="flex items-center justify-between">
              <span className="text-[10px] text-[#666666] uppercase">Your Address</span>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleCopyPrincipal}
                className="h-5 px-1 text-[10px]"
              >
                <Copy className="h-3 w-3 mr-1" />
                {copiedAddress ? 'COPIED' : 'COPY'}
              </Button>
            </div>
            <div className="text-[10px] font-mono text-white bg-[#0a0a0a] p-1.5 border border-[#1f1f1f] break-all">
              {userPrincipal}
            </div>
            <p className="text-[9px] text-[#666666]">
              Share this address to receive any ICRC1 token
            </p>
          </div>

          <div className="border-t border-[#1f1f1f] pt-2" />

          {/* SECTION 2: Index Token (ICPI) */}
          {indexToken && (
            <div className="space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-[10px] text-[#666666] uppercase">Index Token</span>
              </div>
              <TokenBalanceRow
                token={indexToken}
                onSend={onSendToken}
                highlight={true}
              />
            </div>
          )}

          {/* SECTION 3: Stablecoin (ckUSDT) */}
          {stablecoin && (
            <div className="space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-[10px] text-[#666666] uppercase">Stablecoin</span>
              </div>
              <TokenBalanceRow
                token={stablecoin}
                onSend={onSendToken}
              />
            </div>
          )}

          {/* SECTION 4: Portfolio Tokens (Dynamic) */}
          {portfolioTokens.length > 0 && (
            <div className="space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-[10px] text-[#666666] uppercase">Portfolio Tokens</span>
                <Badge variant="secondary" className="h-4 text-[9px]">
                  {portfolioTokens.length}
                </Badge>
              </div>
              <div className="space-y-0.5">
                {portfolioTokens.map(token => (
                  <TokenBalanceRow
                    key={token.symbol}
                    token={token}
                    onSend={onSendToken}
                  />
                ))}
              </div>
            </div>
          )}


          {/* Loading State */}
          {isLoading && (
            <div className="text-center py-4">
              <span className="text-[10px] text-[#666666]">Loading balances...</span>
            </div>
          )}

          {/* Empty State */}
          {!isLoading && balances.length === 0 && (
            <div className="text-center py-4">
              <span className="text-[10px] text-[#666666]">Your wallet is empty</span>
            </div>
          )}
        </CardContent>
      )}
    </Card>
  )
}

// Individual token balance row component
interface TokenBalanceRowProps {
  token: UserTokenBalance
  onSend?: (symbol: string) => void
  highlight?: boolean
}

const TokenBalanceRow: React.FC<TokenBalanceRowProps> = ({
  token,
  onSend,
  highlight = false
}) => {
  return (
    <div className={`flex items-center justify-between p-1.5 ${highlight ? 'bg-[#0a0a0a] border border-[#1f1f1f]' : ''}`}>
      <div className="flex items-center gap-2 flex-1">
        <span className={`text-xs font-mono ${highlight ? 'text-[#00FF41]' : 'text-white'}`}>
          {token.symbol}
        </span>
        {token.error && (
          <span className="text-[9px] text-[#FF0055]" title={token.error}>
            ⚠ Error
          </span>
        )}
      </div>

      <div className="flex items-center gap-2">
        <span className="text-xs font-mono text-white">
          {formatNumber(token.balanceFormatted)}
        </span>

        {onSend && token.balanceFormatted > 0 && !token.error && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onSend(token.symbol)}
            className="h-5 w-5 p-0"
            title="Send"
          >
            <Send className="h-3 w-3" />
          </Button>
        )}
      </div>
    </div>
  )
}
