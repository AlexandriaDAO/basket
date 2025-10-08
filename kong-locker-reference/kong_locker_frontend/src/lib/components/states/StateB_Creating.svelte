<script lang="ts">
  import { userLockStore } from '../../stores/userLock';
  
  export let integrated = false;
</script>

<!-- Creating State (State B) - show progress -->
<div class="{integrated ? 'border-t border-kong-border/30 pt-6' : 'max-w-lg mx-auto'}">
  <div class="{integrated ? 'space-y-4' : 'kong-panel space-y-4'}">
    <h3 class="{integrated ? 'text-lg' : 'text-xl'} font-semibold text-kong-text-primary text-center">Creating Your Lock Canister...</h3>
    
    <div class="space-y-3">
      {#each [
        { step: 'Payment received', key: 'payment' },
        { step: 'Canister created', key: 'created' }, 
        { step: 'Code installed', key: 'installed' },
        { step: 'Address saved', key: 'saved' },
        { step: 'Funding with ICP...', key: 'Funding' },
        { step: 'Registering with KongSwap...', key: 'Registering' },
        { step: 'Blackholing canister...', key: 'Blackholing' }
      ] as {step, key}, i}
        <div class="flex items-center space-x-3">
          {#if i < 4}
            <!-- First 4 steps are always completed quickly -->
            <div class="w-5 h-5 bg-kong-accent-green rounded-full flex items-center justify-center">
              <div class="w-2 h-2 bg-white rounded-full"></div>
            </div>
            <span class="text-sm text-kong-text-primary">✅ {step}</span>
          {:else if $userLockStore.creationStep.includes(key)}
            <!-- Currently active step -->
            <div class="w-5 h-5 border-2 border-kong-accent-blue/30 border-t-kong-accent-blue rounded-full animate-spin"></div>
            <span class="text-sm text-kong-accent-blue">⏳ {step}</span>
          {:else if $userLockStore.creationStep === '' && i >= 4}
            <!-- Not yet reached -->
            <div class="w-5 h-5 border-2 border-kong-border/30 rounded-full"></div>
            <span class="text-sm text-kong-text-secondary">{step}</span>
          {:else}
            <!-- Completed step -->
            <div class="w-5 h-5 bg-kong-accent-green rounded-full flex items-center justify-center">
              <div class="w-2 h-2 bg-white rounded-full"></div>
            </div>
            <span class="text-sm text-kong-text-primary">✅ {step}</span>
          {/if}
        </div>
      {/each}
    </div>
    
    <div class="text-center pt-4 border-t border-kong-border/50">
      <p class="text-sm text-kong-text-secondary">
        If any step shows ⚠️, you can complete setup later
      </p>
    </div>

    <!-- Progress bar -->
    <div class="pt-3">
      <div class="w-full bg-kong-border/30 rounded-full h-2">
        <div 
          class="bg-kong-accent-green h-2 rounded-full transition-all duration-500 ease-out"
          style="width: {$userLockStore.isCreating ? '85%' : '100%'}"
        ></div>
      </div>
      <p class="text-xs text-kong-text-secondary text-center mt-2">
        {$userLockStore.isCreating ? 'Almost done...' : 'Creation complete!'}
      </p>
    </div>
  </div>
</div>