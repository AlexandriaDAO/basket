<script lang="ts">
  import type { LPPosition } from '../stores/userLock';

  export let position: LPPosition;
  export let percentOfTotal: number;

  function formatTokenAmount(amount: number, symbol: string): string {
    if (amount >= 1000000) {
      return `${(amount / 1000000).toFixed(2)}M ${symbol}`;
    } else if (amount >= 1000) {
      return `${(amount / 1000).toFixed(2)}K ${symbol}`;
    } else if (amount >= 1) {
      return `${amount.toFixed(2)} ${symbol}`;
    } else {
      return `${amount.toFixed(6)} ${symbol}`;
    }
  }

  function formatDate(timestamp: bigint): string {
    try {
      return new Date(Number(timestamp)).toLocaleDateString();
    } catch {
      return 'Unknown';
    }
  }
</script>

<div class="bg-kong-bg-secondary/50 border border-kong-border/50 rounded-lg p-4">
  <div class="flex justify-between items-start mb-3">
    <div>
      <h5 class="font-semibold text-kong-text-primary">{position.name}</h5>
      <p class="text-sm text-kong-text-secondary">Pool: {position.symbol}</p>
    </div>
    <div class="text-right">
      <div class="text-lg font-bold text-kong-accent-green">${position.usd_balance.toFixed(2)}</div>
      <div class="text-xs text-kong-text-secondary">{percentOfTotal.toFixed(1)}% of total</div>
    </div>
  </div>
  
  <div class="grid grid-cols-1 md:grid-cols-3 gap-3 text-sm">
    <!-- LP Tokens -->
    <div class="bg-kong-bg-tertiary/50 rounded p-3">
      <div class="text-xs text-kong-text-secondary mb-1">LP Tokens</div>
      <div class="font-semibold text-kong-text-primary">{formatTokenAmount(position.balance, 'LP')}</div>
    </div>
    
    <!-- First Token -->
    <div class="bg-kong-bg-tertiary/50 rounded p-3">
      <div class="text-xs text-kong-text-secondary mb-1">{position.symbol_0}</div>
      <div class="font-semibold text-kong-accent-blue">{formatTokenAmount(position.amount_0, position.symbol_0)}</div>
      <div class="text-xs text-kong-text-secondary">${position.usd_amount_0.toFixed(2)}</div>
    </div>
    
    <!-- Second Token -->
    <div class="bg-kong-bg-tertiary/50 rounded p-3">
      <div class="text-xs text-kong-text-secondary mb-1">{position.symbol_1}</div>
      <div class="font-semibold text-kong-accent-orange">{formatTokenAmount(position.amount_1, position.symbol_1)}</div>
      <div class="text-xs text-kong-text-secondary">${position.usd_amount_1.toFixed(2)}</div>
    </div>
  </div>
  
  <div class="mt-3 pt-3 border-t border-kong-border/50">
    <div class="text-xs text-kong-text-secondary">
      Token ID: {position.lp_token_id.toString()} • 
      Last Updated: {formatDate(position.ts)}
    </div>
  </div>
</div>