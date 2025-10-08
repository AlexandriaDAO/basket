import React, { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from './ui/card'
import { Button } from './ui/button'
import { Input } from './ui/input'
import { Label } from './ui/label'
import { Loader2, X } from 'lucide-react'
import { Principal } from '@dfinity/principal'
import { UserTokenBalance } from '@/hooks/useICPI'

interface SendTokenModalProps {
  token: UserTokenBalance
  userPrincipal: string
  onClose: () => void
  onSend: (recipient: string, amount: string) => Promise<void>
}

export const SendTokenModal: React.FC<SendTokenModalProps> = ({
  token,
  userPrincipal,
  onClose,
  onSend,
}) => {
  const [recipient, setRecipient] = useState('')
  const [amount, setAmount] = useState('')
  const [isSending, setIsSending] = useState(false)
  const [error, setError] = useState('')

  const handleSend = async () => {
    setError('')

    // Validation
    if (!recipient.trim()) {
      setError('Recipient address is required')
      return
    }

    // Validate principal format using @dfinity/principal
    try {
      Principal.fromText(recipient)
    } catch {
      setError('Invalid principal format')
      return
    }

    // Warn if sending to self (but allow it)
    if (recipient === userPrincipal) {
      console.warn('User is sending tokens to themselves')
    }

    if (!amount || parseFloat(amount) <= 0) {
      setError('Invalid amount')
      return
    }

    if (parseFloat(amount) > token.balanceFormatted) {
      setError('Insufficient balance')
      return
    }

    setIsSending(true)
    try {
      await onSend(recipient, amount)
      onClose()
    } catch (err: any) {
      setError(err.message || 'Transfer failed')
    } finally {
      setIsSending(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50">
      <Card className="w-full max-w-md border-[#1f1f1f]">
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <CardTitle className="text-sm">SEND {token.symbol}</CardTitle>
            <Button
              variant="ghost"
              size="sm"
              onClick={onClose}
              className="h-6 w-6 p-0"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        </CardHeader>

        <CardContent className="p-3 space-y-3">
          {/* Balance Display */}
          <div className="bg-[#0a0a0a] p-2 border border-[#1f1f1f]">
            <div className="flex justify-between text-xs">
              <span className="text-[#666666]">Available Balance</span>
              <span className="text-white font-mono">
                {token.balanceFormatted.toFixed(6)} {token.symbol}
              </span>
            </div>
          </div>

          {/* Recipient Input */}
          <div className="space-y-1">
            <Label className="text-xs">Recipient Address</Label>
            <Input
              type="text"
              placeholder="xxxxx-xxxxx-xxxxx-xxxxx-cai"
              value={recipient}
              onChange={(e) => setRecipient(e.target.value)}
              className="font-mono text-xs"
            />
            <p className="text-[9px] text-[#666666]">
              Enter the recipient's principal ID
            </p>
          </div>

          {/* Amount Input */}
          <div className="space-y-1">
            <Label className="text-xs">Amount</Label>
            <div className="flex gap-2">
              <Input
                type="number"
                placeholder="0.00"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                className="flex-1 text-right font-mono"
              />
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setAmount(token.balanceFormatted.toString())}
                className="px-2"
              >
                MAX
              </Button>
            </div>
          </div>

          {/* Error Display */}
          {error && (
            <div className="px-2 py-1 bg-[#FF005520] border border-[#FF0055] text-[#FF0055] text-xs">
              {error}
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex gap-2 pt-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={onClose}
              className="flex-1"
              disabled={isSending}
            >
              CANCEL
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleSend}
              className="flex-1"
              disabled={isSending || !recipient || !amount}
            >
              {isSending ? (
                <>
                  <Loader2 className="mr-2 h-3 w-3 animate-spin" />
                  SENDING...
                </>
              ) : (
                'SEND'
              )}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
