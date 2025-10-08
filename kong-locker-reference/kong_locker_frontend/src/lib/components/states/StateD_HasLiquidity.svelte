<script lang="ts">
  import { Lock, Copy, ExternalLink, Eye, Check, Plus, ArrowRight, TrendingUp } from 'lucide-svelte';
  import { userLockStore } from '../../stores/userLock';
  import { formatUsd } from '../../services/lpLocking';
  import LPPositionCard from '../LPPositionCard.svelte';
  
  export let integrated = false;
  
  let copySuccess = false;
  let showDetailsModal = false;
  
  // Use real voting power and value from store
  $: totalVotingPower = $userLockStore.votingPower;
  $: totalUsdValue = totalVotingPower / 100; // Convert cents to dollars
  $: lpPositions = $userLockStore.lpPositions;
  $: hasMultiplePositions = lpPositions.length > 1;
  
  // Use real canister details from store
  $: canisterDetails = {
    isBlackholed: $userLockStore.isBlackholed,
    isRegistered: $userLockStore.isRegisteredOnKongSwap,
    hasFunding: $userLockStore.hasIcpFunding,
    cycles: 792_000_000_000, // TODO: Get from detailed status
    memory: 2.1,
    memoryLimit: 4096,
    moduleHash: '0x3f2a1b8c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c',
  };
  
  async function copyAddress() {
    if ($userLockStore.canisterId) {
      try {
        await navigator.clipboard.writeText($userLockStore.canisterId.toString());
        copySuccess = true;
        setTimeout(() => {
          copySuccess = false;
        }, 2000);
      } catch (error) {
        console.error('Failed to copy address:', error);
      }
    }
  }
  
  function openDAOPad() {
    window.open('https://daopad.org', '_blank');
  }
  
  function openKongSwap() {
    window.open('https://kongswap.io/pools', '_blank');
  }
  
  function truncateAddress(address: string): string {
    if (address.length <= 25) return address;
    return `${address.substring(0, 12)}...${address.substring(address.length - 8)}`;
  }
  
  function formatCycles(cycles: number): string {
    return (cycles / 1_000_000_000).toFixed(1) + 'B';
  }
</script>

