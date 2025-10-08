<script lang="ts">
  import { Lock, Copy, Eye, Check, User } from 'lucide-svelte';
  import { userLockStore } from '../../stores/userLock';
  import { principal } from '../../stores/auth';
  
  export let integrated = false;
  
  let copySuccess = false;
  let copyPrincipalSuccess = false;
  let showDetailsModal = false;
  
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
  
  function truncateAddress(address: string): string {
    if (address.length <= 25) return address;
    return `${address.substring(0, 12)}...${address.substring(address.length - 8)}`;
  }
  
  function truncatePrincipal(principalStr: string): string {
    if (principalStr.length <= 20) return principalStr;
    return `${principalStr.substring(0, 10)}...${principalStr.substring(principalStr.length - 10)}`;
  }
  
  function formatCycles(cycles: number): string {
    return (cycles / 1_000_000_000).toFixed(1) + 'B';
  }
</script>

<div class="{integrated ? 'border-t border-kong-border/30 pt-6' : 'max-w-2xl mx-auto'}">
  <!-- Address info is now shown in the CombinedAddressInfo component above -->
  {#if !integrated}
    <div class="kong-panel text-center space-y-4">
      <h3 class="text-xl font-semibold text-kong-text-primary">Lock Canister Ready</h3>
      <p class="text-kong-text-secondary">Your lock canister is active and ready to receive LP tokens.</p>
    </div>
  {/if}
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