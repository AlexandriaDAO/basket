<script lang="ts">
  import { TrendingUp, Shield, RefreshCw } from 'lucide-svelte';
  import { lpLockingService, lockingStatsStore, formatUsd, formatCount } from '../services/lpLocking';
  
  async function refreshStats() {
    try {
      await lpLockingService.fetchStats();
    } catch (error) {
      console.error('Failed to refresh stats:', error);
    }
  }
</script>

<!-- Statistics Dashboard -->
<section class="space-y-4">
  <!-- Refresh Button -->
  <div class="flex justify-center">
    <button 
      on:click={refreshStats}
      class="flex items-center space-x-2 px-4 py-2 text-sm text-kong-accent-green hover:bg-kong-accent-green/10 border border-kong-accent-green/30 hover:border-kong-accent-green/50 rounded-lg transition-all duration-200"
      disabled={$lockingStatsStore.isLoading}
    >
      <RefreshCw class="w-4 h-4 {$lockingStatsStore.isLoading ? 'animate-spin' : ''}" />
      <span>{$lockingStatsStore.isLoading ? 'Refreshing...' : 'Refresh Stats'}</span>
    </button>
  </div>
  
  <!-- Stats Error -->
  {#if $lockingStatsStore.error}
    <div class="kong-panel bg-kong-error/10 border-kong-error/20 text-center">
      <p class="text-sm text-kong-error">{$lockingStatsStore.error}</p>
      <button 
        on:click={lpLockingService.clearError}
        class="text-xs text-kong-error/70 hover:text-kong-error mt-2"
      >
        Dismiss
      </button>
    </div>
  {/if}
  
  <!-- Statistics Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <div class="kong-panel text-center">
      <div class="flex items-center justify-center mb-3">
        <Shield class="w-8 h-8 text-kong-accent-blue mr-2" />
        <h3 class="text-lg font-semibold text-kong-text-primary">LP Positions</h3>
      </div>
      {#if $lockingStatsStore.isLoading}
        <div class="flex items-center justify-center">
          <div class="w-6 h-6 border-2 border-kong-accent-blue/30 border-t-kong-accent-blue rounded-full animate-spin"></div>
        </div>
      {:else}
        <p class="text-3xl font-bold text-kong-accent-blue">
          {formatCount($lockingStatsStore.totalPositions)}
        </p>
      {/if}
      <p class="text-xs text-kong-text-secondary mt-2">Unique locked positions</p>
    </div>
    
    <div class="kong-panel text-center">
      <div class="flex items-center justify-center mb-3">
        <TrendingUp class="w-8 h-8 text-kong-accent-green mr-2" />
        <h3 class="text-lg font-semibold text-kong-text-primary">Total Value Locked</h3>
      </div>
      {#if $lockingStatsStore.isLoading}
        <div class="flex items-center justify-center">
          <div class="w-6 h-6 border-2 border-kong-accent-green/30 border-t-kong-accent-green rounded-full animate-spin"></div>
        </div>
      {:else}
        <p class="text-3xl font-bold kong-gradient-text">
          {formatUsd($lockingStatsStore.totalValueLocked)}
        </p>
      {/if}
      <p class="text-xs text-kong-text-secondary mt-2">USD value of locked LP tokens</p>
    </div>
  </div>
  
  <!-- Last Updated -->
  {#if $lockingStatsStore.lastUpdated}
    <div class="text-center">
      <p class="text-xs text-kong-text-secondary">
        Last updated: {$lockingStatsStore.lastUpdated.toLocaleString()}
      </p>
    </div>
  {/if}
</section>