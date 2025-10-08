<script lang="ts">
  import { CheckCircle, XCircle, AlertCircle, Loader, RefreshCw } from 'lucide-svelte';
  import { userLockStore, userLockService } from '../../stores/userLock';
  import { lpLockingService } from '../../services/lpLocking';
  import { get } from 'svelte/store';
  import { authStore } from '../../stores/auth';
  
  export let integrated = false;
  let isRegistering = false;
  let registrationError = '';
  
  $: canisterStatus = {
    exists: !!$userLockStore.canisterId,
    blackholed: $userLockStore.isBlackholed,
    funded: $userLockStore.hasIcpFunding,
    registered: $userLockStore.isRegisteredOnKongSwap,
  };
  
  // Determine what actions are needed
  $: needsRegistration = !canisterStatus.registered;
  $: needsFunding = !canisterStatus.registered && !canisterStatus.funded;
  $: needsBlackholing = !canisterStatus.blackholed;
  
  async function handleCompleteSetup() {
    await userLockService.completeSetup();
  }
  
  async function handleRetryRegistration() {
    if (!$userLockStore.canisterId) return;
    
    isRegistering = true;
    registrationError = '';
    
    try {
      const result = await lpLockingService.registerLockCanister($userLockStore.canisterId);
      console.log('Registration result:', result);
      
      // Reload user state to get updated status
      const authState = get(authStore);
      if (authState.principal) {
        await userLockService.loadUserState(authState.principal);
      }
      
      isRegistering = false;
    } catch (error) {
      console.error('Registration failed:', error);
      registrationError = error instanceof Error ? error.message : 'Registration failed';
      isRegistering = false;
    }
  }
</script>

<div class="{integrated ? 'border-t border-kong-border/30 pt-6' : 'max-w-lg mx-auto'}">
  <div class="{integrated ? 'bg-kong-accent-orange/10 border border-kong-accent-orange/20 rounded-lg p-4 space-y-6' : 'kong-panel bg-kong-accent-orange/10 border-kong-accent-orange/20 space-y-6'}">
    <div class="space-y-2">
      <h3 class="{integrated ? 'text-lg' : 'text-xl'} font-bold text-kong-text-primary flex items-center space-x-2">
        <AlertCircle class="w-6 h-6 text-kong-accent-orange" />
        <span>Setup Incomplete</span>
      </h3>
      <p class="text-sm text-kong-text-secondary">
        Your lock canister needs to complete setup before it can receive LP tokens.
      </p>
    </div>

    <!-- Status Checklist -->
    <div class="space-y-3">
      <h3 class="text-sm font-semibold text-kong-text-primary">Canister Status:</h3>
      
      <div class="space-y-2">
        <div class="flex items-center space-x-2">
          {#if canisterStatus.exists}
            <CheckCircle class="w-5 h-5 text-kong-accent-green" />
            <span class="text-sm text-kong-text-secondary">Canister created</span>
          {:else}
            <XCircle class="w-5 h-5 text-kong-error" />
            <span class="text-sm text-kong-text-secondary">Canister not found</span>
          {/if}
        </div>
        
        <div class="flex items-center space-x-2">
          {#if canisterStatus.blackholed}
            <CheckCircle class="w-5 h-5 text-kong-accent-green" />
            <span class="text-sm text-kong-text-secondary">Canister blackholed (immutable)</span>
          {:else}
            <XCircle class="w-5 h-5 text-kong-error" />
            <span class="text-sm text-kong-text-secondary">Canister not blackholed</span>
          {/if}
        </div>
        
        <div class="flex items-center space-x-2">
          {#if canisterStatus.registered}
            <CheckCircle class="w-5 h-5 text-kong-accent-green" />
            <span class="text-sm text-kong-text-secondary">Has ALEX tokens (registered)</span>
          {:else}
            <XCircle class="w-5 h-5 text-kong-error" />
            <span class="text-sm text-kong-text-secondary">No ALEX tokens (not registered)</span>
          {/if}
        </div>
        
        {#if !canisterStatus.registered}
          <div class="flex items-center space-x-2">
            {#if canisterStatus.funded}
              <CheckCircle class="w-5 h-5 text-kong-accent-green" />
              <span class="text-sm text-kong-text-secondary">Has 0.99+ ICP for registration</span>
            {:else}
              <XCircle class="w-5 h-5 text-kong-error" />
              <span class="text-sm text-kong-text-secondary">Needs 0.99 ICP for registration</span>
            {/if}
          </div>
        {/if}
      </div>
    </div>

    <!-- Canister ID Display -->
    {#if $userLockStore.canisterId}
      <div class="p-3 bg-kong-bg-secondary rounded-lg">
        <p class="text-xs text-kong-text-secondary mb-1">Your Lock Canister:</p>
        <p class="text-sm font-mono text-kong-text-primary break-all">
          {$userLockStore.canisterId.toText()}
        </p>
      </div>
    {/if}

    <!-- Action Buttons based on what's needed -->
    <div class="space-y-3">
      {#if needsRegistration && !needsFunding}
        <!-- Registration retry option -->
        <div class="space-y-2">
          <p class="text-sm text-kong-text-secondary">
            Your canister has ICP but hasn't swapped for ALEX yet.
          </p>
          <button 
            on:click={handleRetryRegistration}
            disabled={isRegistering}
            class="w-full kong-button-primary py-3 text-sm font-semibold disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center space-x-2"
          >
            {#if isRegistering}
              <Loader class="w-4 h-4 animate-spin" />
              <span>Registering...</span>
            {:else}
              <RefreshCw class="w-4 h-4" />
              <span>Complete Registration</span>
            {/if}
          </button>
          {#if registrationError}
            <p class="text-xs text-kong-error">{registrationError}</p>
          {/if}
        </div>
      {/if}
      
      {#if needsBlackholing || needsFunding}
        <!-- Complete setup button for cases needing factory intervention -->
        <button 
          on:click={handleCompleteSetup}
          disabled={$userLockStore.isLoading}
          class="w-full kong-button-primary py-3 text-sm font-semibold disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if $userLockStore.isLoading}
            <div class="flex items-center justify-center space-x-2">
              <Loader class="w-4 h-4 animate-spin" />
              <span>Completing Setup...</span>
            </div>
          {:else}
            Complete Setup
          {/if}
        </button>
      {/if}
    </div>
    
    <p class="text-xs text-kong-text-secondary text-center">
      {#if needsFunding}
        This will fund your canister with ICP and complete registration.
      {:else if needsRegistration}
        This will swap ICP for ALEX to complete registration.
      {:else if needsBlackholing}
        This will blackhole your canister to make it immutable.
      {:else}
        This will complete any remaining setup steps.
      {/if}
    </p>
  </div>
</div>