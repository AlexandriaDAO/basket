import React, { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from './ui/card'
import { Alert, AlertDescription, AlertTitle } from './ui/alert'
import { Badge } from './ui/badge'
import { Button } from './ui/button'
import { Switch } from './ui/switch'
import { Separator } from './ui/separator'
import { ScrollArea } from './ui/scroll-area'
import { Activity, RefreshCw, Loader2, Info, Clock, ArrowRight } from 'lucide-react'

interface RebalanceAction {
  type: 'buy' | 'sell'
  token: string
  amount: number
  usdtAmount: number
}

interface RebalanceHistoryItem {
  timestamp: Date
  action: RebalanceAction
  status: 'success' | 'failed'
  txHash?: string
}

interface RebalancingPanelProps {
  nextRebalance: Date
  nextAction: RebalanceAction | null
  rebalanceHistory: RebalanceHistoryItem[]
  isRebalancing: boolean
  onManualRebalance: () => Promise<void>
  onToggleAutoRebalance: (enabled: boolean) => void
  autoRebalanceEnabled: boolean
}

const CountdownTimer: React.FC<{ targetTime: Date }> = ({ targetTime }) => {
  const [timeLeft, setTimeLeft] = useState('')

  useEffect(() => {
    const timer = setInterval(() => {
      const now = new Date().getTime()
      const target = targetTime.getTime()
      const difference = target - now

      if (difference > 0) {
        const hours = Math.floor(difference / (1000 * 60 * 60))
        const minutes = Math.floor((difference % (1000 * 60 * 60)) / (1000 * 60))
        const seconds = Math.floor((difference % (1000 * 60)) / 1000)
        setTimeLeft(`${hours}h ${minutes}m ${seconds}s`)
      } else {
        setTimeLeft('Now')
      }
    }, 1000)

    return () => clearInterval(timer)
  }, [targetTime])

  return <span className="font-mono text-[#00FF41]">{timeLeft}</span>
}

const RebalanceHistoryItem: React.FC<RebalanceHistoryItem> = ({
  timestamp,
  action,
  status,
  txHash,
}) => {
  return (
    <div className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-muted/50 transition-colors">
      <div className="flex items-center gap-3">
        <div className={`w-2 h-2 rounded-full ${
          status === 'success' ? 'bg-green-500' : 'bg-red-500'
        }`} />
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">
              {action.type === 'buy' ? 'Bought' : 'Sold'}
            </span>
            <Badge variant="outline" className="text-xs">
              {action.token}
            </Badge>
            <ArrowRight className="w-3 h-3 text-muted-foreground" />
            <span className="text-sm text-muted-foreground">
              {action.amount.toFixed(4)} tokens
            </span>
          </div>
          <div className="flex items-center gap-2">
            <Clock className="w-3 h-3 text-muted-foreground" />
            <span className="text-xs text-muted-foreground">
              {new Date(timestamp).toLocaleString()}
            </span>
          </div>
        </div>
      </div>
      {txHash && (
        <a
          href={`https://dashboard.internetcomputer.org/transaction/${txHash}`}
          target="_blank"
          rel="noopener noreferrer"
          className="text-xs text-primary hover:underline"
        >
          View
        </a>
      )}
    </div>
  )
}

export const RebalancingPanel: React.FC<RebalancingPanelProps> = ({
  nextRebalance,
  nextAction,
  rebalanceHistory,
  isRebalancing,
  onManualRebalance,
  onToggleAutoRebalance,
  autoRebalanceEnabled,
}) => {
  return (
    <Card className="border-[#1f1f1f]">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm flex items-center justify-between">
          <span>REBALANCING</span>
          <Switch
            checked={autoRebalanceEnabled}
            onCheckedChange={onToggleAutoRebalance}
          />
        </CardTitle>
      </CardHeader>
      <CardContent className="p-3 space-y-2">
        {nextAction && (
          <div className="p-2 border-l-4 border-l-[#00FF41] border-y border-r border-[#00FF4160] bg-[#00FF4120] space-y-1">
            <div className="flex justify-between text-xs">
              <span className="text-[#666666]">ACTION</span>
              <Badge variant={nextAction.type === 'buy' ? 'success' : 'warning'} className="text-[10px] px-1 py-0">
                {nextAction.type.toUpperCase()} {nextAction.token}
              </Badge>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-[#666666]">AMOUNT</span>
              <span className="font-mono text-white">{nextAction.usdtAmount.toFixed(2)} USDT</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-[#666666]">NEXT</span>
              <CountdownTimer targetTime={nextRebalance} />
            </div>
          </div>
        )}

        <Button
          variant="primary"
          size="sm"
          className="w-full"
          onClick={onManualRebalance}
          disabled={isRebalancing}
        >
          {isRebalancing ? (
            <>
              <Loader2 className="mr-2 h-3 w-3 animate-spin" />
              Rebalancing...
            </>
          ) : (
            <>
              <RefreshCw className="mr-2 h-3 w-3" />
              EXECUTE NOW
            </>
          )}
        </Button>

        <Separator />

        <div className="space-y-1">
          <h4 className="text-[10px] text-[#666666] uppercase">Recent Activity</h4>
          <div className="space-y-1 max-h-[150px] overflow-y-auto">
            {rebalanceHistory.length > 0 ? (
              rebalanceHistory.slice(0, 5).map((item, idx) => (
                <div key={idx} className="flex items-center justify-between py-1 text-xs border-b border-[#1f1f1f] last:border-0">
                  <div className="flex items-center gap-2">
                    <div className={`w-1.5 h-1.5 ${
                      item.status === 'success' ? 'bg-[#00FF41]' : 'bg-[#FF0055]'
                    }`} />
                    <span className="text-[#999999]">{item.action.type.toUpperCase()}</span>
                    <span className="text-white font-sans">{item.action.token}</span>
                  </div>
                  <span className="text-[#666666] font-mono text-[10px]">
                    {new Date(item.timestamp).toLocaleTimeString()}
                  </span>
                </div>
              ))
            ) : (
              <div className="text-[10px] text-[#666666] py-2 text-center">
                No activity yet
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}