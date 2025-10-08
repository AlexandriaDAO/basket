import React from 'react'
import { AllocationChart, TokenAllocation } from './AllocationChart'
import { RebalancingPanel } from './RebalancingPanel'

interface PortfolioDashboardProps {
  portfolioData: {
    portfolioValue: number
    indexPrice: number
    totalSupply: number
    apy: number
    dailyChange: number
    priceChange: number
  }
  allocations: TokenAllocation[]
  rebalancingData: {
    nextRebalance: Date
    nextAction: any
    history: any[]
    isRebalancing: boolean
    autoEnabled: boolean
  }
  onManualRebalance: () => Promise<void>
  onToggleAutoRebalance: (enabled: boolean) => void
}

export const PortfolioDashboard: React.FC<PortfolioDashboardProps> = ({
  portfolioData,
  allocations,
  rebalancingData,
  onManualRebalance,
  onToggleAutoRebalance,
}) => {
  return (
    <div className="space-y-2">
      <AllocationChart data={allocations} />
      <RebalancingPanel
        nextRebalance={rebalancingData.nextRebalance}
        nextAction={rebalancingData.nextAction}
        rebalanceHistory={rebalancingData.history}
        isRebalancing={rebalancingData.isRebalancing}
        onManualRebalance={onManualRebalance}
        onToggleAutoRebalance={onToggleAutoRebalance}
        autoRebalanceEnabled={rebalancingData.autoEnabled}
      />
    </div>
  )
}