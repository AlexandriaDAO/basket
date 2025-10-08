import React, { useState } from 'react'
import { PortfolioDashboard } from './PortfolioDashboard'
import { TradeInterface } from './TradeInterface'
import { WalletBalances } from './WalletBalances'
import { UserTokenBalance } from '@/hooks/useICPI'
import { formatNumber, shortenAddress } from '@/lib/utils'
import { Copy } from 'lucide-react'

interface DashboardProps {
  principal: string
  tvl: number
  portfolioData: any
  allocations: any[]
  rebalancingData: any
  userICPIBalance: number
  userUSDTBalance: number
  tokenHoldings: any[]
  walletBalances: UserTokenBalance[]
  onDisconnect: () => void
  onMint: (amount: number) => Promise<void>
  onRedeem: (amount: number) => Promise<void>
  onManualRebalance: () => Promise<void>
  onToggleAutoRebalance: (enabled: boolean) => void
  onSendToken: (symbol: string) => void
  onRefreshBalances: () => void
}

export const Dashboard: React.FC<DashboardProps> = ({
  principal,
  tvl,
  portfolioData,
  allocations,
  rebalancingData,
  userICPIBalance,
  userUSDTBalance,
  tokenHoldings,
  walletBalances,
  onDisconnect,
  onMint,
  onRedeem,
  onManualRebalance,
  onToggleAutoRebalance,
  onSendToken,
  onRefreshBalances,
}) => {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(principal)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (error) {
      console.error('Failed to copy:', error)
    }
  }

  return (
    <div className="min-h-screen bg-[#000000]">
      {/* Stats Bar */}
      <div className="border-b border-[#1f1f1f] bg-[#000000]">
        <div className="container py-2 px-3">
          <div className="overflow-x-auto">
            <div className="flex items-center justify-between text-xs font-mono min-w-max">
              <div className="flex items-center gap-4">
                <span className="text-[#666666]">TVL</span>
                <span className="text-white">{formatNumber(tvl)}</span>
                <span className="text-[#666666]">SUPPLY</span>
                <span className="text-white">{portfolioData.totalSupply.toFixed(2)} ICPI</span>
                <span className="text-[#666666]">NAV</span>
                <span className="text-white">${portfolioData.indexPrice.toFixed(4)}</span>
                <span className="text-[#666666]">APY</span>
                <span className="text-white">{portfolioData.apy.toFixed(2)}%</span>
              </div>
              <div className="flex items-center gap-3">
                <span className="text-[#666666]">{shortenAddress(principal)}</span>
                <button
                  onClick={handleCopy}
                  className="text-[#999999] hover:text-[#00FF41] transition-colors duration-150 px-1.5 py-0.5 border border-[#1f1f1f] hover:border-[#00FF41] flex items-center gap-1 active:scale-[0.98]"
                  title="Copy principal"
                >
                  <Copy className="h-3 w-3" />
                  <span className="hidden sm:inline">{copied ? 'COPIED' : 'COPY'}</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="container py-3 px-3">
        {/* Responsive grid: 1 col (mobile) -> 2 col (tablet) -> 3 col (desktop) */}
        <div className="grid grid-cols-1 lg:grid-cols-[1fr_1fr] xl:grid-cols-[1.5fr_1fr_1fr] gap-3">
          {/* Left: Portfolio Dashboard */}
          <PortfolioDashboard
            portfolioData={portfolioData}
            allocations={allocations}
            rebalancingData={rebalancingData}
            onManualRebalance={onManualRebalance}
            onToggleAutoRebalance={onToggleAutoRebalance}
          />

          {/* Middle: Trade Interface */}
          <TradeInterface
            currentTVL={tvl}
            currentSupply={portfolioData.totalSupply}
            userUSDTBalance={userUSDTBalance}
            onMint={onMint}
            userICPIBalance={userICPIBalance}
            tokenHoldings={tokenHoldings}
            onRedeem={onRedeem}
          />

          {/* Right: Wallet Balances */}
          <WalletBalances
            balances={walletBalances}
            userPrincipal={principal}
            onSendToken={onSendToken}
            onRefresh={onRefreshBalances}
          />
        </div>
      </div>
    </div>
  )
}