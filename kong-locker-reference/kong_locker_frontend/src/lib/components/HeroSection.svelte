<script lang="ts">
  import { Lock, Loader } from 'lucide-svelte';
  import { authService, isAuthenticated, isAuthLoading, authError } from '../stores/auth';
  
  export let showGlow = false;
  
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
</script>

<!-- Hero Section -->
<section class="text-center space-y-8 py-12">
  <!-- Logo/Brand -->
  <div class="flex items-center justify-center space-x-3 mb-8">
    <div class="p-3 bg-gradient-button rounded-full {showGlow ? 'animate-glow' : ''}">
      <Lock class="w-8 h-8 text-white" />
    </div>
    <h1 class="text-4xl md:text-6xl font-bold font-exo">
      <span class="kong-gradient-text">Kong</span>
      <span class="text-kong-text-primary">Locker</span>
    </h1>
  </div>
  
  <!-- Tagline -->
  <div class="max-w-4xl mx-auto">
    <h2 class="text-xl md:text-2xl text-kong-text-secondary font-space">
      Transform KongSwap LP tokens into permanent voting power
    </h2>
  </div>
  
  <!-- CTA Button -->
  <div class="flex justify-center items-center pt-6">
    <button 
      class="kong-button-primary px-8 py-4 text-lg font-semibold kong-shine disabled:opacity-50 disabled:cursor-not-allowed"
      on:click={handleConnect}
      disabled={$isAuthLoading}
    >
      {#if $isAuthLoading}
        <div class="flex items-center space-x-2">
          <Loader class="w-5 h-5 animate-spin" />
          <span>Connecting...</span>
        </div>
      {:else if $isAuthenticated}
        <span>Connected ✨</span>
      {:else}
        <span>Connect Wallet to Start</span>
      {/if}
    </button>
  </div>
  
  <!-- Auth Error Display -->
  {#if $authError}
    <div class="mt-4 p-3 bg-kong-error/10 border border-kong-error/20 rounded-lg max-w-md mx-auto">
      <p class="text-sm text-kong-error text-center">{$authError}</p>
      <button 
        on:click={authService.clearError}
        class="text-xs text-kong-error/70 hover:text-kong-error mt-1 w-full"
      >
        Dismiss
      </button>
    </div>
  {/if}
</section>