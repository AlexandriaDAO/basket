<script lang="ts">
  import { onMount } from 'svelte';
  import { Loader, Eye, BarChart3, Users, DollarSign, Activity, RefreshCw, TrendingUp, Lock, Layers, ChevronDown, ChevronUp, Copy, Check, Coins } from 'lucide-svelte';
  import { authService, authStore, isAuthenticated, isAuthLoading, principal } from '../lib/stores/auth';
  import { userLockService, userState, userLockStore } from '../lib/stores/userLock';
  import { lpLockingService, lockingStatsStore, formatUsd, formatCount } from '../lib/services/lpLocking';
  import { analyticsService, type CanisterListItem, type SystemStats } from '../lib/services/analytics';
  import type { LPReply } from '../lib/services/kongSwapDirect';
  
  // Components
  import CombinedAddressInfo from '../lib/components/CombinedAddressInfo.svelte';
  import StateA_NoCanister from '../lib/components/states/StateA_NoCanister.svelte';
  import StateB_Creating from '../lib/components/states/StateB_Creating.svelte';
  import StateC_Ready from '../lib/components/states/StateC_Ready.svelte';
  import StateE_NeedsSetup from '../lib/components/states/StateE_NeedsSetup.svelte';
  import LPPositionCard from '../lib/components/LPPositionCard.svelte';
  import TokenBreakdown from '../lib/components/TokenBreakdown.svelte';
  
  // Analytics state
  let analyticsLoading = false;
  let analyticsError = '';
  let canisterList: CanisterListItem[] = [];
  let systemStats: SystemStats | null = null;
  let tokenBreakdown = new Map<string, {
    totalValue: number;
    lpPools: Set<string>;
    totalTokenAmount: number;
  }>();
  
  // Detail modal state
  let selectedCanister: CanisterListItem | null = null;
  let detailModalOpen = false;
  let detailLoading = false;
  let detailError = '';
  let detailPositions: LPReply[] = [];
  let detailSummary: any = null;
  
  // View state
  let showCanisterTable = true;
  let showTokenBreakdown = true;
  let showGlow = false;
  let initialLoadComplete = false;
  let refreshing = false;
  
  // Copy states for the analytics table
  let copyStates = new Map<string, boolean>();
  
  // User LP state
  $: totalVotingPower = $userLockStore.votingPower;
  $: totalUserValue = totalVotingPower / 100;
  $: userLpPositions = $userLockStore.lpPositions;
  
  // Initialize on mount
  onMount(async () => {
    // Initialize authentication
    try {
      await authService.init();
    } catch (error) {
      console.error('Failed to initialize auth:', error);
    }

    // Load statistics and analytics in parallel
    try {
      await Promise.all([
        lpLockingService.fetchStats(),
        loadAnalytics()
      ]);
    } catch (error) {
      console.error('Failed to load initial data:', error);
    }

    // Enable glow animation
    setTimeout(() => {
      showGlow = true;
    }, 1000);
  });
  
  // Load user lock state when authenticated
  $: if ($isAuthenticated && $principal) {
    if (!initialLoadComplete) {
      userLockStore.update(state => ({ ...state, isLoading: true }));
      initialLoadComplete = true;
    }
    userLockService.loadUserState($principal);
  } else if (!$isAuthenticated) {
    userLockService.reset();
    initialLoadComplete = false;
  }
  
  // Load analytics data
  async function loadAnalytics(forceReload = false) {
    if (!forceReload && canisterList.length > 0) return; // Already loaded

    analyticsLoading = true;
    analyticsError = '';

    try {
      // First get the canister list
      canisterList = await analyticsService.getCanisterList();

      // Calculate system stats - this loads all canister details and updates the cache
      systemStats = await analyticsService.calculateSystemStats(20);

      if (systemStats?.tokenBreakdown) {
        tokenBreakdown = systemStats.tokenBreakdown;
      }

      // Now refresh the canister list to get updated statuses from the cache
      canisterList = await analyticsService.getCanisterList();
    } catch (err) {
      console.error('Failed to load analytics:', err);
      analyticsError = err instanceof Error ? err.message : 'Failed to load analytics data';
    } finally {
      analyticsLoading = false;
    }
  }
  
  // Refresh all data
  async function refreshAllData() {
    refreshing = true;

    // Clear caches
    analyticsService.clearCache();

    // Reload data in parallel with force reload
    try {
      await Promise.all([
        lpLockingService.fetchStats(),
        loadAnalytics(true) // Force reload analytics
      ]);
    } catch (error) {
      console.error('Failed to refresh data:', error);
    } finally {
      refreshing = false;
    }
  }
  
  // Open detail modal for specific canister
  async function openCanisterDetails(canister: CanisterListItem) {
    selectedCanister = canister;
    detailModalOpen = true;
    detailLoading = true;
    detailError = '';
    detailPositions = [];
    
    try {
      const details = await analyticsService.getCanisterDetails(canister.lockCanister);
      detailPositions = details.positions;
      detailSummary = details;
      
      // Update the list item with fresh data
      const index = canisterList.findIndex(item => 
        item.lockCanister.toText() === canister.lockCanister.toText()
      );
      if (index >= 0) {
        canisterList[index] = {
          ...canisterList[index],
          status: details.positions.length > 0 ? 'active' : 'empty',
          lastQueried: Date.now(),
          cachedPositions: details.positions,
          cachedValue: details.totalUSD,
          cachedVotingPower: details.votingPower
        };
      }

      // Don't update token breakdown from cache - keep consistent with system stats
    } catch (err) {
      console.error('Failed to load canister details:', err);
      detailError = err instanceof Error ? err.message : 'Failed to load position details';
    } finally {
      detailLoading = false;
    }
  }
  
  // Close detail modal
  function closeModal() {
    detailModalOpen = false;
    selectedCanister = null;
  }
  
  // Connect/disconnect handler
  async function handleConnect() {
    if ($isAuthenticated) {
      await authService.logout();
    } else {
      try {
        await authService.login();
      } catch (error) {
        console.error('Login failed:', error);
      }
    }
  }
  
  // Format utilities
  function getStatusIcon(status: string): string {
    switch (status) {
      case 'active': return '🟢';
      case 'empty': return '🟡'; 
      case 'error': return '🔴';
      case 'loading': return '🔄';
      default: return '⚫';
    }
  }
  
  function getStatusText(status: string): string {
    switch (status) {
      case 'active': return 'Active';
      case 'empty': return 'No LP Tokens';
      case 'error': return 'Error';
      case 'loading': return 'Loading...';
      default: return 'Unknown';
    }
  }
  
  function openDAOPad() {
    window.open('https://daopad.org', '_blank');
  }
  
  function openKongSwap() {
    window.open('https://kongswap.io/pools', '_blank');
  }
  
  // Copy function for principals in the analytics table
  async function copyPrincipal(text: string, key: string) {
    try {
      await navigator.clipboard.writeText(text);
      copyStates.set(key, true);
      copyStates = copyStates; // Trigger reactivity
      setTimeout(() => {
        copyStates.set(key, false);
        copyStates = copyStates; // Trigger reactivity
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }
  
  // Developer testing controls
  let showDebugPanel = false;
  $: if (typeof window !== 'undefined') {
    showDebugPanel = window.location.hostname === 'localhost' || window.location.hostname.includes('127.0.0.1');
  }
</script>

<svelte:head>
  <title>Kong Locker - LP Token Locking Service</title>
</svelte:head>

<div class="min-h-screen space-y-6">
  <!-- Navigation Bar -->
  <div class="fixed top-4 right-4 z-50 flex items-center gap-3">
    <!-- Docs Button -->
    <a 
      href="/docs" 
      class="bg-gray-800/90 hover:bg-gray-700/90 backdrop-blur-sm px-4 py-2 text-sm font-semibold text-white rounded-xl transition-all duration-200 shadow-lg flex items-center gap-2 border border-gray-700/50 hover:border-gray-600"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
      </svg>
      <span>API Docs</span>
    </a>
    
    <!-- Connect/Disconnect Button -->
    <button 
      class="kong-button-primary px-4 py-2 text-sm font-semibold kong-shine disabled:opacity-50 disabled:cursor-not-allowed shadow-lg"
      on:click={handleConnect}
      disabled={$isAuthLoading}
    >
      {#if $isAuthLoading}
        <div class="flex items-center space-x-2">
          <Loader class="w-4 h-4 animate-spin" />
          <span>Connecting...</span>
        </div>
      {:else if $isAuthenticated}
        <span>Disconnect</span>
      {:else}
        <span>Connect Wallet</span>
      {/if}
    </button>
  </div>

  <!-- Main Dashboard -->
  <div class="max-w-7xl mx-auto px-4 pt-20">
    <div class="kong-panel space-y-6">
      <!-- Dashboard Header -->
      <div class="flex items-center justify-between border-b border-kong-border/30 pb-4">
        <h2 class="text-xl font-semibold text-kong-text-primary">Kong Locker Dashboard</h2>
        
        <!-- Refresh Button -->
        <button 
          on:click={refreshAllData}
          class="flex items-center space-x-2 px-4 py-2 text-sm text-kong-accent-green hover:bg-kong-accent-green/10 border border-kong-accent-green/30 hover:border-kong-accent-green/50 rounded-lg transition-all duration-200"
          disabled={refreshing || $lockingStatsStore.isLoading}
        >
          <RefreshCw class="w-4 h-4 {refreshing || $lockingStatsStore.isLoading ? 'animate-spin' : ''}" />
          <span>{refreshing || $lockingStatsStore.isLoading ? 'Refreshing...' : 'Refresh'}</span>
        </button>
      </div>
      
      <!-- Combined Address Info (when authenticated) -->
      {#if $isAuthenticated}
        <CombinedAddressInfo />
      {/if}
      
      <!-- Unified Platform Overview Grid -->
      <div class="bg-gray-900/30 border border-gray-800/40 rounded-xl p-6 space-y-4">
        <h3 class="text-lg font-semibold text-kong-text-primary flex items-center">
          <BarChart3 class="w-5 h-5 mr-2 text-kong-accent-blue" />
          Platform Overview
        </h3>
        
        <!-- Stats Grid - 6 columns on desktop, 3 on tablet, 2 on mobile -->
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
          <!-- Total Participants -->
          <div class="bg-gradient-to-br from-kong-accent-blue/5 to-kong-accent-blue/10 border border-kong-accent-blue/20 rounded-lg p-3">
            <div class="flex items-center space-x-1 mb-1">
              <Users class="w-4 h-4 text-kong-accent-blue" />
              <span class="text-xs text-kong-text-secondary">Users</span>
            </div>
            {#if $lockingStatsStore.isLoading || analyticsLoading}
              <div class="w-5 h-5 border-2 border-kong-accent-blue/30 border-t-kong-accent-blue rounded-full animate-spin"></div>
            {:else}
              <div class="text-xl font-bold text-kong-accent-blue">
                {formatCount($lockingStatsStore.totalPositions)}
              </div>
            {/if}
          </div>
          
          <!-- Total Value Locked -->
          <div class="bg-gradient-to-br from-kong-accent-green/5 to-kong-accent-green/10 border border-kong-accent-green/20 rounded-lg p-3">
            <div class="flex items-center space-x-1 mb-1">
              <DollarSign class="w-4 h-4 text-kong-accent-green" />
              <span class="text-xs text-kong-text-secondary">Locked</span>
            </div>
            {#if $lockingStatsStore.isLoading || analyticsLoading}
              <div class="w-5 h-5 border-2 border-kong-accent-green/30 border-t-kong-accent-green rounded-full animate-spin"></div>
            {:else}
              <div class="text-xl font-bold kong-gradient-text">
                {formatUsd($lockingStatsStore.totalValueLocked)}
              </div>
            {/if}
          </div>
          
          <!-- Your Voting Power (if authenticated and has liquidity) -->
          {#if $userState === 'has-liquidity'}
            <div class="bg-gradient-to-br from-kong-accent-purple/5 to-kong-accent-purple/10 border border-kong-accent-purple/20 rounded-lg p-3">
              <div class="flex items-center space-x-1 mb-1">
                <TrendingUp class="w-4 h-4 text-kong-accent-purple" />
                <span class="text-xs text-kong-text-secondary">Your Power</span>
              </div>
              <div class="text-xl font-bold text-kong-accent-purple">
                {totalVotingPower.toLocaleString()}
              </div>
            </div>
            
            <!-- Your LP Pools -->
            <div class="bg-gradient-to-br from-kong-accent-orange/5 to-kong-accent-orange/10 border border-kong-accent-orange/20 rounded-lg p-3">
              <div class="flex items-center space-x-1 mb-1">
                <Layers class="w-4 h-4 text-kong-accent-orange" />
                <span class="text-xs text-kong-text-secondary">Your Pools</span>
              </div>
              <div class="text-xl font-bold text-kong-accent-orange">
                {userLpPositions.length}
              </div>
            </div>
            
            <!-- Your Value -->
            <div class="bg-gradient-to-br from-kong-accent-blue/5 to-kong-accent-blue/10 border border-kong-accent-blue/20 rounded-lg p-3">
              <div class="flex items-center space-x-1 mb-1">
                <Lock class="w-4 h-4 text-kong-accent-blue" />
                <span class="text-xs text-kong-text-secondary">Your Value</span>
              </div>
              <div class="text-xl font-bold text-kong-accent-blue">
                {formatUsd(totalUserValue)}
              </div>
            </div>
          {/if}
          
          <!-- System Stats (when analytics loaded) -->
          {#if systemStats}
            <!-- Average Position -->
            <div class="bg-gradient-to-br from-gray-500/5 to-gray-500/10 border border-gray-500/20 rounded-lg p-3">
              <div class="flex items-center space-x-1 mb-1">
                <BarChart3 class="w-4 h-4 text-gray-400" />
                <span class="text-xs text-kong-text-secondary">Avg/User</span>
              </div>
              <div class="text-xl font-bold text-gray-300">
                ${(systemStats.estimatedTotalValue / systemStats.totalCanisters).toFixed(0)}
              </div>
            </div>
            
            {#if !($userState === 'has-liquidity')}
              <!-- System Health (only when user doesn't have liquidity) -->
              <div class="bg-gradient-to-br from-kong-accent-purple/5 to-kong-accent-purple/10 border border-kong-accent-purple/20 rounded-lg p-3">
                <div class="flex items-center space-x-1 mb-1">
                  <Activity class="w-4 h-4 text-kong-accent-purple" />
                  <span class="text-xs text-kong-text-secondary">Health</span>
                </div>
                <div class="text-xl font-bold text-kong-accent-purple">
                  {Math.round((canisterList.filter(c => c.status !== 'error').length / canisterList.length) * 100)}%
                </div>
              </div>
            {/if}
          {/if}
        </div>
        
        {#if $lockingStatsStore.lastUpdated}
          <div class="text-center pt-2">
            <p class="text-xs text-kong-text-secondary">
              Last updated: {$lockingStatsStore.lastUpdated.toLocaleString()}
            </p>
          </div>
        {/if}
      </div>
      
      <!-- User State Section (Non-liquidity states) -->
      {#if $userState !== 'has-liquidity'}
        <div class="border-t border-kong-border/30 pt-6">
          {#if $userState === 'loading'}
            <div class="text-center space-y-4">
              <Loader class="w-8 h-8 animate-spin mx-auto text-kong-accent-green" />
              <h3 class="text-lg font-semibold text-kong-text-primary">Loading Your Status</h3>
              <p class="text-sm text-kong-text-secondary">Checking your lock canister status...</p>
            </div>
          {:else if $userState === 'not-connected'}
            <StateA_NoCanister {showGlow} integrated={true} />
          {:else if $userState === 'no-canister'}
            <StateA_NoCanister authenticated={true} {showGlow} integrated={true} />
          {:else if $userState === 'creating'}
            <StateB_Creating integrated={true} />
          {:else if $userState === 'ready'}
            <StateC_Ready integrated={true} />
          {:else if $userState === 'needs-setup'}
            <StateE_NeedsSetup integrated={true} />
          {/if}
        </div>
      {/if}
      
      <!-- User LP Positions Table (when user has liquidity) -->
      {#if $userState === 'has-liquidity' && userLpPositions.length > 0}
        <div class="bg-gray-900/30 border border-gray-800/40 rounded-xl p-6 space-y-4 mt-6">
          <div class="flex items-center justify-between">
            <h4 class="text-lg font-semibold text-kong-text-primary flex items-center">
              <Lock class="w-5 h-5 mr-2 text-kong-accent-orange" />
              Your Locked LP Positions ({userLpPositions.length})
            </h4>
            <div class="flex gap-2">
              <button 
                on:click={openDAOPad}
                class="px-3 py-1.5 text-xs bg-kong-accent-green/20 text-kong-accent-green border border-kong-accent-green/30 rounded-lg hover:bg-kong-accent-green/30 transition-colors"
              >
                Use in DAOPad →
              </button>
              <button 
                on:click={openKongSwap}
                class="px-3 py-1.5 text-xs bg-kong-accent-blue/20 text-kong-accent-blue border border-kong-accent-blue/30 rounded-lg hover:bg-kong-accent-blue/30 transition-colors"
              >
                Lock More +
              </button>
            </div>
          </div>
          
          <!-- Compact LP Position Table -->
          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-kong-border">
                  <th class="text-left py-2 px-3 text-kong-text-secondary">Pool</th>
                  <th class="text-left py-2 px-3 text-kong-text-secondary">Token A</th>
                  <th class="text-left py-2 px-3 text-kong-text-secondary">Token B</th>
                  <th class="text-right py-2 px-3 text-kong-text-secondary">LP Balance</th>
                  <th class="text-right py-2 px-3 text-kong-text-secondary">USD Value</th>
                  <th class="text-right py-2 px-3 text-kong-text-secondary">% of Total</th>
                </tr>
              </thead>
              <tbody>
                {#each userLpPositions as position}
                  <tr class="border-b border-kong-border/50 hover:bg-kong-bg-secondary/30 transition-colors">
                    <td class="py-2 px-3">
                      <span class="font-medium text-kong-text-primary">{position.symbol}</span>
                    </td>
                    <td class="py-2 px-3">
                      <div class="flex items-center gap-1">
                        <span class="text-kong-accent-blue">{position.symbol_0}</span>
                        <span class="text-xs text-kong-text-secondary">{position.amount_0.toFixed(2)}</span>
                      </div>
                    </td>
                    <td class="py-2 px-3">
                      <div class="flex items-center gap-1">
                        <span class="text-kong-accent-green">{position.symbol_1}</span>
                        <span class="text-xs text-kong-text-secondary">{position.amount_1.toFixed(2)}</span>
                      </div>
                    </td>
                    <td class="py-2 px-3 text-right font-mono text-kong-text-primary">
                      {position.balance.toFixed(4)}
                    </td>
                    <td class="py-2 px-3 text-right font-semibold text-kong-accent-green">
                      ${position.usd_balance.toFixed(2)}
                    </td>
                    <td class="py-2 px-3 text-right">
                      <div class="flex items-center justify-end gap-2">
                        <div class="w-16 bg-kong-bg-secondary rounded-full h-1.5">
                          <div 
                            class="bg-kong-accent-purple h-1.5 rounded-full"
                            style="width: {(position.usd_balance / totalUserValue) * 100}%"
                          ></div>
                        </div>
                        <span class="text-xs text-kong-text-secondary">
                          {((position.usd_balance / totalUserValue) * 100).toFixed(1)}%
                        </span>
                      </div>
                    </td>
                  </tr>
                {/each}
              </tbody>
              <tfoot>
                <tr class="font-semibold">
                  <td colspan="4" class="py-2 px-3 text-kong-text-primary">Total</td>
                  <td class="py-2 px-3 text-right text-kong-accent-green">{formatUsd(totalUserValue)}</td>
                  <td class="py-2 px-3 text-right text-kong-accent-purple">{totalVotingPower.toLocaleString()} VP</td>
                </tr>
              </tfoot>
            </table>
          </div>
          
          <p class="text-xs text-kong-text-secondary text-center">
            LP tokens are permanently locked and can never be withdrawn
          </p>
        </div>
      {/if}
      
      <!-- Analytics Section -->
      <div class="space-y-6 mt-6">
        {#if analyticsLoading}
          <div class="flex items-center justify-center py-8">
            <div class="flex items-center space-x-3">
              <Loader class="w-6 h-6 animate-spin text-kong-accent-blue" />
              <span class="text-kong-text-primary">Loading analytics data...</span>
            </div>
          </div>
        {:else if analyticsError}
          <div class="bg-red-900/20 border border-red-800 rounded-lg p-4">
            <p class="text-sm text-red-400">{analyticsError}</p>
          </div>
        {:else}
          <!-- Token Breakdown -->
          {#if systemStats && tokenBreakdown.size > 0}
            <div class="bg-gray-900/30 border border-gray-800/40 rounded-xl p-6 space-y-4">
              <div class="flex items-center justify-between">
                <h4 class="text-lg font-semibold text-kong-text-primary flex items-center">
                  <Coins class="w-5 h-5 mr-2 text-kong-accent-purple" />
                  Token Distribution ({tokenBreakdown.size} tokens)
                </h4>
                <button
                  on:click={() => showTokenBreakdown = !showTokenBreakdown}
                  class="flex items-center gap-1 px-3 py-1 text-xs text-kong-text-secondary hover:text-kong-text-primary transition-colors"
                >
                  {showTokenBreakdown ? 'Hide' : 'Show'} Details
                  {#if showTokenBreakdown}
                    <ChevronUp class="w-3 h-3" />
                  {:else}
                    <ChevronDown class="w-3 h-3" />
                  {/if}
                </button>
              </div>
              
              {#if showTokenBreakdown}
                <TokenBreakdown 
                  tokenData={tokenBreakdown} 
                  totalValue={systemStats.estimatedTotalValue || 0}
                />
              {:else}
                <div class="text-sm text-kong-text-secondary">
                  Total value locked across {tokenBreakdown.size} different tokens. Click "Show Details" to see breakdown.
                </div>
              {/if}
            </div>
          {/if}
          
          <!-- Canister List Table -->
          <div class="bg-gray-900/30 border border-gray-800/40 rounded-xl p-6 space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="text-lg font-semibold text-kong-text-primary flex items-center">
                  <Users class="w-5 h-5 mr-2 text-kong-accent-blue" />
                  All Lock Canisters ({canisterList.length})
                </h4>
                <p class="text-xs text-kong-text-secondary mt-1">Click any row to view detailed LP positions</p>
              </div>
              <button
                on:click={() => showCanisterTable = !showCanisterTable}
                class="flex items-center gap-1 px-3 py-1 text-xs text-kong-text-secondary hover:text-kong-text-primary transition-colors"
              >
                {showCanisterTable ? 'Hide' : 'Show'} Table
                {#if showCanisterTable}
                  <ChevronUp class="w-3 h-3" />
                {:else}
                  <ChevronDown class="w-3 h-3" />
                {/if}
              </button>
            </div>
            
            {#if showCanisterTable}
              <div class="overflow-x-auto">
                <table class="w-full text-sm">
                  <thead>
                    <tr class="border-b border-kong-border">
                      <th class="text-left py-3 px-4 text-kong-text-secondary">User</th>
                      <th class="text-left py-3 px-4 text-kong-text-secondary">Lock Canister</th>
                      <th class="text-left py-3 px-4 text-kong-text-secondary">Status</th>
                      <th class="text-left py-3 px-4 text-kong-text-secondary">Value ↓</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each canisterList.sort((a, b) => (b.cachedValue || 0) - (a.cachedValue || 0)) as canister}
                      <tr
                        class="border-b border-kong-border/50 hover:bg-kong-bg-secondary/30 transition-colors cursor-pointer"
                        on:click={() => openCanisterDetails(canister)}
                      >
                        <td class="py-3 px-4">
                          <div class="flex items-center space-x-2">
                            {#if canister.username}
                              <span class="text-kong-text-primary text-sm font-medium">@{canister.username}</span>
                            {:else}
                              <code class="text-kong-text-primary font-mono text-xs">{canister.userDisplay}</code>
                            {/if}
                            <button
                              on:click|stopPropagation={() => copyPrincipal(canister.userPrincipal.toString(), `user-${canister.userPrincipal.toString()}`)}
                              class="flex items-center justify-center w-6 h-6 text-xs text-kong-accent-green hover:bg-kong-accent-green/10 rounded transition-all duration-200"
                              title="Copy full principal"
                            >
                              {#if copyStates.get(`user-${canister.userPrincipal.toString()}`)}
                                <Check class="w-3 h-3" />
                              {:else}
                                <Copy class="w-3 h-3" />
                              {/if}
                            </button>
                          </div>
                        </td>
                        <td class="py-3 px-4">
                          <div class="flex items-center space-x-2">
                            <code class="text-kong-text-primary font-mono text-xs">{canister.canisterDisplay}</code>
                            <button
                              on:click|stopPropagation={() => copyPrincipal(canister.lockCanister.toString(), `canister-${canister.lockCanister.toString()}`)}
                              class="flex items-center justify-center w-6 h-6 text-xs text-kong-accent-green hover:bg-kong-accent-green/10 rounded transition-all duration-200"
                              title="Copy full principal"
                            >
                              {#if copyStates.get(`canister-${canister.lockCanister.toString()}`)}
                                <Check class="w-3 h-3" />
                              {:else}
                                <Copy class="w-3 h-3" />
                              {/if}
                            </button>
                          </div>
                        </td>
                        <td class="py-3 px-4">
                          <div class="flex items-center space-x-2">
                            <span>{getStatusIcon(canister.status)}</span>
                            <span class="text-kong-text-primary">{getStatusText(canister.status)}</span>
                          </div>
                        </td>
                        <td class="py-3 px-4">
                          {#if canister.cachedValue !== undefined}
                            <div class="text-kong-accent-green font-semibold">${canister.cachedValue.toFixed(2)}</div>
                          {:else}
                            <span class="text-kong-text-secondary">—</span>
                          {/if}
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {/if}
            
            {#if !showCanisterTable && canisterList.length > 0}
              <div class="text-sm text-kong-text-secondary">
                {canisterList.length} lock canisters created by users. Click "Show Table" to see details.
              </div>
            {/if}
            
            {#if canisterList.length === 0}
              <div class="text-center py-8">
                <div class="text-kong-text-secondary mb-2">No lock canisters found</div>
                <div class="text-xs text-kong-text-secondary">Users haven't created any lock canisters yet</div>
              </div>
            {/if}
          </div>
        {/if}
      </div>
      
      <!-- Error Display -->
      {#if $lockingStatsStore.error || $userLockStore.error}
        <div class="bg-kong-error/10 border-kong-error/20 border rounded-lg p-4">
          <div class="flex items-center justify-between">
            <p class="text-sm text-kong-error">
              {$lockingStatsStore.error || $userLockStore.error}
            </p>
            <button 
              on:click={() => {
                if ($lockingStatsStore.error) lpLockingService.clearError();
                if ($userLockStore.error) userLockService.clearError();
              }}
              class="text-sm text-kong-error/70 hover:text-kong-error ml-4"
            >
              Dismiss
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>
  
  <!-- Developer Debug Panel (only in development) -->
  {#if showDebugPanel && $isAuthenticated}
    <section class="max-w-lg mx-auto px-4 mt-8">
      <div class="kong-panel bg-kong-accent-orange/5 border-kong-accent-orange/20 space-y-4">
        <h3 class="text-sm font-semibold text-kong-accent-orange">🛠️ Developer Debug Panel</h3>
        <p class="text-xs text-kong-text-secondary">Testing controls (only visible in development)</p>
        
        <div class="flex flex-col space-y-2">
          <p class="text-xs text-kong-text-secondary">Current State: <strong class="text-kong-text-primary">{$userState}</strong></p>
          
          <div class="flex space-x-2">
            <button 
              on:click={() => userLockService.enableMockData()}
              class="flex-1 px-3 py-2 text-xs font-medium text-kong-accent-green hover:bg-kong-accent-green/10 border border-kong-accent-green/30 hover:border-kong-accent-green/50 rounded-lg transition-all duration-200"
              disabled={$userLockStore.useMockData}
            >
              Enable Mock Data (State D)
            </button>
            
            <button 
              on:click={() => userLockService.disableMockData()}
              class="flex-1 px-3 py-2 text-xs font-medium text-kong-accent-blue hover:bg-kong-accent-blue/10 border border-kong-accent-blue/30 hover:border-kong-accent-blue/50 rounded-lg transition-all duration-200"
              disabled={!$userLockStore.useMockData}
            >
              Disable Mock Data
            </button>
          </div>
          
          <div class="text-xs text-kong-text-secondary">
            Mock Status: {$userLockStore.useMockData ? '✅ Enabled' : '❌ Disabled'}
          </div>
        </div>
      </div>
    </section>
  {/if}
</div>

<!-- Detail Modal -->
{#if detailModalOpen && selectedCanister}
  <div class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
    <div class="kong-panel max-w-4xl w-full max-h-[90vh] overflow-y-auto space-y-4">
      <!-- Modal Header -->
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-semibold text-kong-text-primary">LP Position Details</h3>
          <p class="text-sm text-kong-text-secondary">
            Lock Canister: <code class="font-mono">{selectedCanister.canisterDisplay}</code>
          </p>
        </div>
        <button 
          on:click={closeModal}
          class="p-2 text-kong-text-secondary hover:text-kong-text-primary rounded transition-colors"
        >
          ✕
        </button>
      </div>

      <!-- Loading State -->
      {#if detailLoading}
        <div class="flex items-center justify-center py-8">
          <div class="flex items-center space-x-3">
            <Loader class="w-6 h-6 animate-spin text-kong-accent-blue" />
            <span class="text-kong-text-primary">Loading LP positions...</span>
          </div>
        </div>
      {:else if detailError}
        <div class="bg-red-900/20 border border-red-800 rounded-lg p-4">
          <div class="flex items-center space-x-3">
            <div class="w-8 h-8 bg-red-900/30 rounded-full flex items-center justify-center">
              <span class="text-red-400 text-sm">!</span>
            </div>
            <div>
              <h4 class="font-semibold text-red-400">Failed to Load Positions</h4>
              <p class="text-sm text-red-300">{detailError}</p>
              {#if detailError.includes('User not found')}
                <p class="text-xs text-red-300 mt-1">
                  This lock canister may not be registered on KongSwap yet.
                </p>
              {/if}
            </div>
          </div>
        </div>
      {:else if detailPositions.length > 0}
        <!-- Position Summary -->
        <div class="bg-kong-bg-secondary/50 border border-kong-border/50 rounded-lg p-4">
          <h4 class="text-lg font-semibold text-kong-text-primary mb-3">Summary</h4>
          <div class="grid grid-cols-3 gap-4 text-center">
            <div>
              <div class="text-2xl font-bold text-kong-accent-blue">{detailPositions.length}</div>
              <div class="text-xs text-kong-text-secondary">LP Pools</div>
            </div>
            <div>
              <div class="text-2xl font-bold text-kong-accent-green">${detailSummary?.totalUSD.toFixed(2) || '0.00'}</div>
              <div class="text-xs text-kong-text-secondary">Total Value</div>
            </div>
            <div>
              <div class="text-2xl font-bold text-kong-accent-purple">{detailSummary?.votingPower.toLocaleString() || '0'}</div>
              <div class="text-xs text-kong-text-secondary">Voting Power</div>
            </div>
          </div>
        </div>

        <!-- Individual Positions -->
        <div class="space-y-3">
          <h4 class="text-lg font-semibold text-kong-text-primary">Individual LP Positions</h4>
          {#each detailPositions as position}
            <LPPositionCard 
              {position} 
              percentOfTotal={(position.usd_balance / detailSummary.totalUSD) * 100}
            />
          {/each}
        </div>
      {:else}
        <!-- No Positions -->
        <div class="text-center py-8">
          <div class="w-16 h-16 bg-kong-accent-orange/20 rounded-full flex items-center justify-center mx-auto mb-4">
            <span class="text-2xl">🔒</span>
          </div>
          <h4 class="text-lg font-semibold text-kong-text-primary mb-2">No LP Tokens Locked</h4>
          <p class="text-kong-text-secondary">
            This lock canister is registered on KongSwap but doesn't have any LP tokens yet.
          </p>
        </div>
      {/if}

      <!-- Modal Footer -->
      <div class="flex justify-end pt-4 border-t border-kong-border/50">
        <button 
          on:click={closeModal}
          class="kong-button-secondary px-6 py-2 text-sm font-medium"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}