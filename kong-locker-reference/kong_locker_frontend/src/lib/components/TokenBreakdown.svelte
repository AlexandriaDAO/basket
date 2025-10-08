<script lang="ts">
  import { TrendingUp, Coins, DollarSign } from 'lucide-svelte';
  import { getTokenColors, generateTokenIcon } from '../utils/colorGenerator';
  
  export let tokenData: Map<string, {
    totalValue: number;
    lpPools: Set<string>;
    totalTokenAmount: number;
  }> = new Map();
  export let totalValue: number = 0;
  
  // Convert map to sorted array for display - completely dynamic
  $: sortedTokens = tokenData && tokenData.size > 0
    ? Array.from(tokenData.entries())
        .map(([symbol, data]) => ({
          symbol,
          ...data,
          percentage: totalValue > 0 ? (data.totalValue / totalValue) * 100 : 0,
          dynamicColors: getTokenColors(symbol),
          dynamicIcon: generateTokenIcon(symbol),
          tvlContribution: data.totalValue * 2 // TVL is double the token value since each token is paired
        }))
        .sort((a, b) => b.tvlContribution - a.tvlContribution) // Sort by TVL contribution
        .filter(token => token.totalValue > 0)
    : [];
  
  
  // Format large numbers
  function formatNumber(num: number): string {
    if (num >= 1000000) {
      return `${(num / 1000000).toFixed(2)}M`;
    } else if (num >= 1000) {
      return `${(num / 1000).toFixed(1)}K`;
    }
    return num.toFixed(2);
  }
</script>

{#if sortedTokens.length === 0}
    <div class="text-center py-8">
      <div class="text-kong-text-secondary">No token data available yet</div>
      <div class="text-xs text-kong-text-secondary mt-1">
        Token data will appear automatically when lock canisters are loaded
      </div>
    </div>
  {:else}
    <!-- All Tokens Table - Completely Dynamic -->
    <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-kong-border/50">
              <th class="text-left py-2 px-3 text-kong-text-secondary font-medium">Token</th>
              <th class="text-right py-2 px-3 text-kong-text-secondary font-medium">TVL Contribution</th>
              <th class="text-right py-2 px-3 text-kong-text-secondary font-medium">Token Value</th>
              <th class="text-right py-2 px-3 text-kong-text-secondary font-medium">% of Total</th>
              <th class="text-right py-2 px-3 text-kong-text-secondary font-medium">LP Pools</th>
              <th class="text-left py-2 px-3 text-kong-text-secondary font-medium">Distribution</th>
            </tr>
          </thead>
          <tbody>
            {#each sortedTokens as token}
              <tr class="border-b border-kong-border/30 hover:bg-kong-bg-secondary/30 transition-colors">
                <td class="py-2 px-3">
                  <div class="flex items-center space-x-2">
                    <span class="text-sm">{token.dynamicIcon}</span>
                    <span
                      class="font-medium"
                      style="color: {token.dynamicColors.primary}"
                    >
                      {token.symbol}
                    </span>
                  </div>
                </td>
                <td class="text-right py-2 px-3">
                  <span class="font-semibold text-kong-accent-green">
                    ${formatNumber(token.tvlContribution)}
                  </span>
                </td>
                <td class="text-right py-2 px-3">
                  <span class="text-kong-text-secondary">
                    ${formatNumber(token.totalValue)}
                  </span>
                </td>
                <td class="text-right py-2 px-3 text-kong-text-secondary">
                  {token.percentage.toFixed(2)}%
                </td>
                <td class="text-right py-2 px-3 text-kong-text-secondary">
                  {token.lpPools.size}
                </td>
                <td class="py-2 px-3">
                  <div class="w-full bg-kong-bg-secondary rounded-full h-2 overflow-hidden">
                    <div
                      class="h-full transition-all duration-300"
                      style="
                        width: {Math.min(token.percentage, 100)}%;
                        background: {token.dynamicColors.primary};
                      "
                    ></div>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
  {/if}

