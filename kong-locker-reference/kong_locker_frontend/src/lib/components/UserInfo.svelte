<script lang="ts">
  import { onMount } from 'svelte';
  import { User, LogOut, Wallet, Copy, Check } from 'lucide-svelte';
  import { authStore, authService, isAuthenticated, principal } from '../stores/auth';
  import { balanceStore, BalanceService, formatIcpBalance } from '../services/balance';
  
  let balanceService: BalanceService | null = null;
  let copySuccess = false;
  
  // Subscribe to auth changes to create balance service
  $: if ($authStore.identity && $authStore.principal) {
    balanceService = new BalanceService($authStore.identity);
    balanceService.fetchIcpBalance($authStore.principal);
  }
  
  async function handleLogout() {
    try {
      await authService.logout();
      balanceService = null;
    } catch (error) {
      console.error('Logout failed:', error);
    }
  }
  
  async function copyPrincipal() {
    if ($principal) {
      try {
        await navigator.clipboard.writeText($principal.toString());
        copySuccess = true;
        setTimeout(() => {
          copySuccess = false;
        }, 2000);
      } catch (error) {
        console.error('Failed to copy principal:', error);
      }
    }
  }
  
  function truncatePrincipal(principalStr: string): string {
    if (principalStr.length <= 20) return principalStr;
    return `${principalStr.substring(0, 10)}...${principalStr.substring(principalStr.length - 10)}`;
  }
</script>

{#if $isAuthenticated && $principal}
  <div class="space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h3 class="text-lg font-semibold text-kong-text-primary">Wallet Connection</h3>
      <button 
        on:click={handleLogout}
        class="p-2 text-kong-text-secondary hover:text-kong-error hover:bg-kong-error/10 rounded-lg transition-all duration-200"
        title="Logout"
      >
        <LogOut class="w-4 h-4" />
      </button>
    </div>
    
    <!-- Connection Status -->
    <div class="flex items-center space-x-3">
      <div class="p-2 bg-kong-accent-green/20 rounded-lg">
        <User class="w-5 h-5 text-kong-accent-green" />
      </div>
      <div>
        <div class="text-base font-medium text-kong-text-primary">Connected</div>
        <div class="flex items-center space-x-1">
          <div class="w-2 h-2 bg-kong-accent-green rounded-full animate-pulse"></div>
          <span class="text-sm text-kong-accent-green">Active</span>
        </div>
      </div>
    </div>
    
    <!-- Principal ID -->
    <div class="space-y-2">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <span class="text-sm font-medium text-kong-text-primary">Principal ID</span>
          <div class="bg-kong-accent-blue/20 px-2 py-1 rounded-full">
            <span class="text-xs font-semibold text-kong-accent-blue">WALLET ADDRESS</span>
          </div>
        </div>
        <button 
          on:click={copyPrincipal}
          class="flex items-center space-x-1 px-2 py-1 text-sm text-kong-accent-green hover:bg-kong-accent-green/10 rounded-md transition-all duration-200"
          title="Copy Principal"
        >
          {#if copySuccess}
            <Check class="w-4 h-4" />
            <span>Copied!</span>
          {:else}
            <Copy class="w-4 h-4" />
            <span>Copy</span>
          {/if}
        </button>
      </div>
      <div class="bg-kong-bg-tertiary/50 p-3 rounded-lg border border-kong-border/50">
        <p class="text-sm font-mono text-kong-text-primary mb-2">
          {truncatePrincipal($principal.toString())}
        </p>
        <p class="text-xs text-kong-text-secondary">
          <strong>For ICP only</strong> • Send LP tokens to your Lock Address instead
        </p>
      </div>
    </div>
    
    <!-- ICP Balance -->
    <div class="space-y-3 pt-4 border-t border-kong-border/50">
      <div class="flex items-center space-x-3">
        <div class="p-2 bg-kong-accent-blue/20 rounded-lg">
          <Wallet class="w-5 h-5 text-kong-accent-blue" />
        </div>
        <div class="flex-1">
          <div class="text-sm font-medium text-kong-text-primary">ICP Balance</div>
          {#if $balanceStore.isLoading}
            <div class="flex items-center space-x-2">
              <div class="w-4 h-4 border-2 border-kong-accent-green/30 border-t-kong-accent-green rounded-full animate-spin"></div>
              <span class="text-sm text-kong-text-secondary">Loading...</span>
            </div>
          {:else if $balanceStore.error}
            <div class="text-sm text-kong-error">Failed to load</div>
            <button 
              on:click={() => balanceService?.fetchIcpBalance($principal)}
              class="text-sm text-kong-accent-green hover:text-kong-success-hover transition-colors"
            >
              Retry
            </button>
          {:else}
            <div class="text-lg font-bold text-kong-text-primary">
              {formatIcpBalance($balanceStore.icpBalance)}
            </div>
            {#if $balanceStore.lastUpdated}
              <div class="text-xs text-kong-text-secondary">
                Updated: {$balanceStore.lastUpdated.toLocaleTimeString()}
              </div>
            {/if}
          {/if}
        </div>
      </div>
      
      <!-- Refresh Button -->
      <button 
        on:click={() => $principal && balanceService?.fetchIcpBalance($principal)}
        class="w-full px-4 py-2 text-sm font-medium text-kong-accent-green hover:bg-kong-accent-green/10 border border-kong-accent-green/30 hover:border-kong-accent-green/50 rounded-lg transition-all duration-200"
        disabled={$balanceStore.isLoading}
      >
        {$balanceStore.isLoading ? 'Refreshing...' : 'Refresh Balance'}
      </button>
    </div>
  </div>
{/if}