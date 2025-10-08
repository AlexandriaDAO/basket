import React, { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from './ui/card'
import { Button } from './ui/button'
import { Input } from './ui/input'
import { Label } from './ui/label'
import { Separator } from './ui/separator'
import { Loader2 } from 'lucide-react'
import { formatNumber } from '@/lib/utils'

// ICRC1 transfer fees (in human-readable units)
const CKUSDT_FEE = 0.01 // 0.01 USDT (10000 e6s)
const ICPI_FEE = 0.0001   // 0.0001 ICPI (10000 e8s)

interface TokenAmount {
  token: string
  amount: number
  value: number
}

interface TradeInterfaceProps {
  currentTVL: number
  currentSupply: number
  userUSDTBalance: number
  onMint: (amount: number) => Promise<void>
  userICPIBalance: number
  tokenHoldings: TokenAmount[]
  onRedeem: (amount: number) => Promise<void>
}

export const TradeInterface: React.FC<TradeInterfaceProps> = ({
  currentTVL,
  currentSupply,
  userUSDTBalance,
  onMint,
  userICPIBalance,
  tokenHoldings,
  onRedeem,
}) => {
  const [mintAmount, setMintAmount] = useState('')
  const [redeemAmount, setRedeemAmount] = useState('')
  const [isMinting, setIsMinting] = useState(false)
  const [isRedeeming, setIsRedeeming] = useState(false)
  const [error, setError] = useState('')

  const calculateMintReceive = (usdtAmount: number) => {
    if (currentSupply === 0) return usdtAmount
    return (usdtAmount * currentSupply) / currentTVL
  }

  const calculateRedeemReceive = (icpiAmount: number): TokenAmount[] => {
    const redeemPercent = userICPIBalance > 0 ? icpiAmount / userICPIBalance : 0
    return tokenHoldings.map(holding => ({
      token: holding.token,
      amount: holding.amount * redeemPercent,
      value: holding.value * redeemPercent,
    }))
  }

  const mintReceive = mintAmount ? calculateMintReceive(parseFloat(mintAmount)) : 0
  const redeemReceive = redeemAmount ? calculateRedeemReceive(parseFloat(redeemAmount)) : []

  const handleMint = async () => {
    const amount = parseFloat(mintAmount)
    if (!mintAmount || amount <= 0) {
      setError('Invalid amount')
      return
    }
    if (amount > userUSDTBalance) {
      setError('Insufficient balance')
      return
    }

    setError('')
    setIsMinting(true)
    try {
      await onMint(amount)
      setMintAmount('')
    } catch (err: any) {
      const errMsg = err.message || ''
      if (errMsg.includes('InsufficientFunds') || errMsg.includes('insufficient funds')) {
        setError(`Insufficient USDT balance (need ${amount} + ${CKUSDT_FEE} fee)`)
      } else if (errMsg.includes('Allowance') || errMsg.includes('allowance')) {
        setError('Token approval failed - please try again')
      } else if (errMsg.includes('rejected') || errMsg.includes('denied')) {
        setError('Transaction rejected by user')
      } else {
        setError(errMsg || 'Mint failed - please try again')
      }
    } finally {
      setIsMinting(false)
    }
  }

  const handleRedeem = async () => {
    const amount = parseFloat(redeemAmount)
    if (!redeemAmount || amount <= 0) {
      setError('Invalid amount')
      return
    }
    if (amount > userICPIBalance) {
      setError('Insufficient balance')
      return
    }

    setError('')
    setIsRedeeming(true)
    try {
      await onRedeem(amount)
      setRedeemAmount('')
    } catch (err: any) {
      const errMsg = err.message || ''
      if (errMsg.includes('InsufficientFunds') || errMsg.includes('insufficient funds')) {
        setError(`Insufficient ICPI balance (need ${amount} + ${ICPI_FEE} fee)`)
      } else if (errMsg.includes('rejected') || errMsg.includes('denied')) {
        setError('Transaction rejected by user')
      } else if (errMsg.includes('supply') || errMsg.includes('Supply')) {
        setError('Cannot redeem - insufficient token reserves')
      } else {
        setError(errMsg || 'Redeem failed - please try again')
      }
    } finally {
      setIsRedeeming(false)
    }
  }

  return (
    <div className="space-y-2">
      <Card className="border-[#1f1f1f]">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">MINT</CardTitle>
        </CardHeader>
        <CardContent className="p-3 space-y-2">
          <div className="space-y-1">
            <Label>USDT Amount</Label>
            <div className="flex gap-2">
              <Input
                type="number"
                placeholder="0.00"
                value={mintAmount}
                onChange={(e) => setMintAmount(e.target.value)}
                disabled={isMinting}
                className="flex-1 text-right"
              />
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  // Reserve fee for the transfer
                  const maxAmount = Math.max(0, userUSDTBalance - CKUSDT_FEE)
                  setMintAmount(maxAmount.toFixed(6))
                }}
                className="px-2"
              >
                MAX
              </Button>
            </div>
            <p className="text-[10px] text-[#666666] text-right font-mono">
              Balance: {formatNumber(userUSDTBalance)} • Fee: {CKUSDT_FEE} USDT
            </p>
          </div>

          {mintReceive > 0 && (
            <div className="pt-2 border-t border-[#1f1f1f] space-y-1">
              <div className="flex justify-between text-xs">
                <span className="text-[#666666]">Receive</span>
                <span className="text-white font-mono">{mintReceive.toFixed(6)} ICPI</span>
              </div>
              <div className="flex justify-between text-[10px]">
                <span className="text-[#666666]">Price per ICPI</span>
                <span className="text-[#999999] font-mono">${(parseFloat(mintAmount) / mintReceive).toFixed(4)}</span>
              </div>
              <div className="flex justify-between text-[10px]">
                <span className="text-[#666666]">Transfer Fee</span>
                <span className="text-[#FFE600] font-mono">{CKUSDT_FEE} USDT</span>
              </div>
            </div>
          )}

          {!mintReceive && mintAmount && parseFloat(mintAmount) > 0 && (
            <div className="pt-2 border-t border-[#1f1f1f]">
              <div className="flex justify-between text-[10px]">
                <span className="text-[#666666]">Transfer Fee</span>
                <span className="text-[#FFE600] font-mono">{CKUSDT_FEE} USDT</span>
              </div>
            </div>
          )}

          <Button
            variant="primary"
            size="sm"
            className="w-full"
            onClick={handleMint}
            disabled={isMinting || !mintAmount || parseFloat(mintAmount) <= 0}
          >
            {isMinting ? (
              <>
                <Loader2 className="mr-2 h-3 w-3 animate-spin" />
                Minting...
              </>
            ) : (
              'MINT ICPI'
            )}
          </Button>
        </CardContent>
      </Card>

      <Card className="border-[#1f1f1f]">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">REDEEM</CardTitle>
        </CardHeader>
        <CardContent className="p-3 space-y-2">
          <div className="space-y-1">
            <Label>ICPI Amount</Label>
            <div className="flex gap-2">
              <Input
                type="number"
                placeholder="0.00"
                value={redeemAmount}
                onChange={(e) => setRedeemAmount(e.target.value)}
                disabled={isRedeeming}
                className="flex-1 text-right"
              />
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  // Reserve fee for the burn operation
                  const maxAmount = Math.max(0, userICPIBalance - ICPI_FEE)
                  setRedeemAmount(maxAmount.toFixed(8))
                }}
                className="px-2"
              >
                MAX
              </Button>
            </div>
            <p className="text-[10px] text-[#666666] text-right font-mono">
              Balance: {userICPIBalance.toFixed(6)} • Fee: {ICPI_FEE} ICPI
            </p>
          </div>

          {redeemReceive.length > 0 && (
            <div className="pt-2 border-t border-[#1f1f1f] space-y-1">
              <span className="text-[10px] text-[#666666] uppercase">Receive</span>
              {redeemReceive.map(item => (
                <div key={item.token} className="flex justify-between text-xs">
                  <span className="text-[#999999]">{item.token}</span>
                  <span className="text-white font-mono">{item.amount.toFixed(6)}</span>
                </div>
              ))}
              <div className="flex justify-between text-[10px] pt-1 border-t border-[#1f1f1f]">
                <span className="text-[#666666]">Burn Fee</span>
                <span className="text-[#FFE600] font-mono">{ICPI_FEE} ICPI</span>
              </div>
            </div>
          )}

          {!redeemReceive.length && redeemAmount && parseFloat(redeemAmount) > 0 && (
            <div className="pt-2 border-t border-[#1f1f1f]">
              <div className="flex justify-between text-[10px]">
                <span className="text-[#666666]">Burn Fee</span>
                <span className="text-[#FFE600] font-mono">{ICPI_FEE} ICPI</span>
              </div>
            </div>
          )}

          <Button
            variant="destructive"
            size="sm"
            className="w-full"
            onClick={handleRedeem}
            disabled={isRedeeming || !redeemAmount || parseFloat(redeemAmount) <= 0}
          >
            {isRedeeming ? (
              <>
                <Loader2 className="mr-2 h-3 w-3 animate-spin" />
                Redeeming...
              </>
            ) : (
              'REDEEM ICPI'
            )}
          </Button>
        </CardContent>
      </Card>

      {error && (
        <div className="px-2 py-1 bg-[#FF005520] border border-[#FF0055] text-[#FF0055] text-xs">
          {error}
        </div>
      )}
    </div>
  )
}
