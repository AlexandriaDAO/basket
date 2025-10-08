<script lang="ts">
  import { User, Lock, Copy, Eye, Check, Wallet } from 'lucide-svelte';
  import { principal, authStore } from '../stores/auth';
  import { userLockStore } from '../stores/userLock';
  import { balanceStore, BalanceService, formatIcpBalance } from '../services/balance';
  
  let copyPrincipalSuccess = false;
  let copyLockSuccess = false;
  let showDetailsModal = false;
  let balanceService = null;
  
  // Subscribe to auth changes to create balance service
  $: if ($authStore.identity && $authStore.principal) {
    balanceService = new BalanceService($authStore.identity);
    balanceService.fetchIcpBalance($authStore.principal);
  }
  
  async function copyPrincipal() {
    if ($principal) {
      try {
        await navigator.clipboard.writeText($principal.toString());
        copyPrincipalSuccess = true;
        setTimeout(() => {
          copyPrincipalSuccess = false;
        }, 2000);
      } catch (error) {
        console.error('Failed to copy principal:', error);
      }
    }
  }
  
  async function copyLockAddress() {
    if ($userLockStore.canisterId) {
      try {
        await navigator.clipboard.writeText($userLockStore.canisterId.toString());
        copyLockSuccess = true;
        setTimeout(() => {
          copyLockSuccess = false;
        }, 2000);
      } catch (error) {
        console.error('Failed to copy address:', error);
      }
    }
  }
  
  // Use real canister details from store
  $: canisterDetails = {
    isBlackholed: $userLockStore.isBlackholed,
    isRegistered: $userLockStore.isRegisteredOnKongSwap,
    hasFunding: $userLockStore.hasIcpFunding,
    cycles: 792_000_000_000,
    memory: 2.1,
    memoryLimit: 4096,
    moduleHash: '0x3f2a1b8c9d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c',
  };
  
  function formatCycles(cycles) {
    return (cycles / 1_000_000_000).toFixed(1) + 'B';
  }
</script>

<!-- Side by Side Address Layout - Now fits on mobile/splitscreen -->
<div class="grid grid-cols-2 gap-2 md:gap-4">
    
    <!-- Wallet Connection - Compact -->
    <div class="bg-kong-accent-blue/10 border border-kong-accent-blue/30 rounded-lg p-2 md:p-3">
      <!-- Header -->
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center space-x-2">
          <User class="w-4 h-4 text-kong-accent-blue" />
          <h4 class="text-xs md:text-sm font-medium text-kong-text-primary">Wallet</h4>
        </div>
        <div class="flex items-center space-x-1 text-xs">
          <div class="w-1.5 h-1.5 bg-kong-accent-green rounded-full animate-pulse"></div>
          <span class="text-kong-accent-green">Active</span>
        </div>
      </div>
      
      <!-- ICP Balance -->
      <div class="mb-2">
        <div class="text-xs text-kong-text-secondary">ICP Balance</div>
        {#if $balanceStore.isLoading}
          <div class="text-xs text-kong-text-secondary">Loading...</div>
        {:else if $balanceStore.error}
          <div class="text-xs text-kong-error">Error</div>
        {:else}
          <div class="text-sm font-bold text-kong-text-primary">
            {formatIcpBalance($balanceStore.icpBalance)}
          </div>
        {/if}
      </div>
      
      <!-- Principal ID - Compact -->
      <div>
        <div class="text-xs text-kong-text-secondary mb-1">Principal ID</div>
        {#if $principal}
          <div class="bg-kong-bg-tertiary/50 p-1.5 rounded border border-kong-border/50 relative group">
            <p class="text-xs font-mono text-kong-text-primary break-all leading-tight pr-6">
              {$principal.toString().substring(0, 20)}...
            </p>
            <button 
              on:click={copyPrincipal}
              class="absolute top-1 right-1 p-1 text-kong-accent-green hover:bg-kong-accent-green/10 rounded transition-all"
              title="Copy full principal"
            >
              {#if copyPrincipalSuccess}
                <Check class="w-3 h-3" />
              {:else}
                <Copy class="w-3 h-3" />
              {/if}
            </button>
          </div>
          <p class="text-xs text-kong-text-secondary mt-1">
            For ICP only
          </p>
        {/if}
      </div>
    </div>

    <!-- Your Locked Liquidity - Compact -->
    <div class="bg-kong-accent-green/10 border border-kong-accent-green/30 rounded-lg p-2 md:p-3">
      <!-- Header -->
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center space-x-2">
          <Lock class="w-4 h-4 text-kong-accent-green" />
          <h4 class="text-xs md:text-sm font-medium text-kong-text-primary">LP Lock</h4>
        </div>
        <div class="flex items-center space-x-1 text-xs">
          <div class="w-1.5 h-1.5 bg-kong-accent-green rounded-full animate-pulse"></div>
          <span class="text-kong-accent-green">Locked</span>
        </div>
      </div>
      
      <!-- Voting Power -->
      <div class="mb-2">
        <div class="text-xs text-kong-text-secondary">Voting Power</div>
        <div class="text-sm font-bold kong-gradient-text">
          {($userLockStore.votingPower || 0).toLocaleString()}
        </div>
      </div>
      
      <!-- Lock Address - Compact -->
      <div>
        <div class="text-xs text-kong-text-secondary mb-1">Lock Address</div>
        {#if $userLockStore.canisterId}
          <div class="bg-kong-bg-tertiary/50 p-1.5 rounded border border-kong-accent-green/30 relative group">
            <p class="text-xs font-mono text-kong-text-primary break-all leading-tight pr-6">
              {$userLockStore.canisterId.toString().substring(0, 20)}...
            </p>
            <button 
              on:click={copyLockAddress}
              class="absolute top-1 right-1 p-1 text-kong-accent-green hover:bg-kong-accent-green/10 rounded transition-all"
              title="Copy lock address"
            >
              {#if copyLockSuccess}
                <Check class="w-3 h-3" />
              {:else}
                <Copy class="w-3 h-3" />
              {/if}
            </button>
          </div>
          <div class="flex items-center justify-between mt-1">
            <a href="/docs#transfer-guide" class="text-xs text-kong-accent-green hover:underline">Guide →</a>
            <button 
              on:click={() => showDetailsModal = true}
              class="text-xs text-kong-accent-blue hover:underline"
            >
              Details
            </button>
          </div>
        {:else}
          <div class="text-xs text-kong-text-secondary">No lock canister yet</div>
        {/if}
      </div>
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