<div class="{integrated ? 'border-t border-kong-border/30 pt-6 space-y-6' : 'max-w-4xl mx-auto space-y-4'}">
  
  <!-- LP Positions Details -->
  <div class="{integrated ? 'space-y-4' : 'kong-panel space-y-4'}">
    <!-- Address info is now shown in the CombinedAddressInfo component above -->
    
    <!-- Your Voting Power Dashboard -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      <!-- Main Voting Power -->
      <div class="bg-gradient-to-br from-kong-accent-green/10 to-kong-accent-green/5 border border-kong-accent-green/20 rounded-lg p-6">
        <div class="flex items-center space-x-3 mb-4">
          <div class="p-2 bg-kong-accent-green/20 rounded-lg">
            <TrendingUp class="w-6 h-6 text-kong-accent-green" />
          </div>
          <div>
            <h3 class="text-base font-medium text-kong-text-primary">Voting Power</h3>
            <p class="text-sm text-kong-text-secondary">Total governance weight</p>
          </div>
        </div>
        <div class="text-3xl font-bold kong-gradient-text">
          {totalVotingPower.toLocaleString()}
        </div>
        <p class="text-sm text-kong-text-secondary mt-1">
          Based on {formatUsd(totalUsdValue)} locked
        </p>
      </div>
      
      <!-- Pool Count -->
      <div class="bg-gradient-to-br from-kong-accent-blue/10 to-kong-accent-blue/5 border border-kong-accent-blue/20 rounded-lg p-6">
        <div class="flex items-center space-x-3 mb-4">
          <div class="p-2 bg-kong-accent-blue/20 rounded-lg">
            <Lock class="w-6 h-6 text-kong-accent-blue" />
          </div>
          <div>
            <h3 class="text-base font-medium text-kong-text-primary">LP Pools</h3>
            <p class="text-sm text-kong-text-secondary">Locked positions</p>
          </div>
        </div>
        <div class="text-3xl font-bold text-kong-accent-blue">
          {lpPositions.length}
        </div>
        <p class="text-sm text-kong-text-secondary mt-1">
          {hasMultiplePositions ? 'Different pools' : 'Single pool'}
        </p>
      </div>
      
      <!-- Total Value -->
      <div class="bg-gradient-to-br from-kong-accent-orange/10 to-kong-accent-orange/5 border border-kong-accent-orange/20 rounded-lg p-6">
        <div class="flex items-center space-x-3 mb-4">
          <div class="p-2 bg-kong-accent-orange/20 rounded-lg">
            <svg class="w-6 h-6 text-kong-accent-orange" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1" />
            </svg>
          </div>
          <div>
            <h3 class="text-base font-medium text-kong-text-primary">Total Value</h3>
            <p class="text-sm text-kong-text-secondary">USD locked forever</p>
          </div>
        </div>
        <div class="text-3xl font-bold text-kong-accent-orange">
          {formatUsd(totalUsdValue)}
        </div>
        <p class="text-sm text-kong-text-secondary mt-1">
          Permanently secured
        </p>
      </div>
    </div>
  </div>
  
  <!-- Locked Positions -->
  <div class="{integrated ? 'space-y-4 border-t border-kong-border/20 pt-6' : 'kong-panel space-y-4'}">
    <h4 class="text-{integrated ? 'base' : 'lg'} font-semibold text-kong-text-primary flex items-center">
      <Lock class="w-5 h-5 mr-2 text-kong-accent-orange" />
      Locked LP Positions ({lpPositions.length})
    </h4>
    
    {#if lpPositions.length > 0}
      <div class="space-y-3">
        {#each lpPositions as position}
          <LPPositionCard 
            {position} 
            percentOfTotal={(position.usd_balance / totalUsdValue) * 100}
          />
        {/each}
      </div>
    {:else}
      <div class="bg-kong-bg-secondary/50 border border-kong-border/50 rounded-lg p-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <div class="w-12 h-12 bg-kong-accent-green/20 rounded-full flex items-center justify-center">
              <Lock class="w-6 h-6 text-kong-accent-green" />
            </div>
            <div>
              <div class="font-semibold text-kong-text-primary">LP Tokens Locked</div>
              <div class="text-sm text-kong-text-secondary">Permanently locked in canister</div>
              <div class="text-xs text-kong-text-secondary">Canister: {$userLockStore.canisterId ? truncateAddress($userLockStore.canisterId.toString()) : ''}</div>
            </div>
          </div>
          
          <div class="text-right">
            <div class="text-lg font-bold text-kong-text-primary">{formatUsd(totalUsdValue)}</div>
            <div class="text-sm text-kong-accent-green">{totalVotingPower} voting power</div>
          </div>
        </div>
      </div>
    {/if}
    
    <p class="text-xs text-kong-text-secondary text-center">
      LP tokens are permanently locked and can never be withdrawn
    </p>
  </div>
  
  <!-- Action Buttons -->
  <div class="{integrated ? 'border-t border-kong-border/20 pt-6' : ''} grid grid-cols-1 md:grid-cols-2 gap-4">
    <button 
      on:click={openDAOPad}
      class="{integrated ? 'bg-kong-bg-secondary/30 hover:bg-kong-accent-green/5 border border-kong-border/30 hover:border-kong-accent-green/50 rounded-lg' : 'kong-panel hover:bg-kong-accent-green/5 border-kong-accent-green/30 hover:border-kong-accent-green/50'} transition-all duration-200 cursor-pointer group"
    >
      <div class="flex items-center justify-between p-2">
        <div class="flex items-center space-x-3">
          <div class="p-2 bg-kong-accent-green/20 rounded-full group-hover:bg-kong-accent-green/30 transition-colors">
            <ArrowRight class="w-5 h-5 text-kong-accent-green" />
          </div>
          <div class="text-left">
            <div class="font-semibold text-kong-text-primary">Use in DAOPad</div>
            <div class="text-sm text-kong-text-secondary">Vote on Alexandria DAO proposals</div>
          </div>
        </div>
        <ExternalLink class="w-4 h-4 text-kong-text-secondary group-hover:text-kong-accent-green transition-colors" />
      </div>
    </button>
    
    <button 
      on:click={openKongSwap}
      class="{integrated ? 'bg-kong-bg-secondary/30 hover:bg-kong-accent-blue/5 border border-kong-border/30 hover:border-kong-accent-blue/50 rounded-lg' : 'kong-panel hover:bg-kong-accent-blue/5 border-kong-accent-blue/30 hover:border-kong-accent-blue/50'} transition-all duration-200 cursor-pointer group"
    >
      <div class="flex items-center justify-between p-2">
        <div class="flex items-center space-x-3">
          <div class="p-2 bg-kong-accent-blue/20 rounded-full group-hover:bg-kong-accent-blue/30 transition-colors">
            <Plus class="w-5 h-5 text-kong-accent-blue" />
          </div>
          <div class="text-left">
            <div class="font-semibold text-kong-text-primary">Lock More</div>
            <div class="text-sm text-kong-text-secondary">Add more LP tokens to increase power</div>
          </div>
        </div>
        <ExternalLink class="w-4 h-4 text-kong-text-secondary group-hover:text-kong-accent-blue transition-colors" />
      </div>
    </button>
  </div>
  
</div>

<!-- Details Modal -->
{#if showDetailsModal}
  <div class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
    <div class="kong-panel max-w-md w-full space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold text-kong-text-primary">Lock Canister Security Details</h3>
        <button 
          on:click={() => showDetailsModal = false}
          class="p-1 text-kong-text-secondary hover:text-kong-text-primary rounded"
        >
          ✕
        </button>
      </div>
      
      <div class="space-y-4 text-sm">
        <!-- Blackhole Status -->
        <div class="flex items-center justify-between p-3 bg-kong-accent-green/10 border border-kong-accent-green/30 rounded-lg">
          <div class="flex items-center space-x-2">
            <div class="w-2 h-2 bg-kong-accent-green rounded-full"></div>
            <span class="font-medium text-kong-text-primary">Blackholed</span>
          </div>
          <span class="text-xs text-kong-text-secondary">No controllers - permanent</span>
        </div>
        
        <!-- Cycles -->
        <div class="flex items-center justify-between p-3 bg-kong-accent-blue/10 border border-kong-accent-blue/30 rounded-lg">
          <div class="flex items-center space-x-2">
            <div class="w-2 h-2 bg-kong-accent-blue rounded-full"></div>
            <span class="font-medium text-kong-text-primary">Cycles</span>
          </div>
          <div class="text-right">
            <div class="font-mono text-kong-text-primary">{formatCycles(canisterDetails.cycles)}</div>
            <div class="text-xs text-kong-text-secondary">Sufficient for ~3 years</div>
          </div>
        </div>
        
        <!-- Memory -->
        <div class="flex items-center justify-between p-3 bg-kong-accent-blue/10 border border-kong-accent-blue/30 rounded-lg">
          <div class="flex items-center space-x-2">
            <div class="w-2 h-2 bg-kong-accent-blue rounded-full"></div>
            <span class="font-medium text-kong-text-primary">Memory</span>
          </div>
          <div class="text-right">
            <div class="font-mono text-kong-text-primary">{canisterDetails.memory} MB / {canisterDetails.memoryLimit} MB</div>
            <div class="text-xs text-kong-text-secondary">Operating normally</div>
          </div>
        </div>
        
        <!-- Module Hash -->
        <div class="space-y-2">
          <span class="font-medium text-kong-text-primary">Module Hash:</span>
          <p class="font-mono text-xs text-kong-text-secondary bg-kong-bg-tertiary/50 p-2 rounded border break-all">
            {canisterDetails.moduleHash.substring(0, 32)}...
          </p>
        </div>
        
        <!-- Security Statement -->
        <div class="bg-kong-accent-green/10 border border-kong-accent-green/30 rounded-lg p-3">
          <p class="text-xs text-kong-text-secondary">
            <strong class="text-kong-text-primary">This canister is autonomous</strong> and cannot be modified by anyone, 
            including the original creators. Your locked tokens are permanently secure.
          </p>
        </div>
      </div>
      
      <div class="flex justify-end pt-3 border-t border-kong-border/50">
        <button 
          on:click={() => showDetailsModal = false}
          class="kong-button-secondary px-6 py-2 text-sm font-medium"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}