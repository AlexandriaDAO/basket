import React, { useState } from 'react'
import { formatNumber, shortenAddress } from '@/lib/utils'
import { Copy } from 'lucide-react'

interface HeaderProps {
  principal: string
  tvl: number
  totalSupply: number
  indexPrice: number
  apy: number
  onDisconnect: () => void
}

export const DashboardHeader: React.FC<HeaderProps> = ({
  principal,
  tvl,
  totalSupply,
  indexPrice,
  apy,
  onDisconnect,
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
    <header className="border-b border-[#1f1f1f] bg-[#000000] sticky top-0 z-50">
      <div className="container py-1.5 px-3">
        <div className="flex items-center justify-between text-xs font-mono">
          <div className="flex items-center gap-4">
            <span className="text-white font-semibold">ICPI</span>
            <span className="text-[#666666]">TVL</span>
            <span className="text-white">{formatNumber(tvl)}</span>
            <span className="text-[#666666]">SUPPLY</span>
            <span className="text-white">{formatNumber(totalSupply)}</span>
            <span className="text-[#666666]">PRICE</span>
            <span className="text-white">${indexPrice.toFixed(4)}</span>
            <span className="text-[#666666]">APY</span>
            <span className="text-white">{apy.toFixed(2)}%</span>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-[#666666]">{shortenAddress(principal)}</span>
            <button
              onClick={handleCopy}
              className="text-[#999999] hover:text-[#00FF41] transition-none px-1.5 py-0.5 border border-[#1f1f1f] hover:border-[#00FF41] flex items-center gap-1"
              title="Copy principal"
            >
              <Copy className="h-3 w-3" />
              {copied ? 'COPIED' : 'COPY'}
            </button>
            <button
              onClick={onDisconnect}
              className="text-[#999999] hover:text-[#00FF41] transition-none px-2 py-0.5 border border-[#1f1f1f] hover:border-[#00FF41]"
            >
              DISCONNECT
            </button>
          </div>
        </div>
      </div>
    </header>
  )
}