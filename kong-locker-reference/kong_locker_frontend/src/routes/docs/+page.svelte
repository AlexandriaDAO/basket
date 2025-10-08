<script lang="ts">
  import { onMount } from 'svelte';
  import { HttpAgent, Actor } from '@dfinity/agent';
  import { Principal } from '@dfinity/principal';
  import { Loader, Menu, X } from 'lucide-svelte';
  
  let activeSection = 'introduction';
  let copySuccess = '';
  let mobileMenuOpen = false;
  let expandedSections: { [key: string]: boolean } = {};
  let developerSectionExpanded = false;
  
  function toggleExpanded(key: string) {
    expandedSections[key] = !expandedSections[key];
    expandedSections = { ...expandedSections };
  }
  
  // API call states
  let apiResults: { [key: string]: any } = {};
  let apiLoading: { [key: string]: boolean } = {};
  let apiErrors: { [key: string]: string } = {};
  let principalInput = '';
  
  // LP Position Demo states
  let lockCanisterInput = '';
  let lpPositions: any[] = [];
  let lpPositionsLoading = false;
  let lpPositionsError = '';
  
  // LP Locking Actor
  let lpLockingActor: any = null;
  
  // KongSwap Actor for LP position queries
  let kongSwapActor: any = null;
  
  const userSections = [
    { id: 'introduction', title: 'Introduction', icon: '📚' },
    { id: 'transfer-guide', title: 'LP Token Transfer Guide', icon: '🚀' },
    { id: 'cross-site-verification', title: 'Cross-Site Identity & Integration', icon: '🔗' }
  ];
  
  const developerSections = [
    { id: 'overview', title: 'API Overview', icon: '🔍' },
    { id: 'query-functions', title: 'Query Functions', icon: '🔎' },
    { id: 'update-functions', title: 'Update Functions', icon: '✏️' },
    { id: 'kongswap-integration', title: 'KongSwap Direct Queries', icon: '🔗' },
    { id: 'canister-verification', title: 'Lock Canister Verification', icon: '🔒' },
    { id: 'lp-position-demo', title: 'LP Position Breakdown Demo', icon: '🎮' },
    { id: 'types', title: 'Data Types', icon: '📊' },
    { id: 'examples', title: 'Integration Examples', icon: '💡' },
    { id: 'errors', title: 'Error Handling', icon: '⚠️' }
  ];
  
  const sections = [...userSections, ...developerSections];
  
  // IDL for LP Locking canister (current interface)
  const lpLockingIdl = ({ IDL }: any) => {
    const DetailedCanisterStatus = IDL.Record({
      'canister_id': IDL.Principal,
      'is_blackholed': IDL.Bool,
      'controller_count': IDL.Nat32,
      'cycle_balance': IDL.Nat,
      'memory_size': IDL.Nat,
      'module_hash': IDL.Opt(IDL.Vec(IDL.Nat8)),
    });
    
    const AnalyticsOverview = IDL.Record({
      'total_lock_canisters': IDL.Nat64,
      'participants': IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Principal)),
      'last_updated': IDL.Nat64,
    });
    
    return IDL.Service({
      // Query calls
      'get_total_positions_count': IDL.Func([], [IDL.Nat64], ['query']),
      'get_my_lock_canister': IDL.Func([], [IDL.Opt(IDL.Principal)], ['query']),
      'get_all_lock_canisters': IDL.Func([], [IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Principal))], ['query']),
      'get_analytics_overview': IDL.Func([], [AnalyticsOverview], ['query']),
      
      // Update calls
      'get_detailed_canister_status': IDL.Func([], [IDL.Variant({ 'Ok': DetailedCanisterStatus, 'Err': IDL.Text })], []),
      'create_lock_canister': IDL.Func([], [IDL.Variant({ 'Ok': IDL.Principal, 'Err': IDL.Text })], []),
      'complete_my_canister_setup': IDL.Func([], [IDL.Variant({ 'Ok': IDL.Text, 'Err': IDL.Text })], []),
    });
  };
  
  // KongSwap IDL for LP position queries
  const kongSwapIdl = ({ IDL }: any) => {
    const LPReply = IDL.Record({
      'name': IDL.Text,
      'symbol': IDL.Text,
      'lp_token_id': IDL.Nat64,
      'balance': IDL.Float64,
      'usd_balance': IDL.Float64,
      'chain_0': IDL.Text,
      'symbol_0': IDL.Text,
      'address_0': IDL.Text,
      'amount_0': IDL.Float64,
      'usd_amount_0': IDL.Float64,
      'chain_1': IDL.Text,
      'symbol_1': IDL.Text,
      'address_1': IDL.Text,
      'amount_1': IDL.Float64,
      'usd_amount_1': IDL.Float64,
      'ts': IDL.Nat64,
    });
    
    const UserBalancesReply = IDL.Variant({ 'LP': LPReply });
    
    return IDL.Service({
      'user_balances': IDL.Func([IDL.Text], [IDL.Variant({
        'Ok': IDL.Vec(UserBalancesReply),
        'Err': IDL.Text
      })], ['query']),
    });
  };
  
  // Initialize the actors
  onMount(async () => {
    const host = 'https://icp0.io';
    const agent = new HttpAgent({ host });
    
    // Try to get authenticated identity, fallback to anonymous
    try {
      // Check if user is authenticated (this would be your auth logic)
      const isAuthenticated = window.ic?.plug?.isConnected() || window.ic?.infinityWallet?.isConnected();
      
      if (isAuthenticated) {
        console.log('Using authenticated calls for better results');
      } else {
        console.log('Using anonymous calls - some functions may return limited results');
      }
    } catch (error) {
      console.log('Using anonymous calls');
    }
    
    lpLockingActor = Actor.createActor(lpLockingIdl, {
      agent,
      canisterId: 'eazgb-giaaa-aaaap-qqc2q-cai',
    });
    
    kongSwapActor = Actor.createActor(kongSwapIdl, {
      agent,
      canisterId: '2ipq2-uqaaa-aaaar-qailq-cai',
    });
    
    // Intersection observer for active section highlighting
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            activeSection = entry.target.id;
          }
        });
      },
      { threshold: 0.3 }
    );
    
    sections.forEach(section => {
      const element = document.getElementById(section.id);
      if (element) observer.observe(element);
    });
    
    return () => observer.disconnect();
  });
  
  // API call functions
  async function callGetMyLockCanister() {
    const key = 'get_my_lock_canister';
    apiLoading[key] = true;
    apiErrors[key] = '';
    
    try {
      const result = await lpLockingActor.get_my_lock_canister();
      apiResults[key] = result.length > 0 ? result[0].toText() : 'No lock canister found';
    } catch (error: any) {
      apiErrors[key] = error.message || 'Failed to call API';
    } finally {
      apiLoading[key] = false;
    }
  }
  
  async function callGetAllLockCanisters() {
    const key = 'get_all_lock_canisters';
    apiLoading[key] = true;
    apiErrors[key] = '';
    
    try {
      const result = await lpLockingActor.get_all_lock_canisters();
      apiResults[key] = result.map((pair: any) => ({
        user: pair[0].toText(),
        canister: pair[1].toText()
      }));
    } catch (error: any) {
      apiErrors[key] = error.message || 'Failed to call API';
    } finally {
      apiLoading[key] = false;
    }
  }
  
  async function callGetTotalPositionsCount() {
    const key = 'get_total_positions_count';
    apiLoading[key] = true;
    apiErrors[key] = '';
    
    try {
      const result = await lpLockingActor.get_total_positions_count();
      apiResults[key] = result.toString();
    } catch (error: any) {
      apiErrors[key] = error.message || 'Failed to call API';
    } finally {
      apiLoading[key] = false;
    }
  }
  
  async function callGetAnalyticsOverview() {
    const key = 'get_analytics_overview';
    apiLoading[key] = true;
    apiErrors[key] = '';
    
    try {
      const result = await lpLockingActor.get_analytics_overview();
      apiResults[key] = {
        total_lock_canisters: result.total_lock_canisters.toString(),
        participant_count: result.participants.length,
        participants: result.participants.slice(0, 5).map((pair: any) => ({
          user: pair[0].toText(),
          canister: pair[1].toText()
        })),
        showing_first: Math.min(5, result.participants.length),
        last_updated: new Date(Number(result.last_updated) / 1_000_000).toLocaleString()
      };
    } catch (error: any) {
      apiErrors[key] = error.message || 'Failed to call API';
    } finally {
      apiLoading[key] = false;
    }
  }

  async function callGetDetailedCanisterStatus() {
    const key = 'get_detailed_canister_status';
    apiLoading[key] = true;
    apiErrors[key] = '';
    
    try {
      const result = await lpLockingActor.get_detailed_canister_status();
      
      if ('Ok' in result) {
        const status = result.Ok;
        apiResults[key] = {
          canister_id: status.canister_id.toText(),
          is_blackholed: status.is_blackholed,
          controller_count: status.controller_count.toString(),
          cycle_balance: status.cycle_balance.toString(),
          memory_size: `${(Number(status.memory_size) / (1024 * 1024)).toFixed(2)} MB`,
          module_hash: status.module_hash.length > 0 ? 'Present' : 'None'
        };
      } else {
        apiResults[key] = { error: result.Err };
      }
    } catch (error: any) {
      apiErrors[key] = error.message || 'Failed to call API';
    } finally {
      apiLoading[key] = false;
    }
  }
  
  // LP Position Demo functions
  async function getLPPositions() {
    if (!lockCanisterInput.trim()) {
      lpPositionsError = 'Please enter a lock canister Principal ID';
      return;
    }
    
    lpPositionsLoading = true;
    lpPositionsError = '';
    lpPositions = [];
    
    try {
      const result = await kongSwapActor.user_balances(lockCanisterInput.trim());
      
      if ('Ok' in result) {
        lpPositions = result.Ok.map((reply: any) => reply.LP);
        if (lpPositions.length === 0) {
          lpPositionsError = 'No LP positions found for this canister. It may not be registered on KongSwap yet.';
        }
      } else if (result.Err.includes("User not found")) {
        lpPositionsError = 'Lock canister not found on KongSwap. It may not be registered yet or the Principal ID is incorrect.';
      } else {
        lpPositionsError = result.Err;
      }
    } catch (error: any) {
      lpPositionsError = error.message || 'Failed to fetch LP positions';
    } finally {
      lpPositionsLoading = false;
    }
  }
  
  function formatTokenAmount(amount: number, symbol: string): string {
    if (amount >= 1000000) {
      return `${(amount / 1000000).toFixed(2)}M ${symbol}`;
    } else if (amount >= 1000) {
      return `${(amount / 1000).toFixed(2)}K ${symbol}`;
    } else if (amount >= 1) {
      return `${amount.toFixed(2)} ${symbol}`;
    } else {
      return `${amount.toFixed(6)} ${symbol}`;
    }
  }
  
  function calculateTotalUSDValue(positions: any[]): number {
    return positions.reduce((sum, pos) => sum + pos.usd_balance, 0);
  }
  
  function calculateVotingPower(positions: any[]): number {
    const totalUSD = calculateTotalUSDValue(positions);
    return Math.floor(totalUSD * 100); // Convert to cents
  }
  
  function scrollToSection(sectionId: string) {
    activeSection = sectionId;
    mobileMenuOpen = false; // Close mobile menu when navigating
    const element = document.getElementById(sectionId);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }
  
  async function copyCode(text: string, buttonId: string) {
    try {
      await navigator.clipboard.writeText(text);
      copySuccess = buttonId;
      setTimeout(() => copySuccess = '', 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }
</script>

<!-- Mobile Navigation Toggle -->
<div class="lg:hidden fixed top-4 left-4 z-50">
  <button
    on:click={() => mobileMenuOpen = !mobileMenuOpen}
    class="bg-kong-bg-secondary border border-kong-border rounded-lg p-3 text-white hover:bg-kong-bg-tertiary transition-colors"
  >
    {#if mobileMenuOpen}
      <X size={24} />
    {:else}
      <Menu size={24} />
    {/if}
  </button>
</div>

<!-- Mobile Navigation Overlay -->
{#if mobileMenuOpen}
  <div class="lg:hidden fixed inset-0 z-40 bg-black/80" role="button" tabindex="0" on:click={() => mobileMenuOpen = false} on:keydown={(e) => e.key === 'Escape' && (mobileMenuOpen = false)}></div>
  <aside class="lg:hidden fixed top-0 left-0 w-80 h-full bg-kong-bg-secondary border-r border-kong-border z-40 transform transition-transform duration-300">
    <div class="p-6 pt-20">
      <h2 class="text-xl font-bold text-white mb-6">Documentation</h2>
      <nav class="space-y-4">
        <!-- User Guide Section -->
        <div>
          <h3 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-2">User Guide</h3>
          <div class="space-y-2">
            {#each userSections as section}
              <button
                on:click={() => scrollToSection(section.id)}
                class="w-full text-left px-4 py-3 rounded-lg transition-all duration-200 flex items-center gap-3
                       {activeSection === section.id 
                         ? 'bg-blue-600/20 text-blue-400 border-l-2 border-blue-400' 
                         : 'text-gray-400 hover:text-white hover:bg-gray-800/50'}"
              >
                <span class="text-lg">{section.icon}</span>
                <span class="text-sm">{section.title}</span>
              </button>
            {/each}
          </div>
        </div>
        
        <!-- Developer Integration Section -->
        <div>
          <h3 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-2">Developer Integration</h3>
          <div class="space-y-2">
            {#each developerSections as section}
              <button
                on:click={() => scrollToSection(section.id)}
                class="w-full text-left px-4 py-3 rounded-lg transition-all duration-200 flex items-center gap-3
                       {activeSection === section.id 
                         ? 'bg-blue-600/20 text-blue-400 border-l-2 border-blue-400' 
                         : 'text-gray-400 hover:text-white hover:bg-gray-800/50'}"
              >
                <span class="text-lg">{section.icon}</span>
                <span class="text-sm">{section.title}</span>
              </button>
            {/each}
          </div>
        </div>
      </nav>
    </div>
  </aside>
{/if}

<div class="flex flex-col lg:flex-row gap-4 lg:gap-8">
  <!-- Desktop Sidebar Navigation -->
  <aside class="hidden lg:block w-64 sticky top-8 h-fit">
    <div class="bg-kong-bg-secondary rounded-2xl p-6 border border-kong-border">
      <h2 class="text-xl font-bold text-white mb-6">Documentation</h2>
      <nav class="space-y-4">
        <!-- User Guide Section -->
        <div>
          <h3 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-2">User Guide</h3>
          <div class="space-y-2">
            {#each userSections as section}
              <button
                on:click={() => scrollToSection(section.id)}
                class="w-full text-left px-4 py-2 rounded-lg transition-all duration-200 flex items-center gap-2
                       {activeSection === section.id 
                         ? 'bg-blue-600/20 text-blue-400 border-l-2 border-blue-400' 
                         : 'text-gray-400 hover:text-white hover:bg-gray-800/50'}"
              >
                <span>{section.icon}</span>
                <span class="text-sm">{section.title}</span>
              </button>
            {/each}
          </div>
        </div>
        
        <!-- Developer Integration Section -->
        <div>
          <h3 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-2">Developer Integration</h3>
          <div class="space-y-2">
            {#each developerSections as section}
              <button
                on:click={() => scrollToSection(section.id)}
                class="w-full text-left px-4 py-2 rounded-lg transition-all duration-200 flex items-center gap-2
                       {activeSection === section.id 
                         ? 'bg-blue-600/20 text-blue-400 border-l-2 border-blue-400' 
                         : 'text-gray-400 hover:text-white hover:bg-gray-800/50'}"
              >
                <span>{section.icon}</span>
                <span class="text-sm">{section.title}</span>
              </button>
            {/each}
          </div>
        </div>
      </nav>
    </div>
  </aside>

  <!-- Main Documentation Content -->
  <main class="flex-1 max-w-full lg:max-w-4xl">
    <!-- Back Button -->
    <div class="mb-4">
      <a 
        href="/" 
        class="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
        </svg>
        <span class="text-sm">Back to Kong Locker</span>
      </a>
    </div>
    
    <!-- Header -->
    <div class="bg-gradient-to-r from-blue-600 to-purple-600 rounded-2xl p-4 sm:p-6 lg:p-8 mb-6 lg:mb-8">
      <h1 class="text-2xl sm:text-3xl lg:text-4xl font-bold text-white mb-2">Kong Locker API Documentation</h1>
      <div class="mt-4 flex flex-col sm:flex-row gap-2 sm:gap-4">
        <span class="bg-white/20 px-3 py-1 rounded-full text-xs sm:text-sm text-white">
          Canister ID: eazgb-giaaa-aaaap-qqc2q-cai
        </span>
      </div>
      
      <!-- Authentication Notice -->
      <div class="mt-4 bg-yellow-900/30 border border-yellow-800/50 rounded-lg p-3">
        <div class="flex items-start gap-2">
          <span class="text-yellow-400 text-sm">⚠️</span>
          <div class="text-yellow-200 text-sm">
            <strong>Authentication Notice:</strong> This documentation uses anonymous calls, so some functions may return limited results. 
            For full functionality (like <code class="bg-black/30 px-1 rounded">get_detailed_canister_status</code>), 
            connect your wallet in the main app.
          </div>
        </div>
      </div>
    </div>

    <!-- Introduction Section -->
    <section id="introduction" class="mb-12 bg-kong-bg-secondary rounded-2xl p-8 border border-kong-border">
      <h2 class="text-3xl font-bold text-white mb-6">📚 Trustless & Secure</h2>
      <div class="prose prose-invert max-w-none">
        <p class="text-gray-300 leading-relaxed mb-6">
          Kong Locker provides permanent liquidity locking through <strong>blackholed canisters</strong> - making it mathematically impossible for anyone, including developers, to access locked tokens.
        </p>
        
        <div class="bg-green-900/20 border border-green-800 rounded-lg p-6 mb-6">
          <h3 class="text-lg font-semibold text-green-400 mb-4 flex items-center gap-2">
            🔒 Zero Trust Required
          </h3>
          <div class="space-y-3 text-gray-300">
            <p><strong>Blackholed Canisters:</strong> Each user gets their own canister that is immediately "blackholed" (all controllers removed), making it completely autonomous and unchangeable.</p>
            <p><strong>No Backend Dependencies:</strong> Once created, your lock canister operates independently. Even if Kong Locker service disappears, your tokens remain safely locked and queryable.</p>
            <p><strong>Immutable Code:</strong> Lock canisters cannot be upgraded, modified, or controlled by anyone - not even the original developers.</p>
            <p><strong>Transparent Operations:</strong> All code is open-source and verifiable on the Internet Computer blockchain.</p>
          </div>
        </div>
        
        <div class="bg-blue-900/20 border border-blue-800 rounded-lg p-4">
          <h3 class="text-lg font-semibold text-blue-400 mb-2">How It Works</h3>
          <ol class="list-decimal list-inside text-gray-300 space-y-1 text-sm">
            <li>Pay 2 ICP to create your personal lock canister (1 ICP buys ALEX in the canister which is burned forever, 1 ICP goes to ALEX stakers to offset canister creation costs)</li>
            <li>Canister is funded, registered on KongSwap, then blackholed</li>
            <li>Send LP tokens to your canister - they're locked forever</li>
            <li>Query your positions directly from KongSwap using your canister ID</li>
          </ol>
        </div>
      </div>
    </section>

    <!-- LP Token Transfer Guide Section -->
    <section id="transfer-guide" class="mb-12 bg-kong-bg-secondary rounded-2xl p-8 border border-kong-border">
      <h2 class="text-3xl font-bold text-white mb-6">🚀 How to Transfer LP Tokens</h2>
      
      <!-- Address Guide -->
      <div class="bg-kong-accent-blue/10 border border-kong-accent-blue/30 rounded-lg p-6 mb-8">
        <div class="flex items-start space-x-3">
          <div class="text-2xl">💡</div>
          <div>
            <h3 class="text-xl font-bold text-kong-accent-blue mb-3">Understanding Your Addresses</h3>
            <div class="space-y-3 text-gray-300">
              <p>Kong Locker shows you two different addresses:</p>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
                <div class="bg-kong-accent-blue/20 p-4 rounded-lg border border-kong-accent-blue/30">
                  <h4 class="font-bold text-kong-accent-blue mb-2">Principal ID (Wallet Address)</h4>
                  <p class="text-sm">For ICP transactions only</p>
                </div>
                <div class="bg-kong-accent-green/20 p-4 rounded-lg border border-kong-accent-green/30">
                  <h4 class="font-bold text-kong-accent-green mb-2">Lock Address (Canister Address)</h4>
                  <p class="text-sm">For LP token transfers</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Step by Step Guide with Integrated Instructions -->
      <div class="space-y-6">
        <h3 class="text-2xl font-bold text-white mb-4">Step-by-Step Transfer Process</h3>
        
        <!-- Step 1 -->
        <div class="bg-kong-accent-blue/10 border border-kong-accent-blue/30 rounded-lg p-6">
          <div class="flex items-center space-x-3 mb-4">
            <div class="w-10 h-10 bg-kong-accent-blue rounded-full flex items-center justify-center text-white font-bold text-lg">1</div>
            <h4 class="text-xl font-bold text-kong-accent-blue">Create Lock Canister</h4>
          </div>
          <p class="text-gray-300 mb-4">Connect your wallet and create your lock canister for 2 ICP. This gives you a permanent lock address.</p>
          <div class="bg-kong-accent-blue/20 p-4 rounded border border-kong-accent-blue/30">
            <p class="text-sm text-gray-300">
              <strong>What happens:</strong> You pay 2 ICP (1 ICP for KongSwap registration, 1 ICP for canister operation). 
              The system creates your personal lock canister and immediately makes it autonomous (no controllers).
            </p>
          </div>
        </div>

        <!-- Step 2 -->
        <div class="bg-kong-accent-green/10 border border-kong-accent-green/30 rounded-lg p-6">
          <div class="flex items-center space-x-3 mb-4">
            <div class="w-10 h-10 bg-kong-accent-green rounded-full flex items-center justify-center text-white font-bold text-lg">2</div>
            <h4 class="text-xl font-bold text-kong-accent-green">Copy Your Lock Address</h4>
          </div>
          <p class="text-gray-300 mb-4">Copy your Lock Address (NOT your Principal ID) from the Kong Locker dashboard.</p>
          <div class="bg-kong-accent-green/20 p-4 rounded border border-kong-accent-green/30 mb-4">
            <p class="text-sm text-gray-300">
              <strong>Important:</strong> Your dashboard shows two addresses side by side. 
              Copy the one labeled "Lock Address" with the green "FOR LP TOKENS" badge.
            </p>
          </div>
          
          <!-- Address Type Guide -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="bg-kong-accent-blue/20 p-3 rounded border border-kong-accent-blue/30">
              <h5 class="font-bold text-kong-accent-blue text-sm mb-2">Principal ID (Wallet)</h5>
              <p class="text-xs text-gray-400">Your wallet address - for ICP transactions only</p>
            </div>
            <div class="bg-kong-accent-green/20 p-3 rounded border border-kong-accent-green/30">
              <h5 class="font-bold text-kong-accent-green text-sm mb-2">Lock Address (Canister)</h5>
              <p class="text-xs text-gray-400">Your lock canister - for LP token transfers</p>
            </div>
          </div>
        </div>

        <!-- Step 3 with Visual Guide -->
        <div class="bg-kong-accent-orange/10 border border-kong-accent-orange/30 rounded-lg p-6">
          <div class="flex items-center space-x-3 mb-4">
            <div class="w-10 h-10 bg-kong-accent-orange rounded-full flex items-center justify-center text-white font-bold text-lg">3</div>
            <h4 class="text-xl font-bold text-kong-accent-orange">Transfer on KongSwap</h4>
          </div>
          <p class="text-gray-300 mb-6">Go to KongSwap, select your LP tokens, and transfer them to your Lock Address.</p>
          
          <!-- KongSwap Interface Guide -->
          <div class="bg-gray-900/50 rounded-lg p-6 border border-gray-700">
            <h5 class="text-lg font-bold text-white mb-4">KongSwap Transfer Interface</h5>
            
            <!-- Image Container -->
            <div class="bg-black rounded-lg p-4 border-2 border-kong-accent-green/50 mb-6">
              <img 
                src="/kongswap_lp_transfer.png" 
                alt="KongSwap LP Token Transfer Interface showing how to send LP tokens to lock address" 
                class="w-full rounded-lg shadow-lg"
                loading="lazy"
              />
            </div>
            
            <!-- Key Instructions -->
            <div class="space-y-3">
              <h6 class="text-base font-bold text-kong-accent-green">Key Steps in KongSwap:</h6>
              <ul class="space-y-2 text-sm text-gray-300">
                <li class="flex items-start space-x-3">
                  <span class="text-kong-accent-green font-bold">1.</span>
                  <span><strong>Select LP Token Pair:</strong> Choose your LP tokens (e.g., ALEX/ckUSDT)</span>
                </li>
                <li class="flex items-start space-x-3">
                  <span class="text-kong-accent-green font-bold">2.</span>
                  <span><strong>Enter Amount:</strong> Use the number input or percentage buttons (25%, 50%, 75%, MAX)</span>
                </li>
                <li class="flex items-start space-x-3">
                  <span class="text-kong-accent-green font-bold">3.</span>
                  <span><strong>Paste Lock Address:</strong> In "Recipient Address" field, paste your Lock Address (the long canister ID)</span>
                </li>
                <li class="flex items-start space-x-3">
                  <span class="text-kong-accent-green font-bold">4.</span>
                  <span><strong>Click "Transfer LP Tokens":</strong> Confirm the transaction to permanently lock your tokens</span>
                </li>
              </ul>
            </div>
          </div>
        </div>

        <!-- Results -->
        <div class="bg-kong-accent-green/10 border border-kong-accent-green/30 rounded-lg p-6">
          <h4 class="text-xl font-bold text-kong-accent-green mb-3">🎉 After Transfer</h4>
          <p class="text-gray-300 mb-4">
            Once transferred, your LP tokens are <strong>permanently locked</strong> and you immediately gain voting power.
          </p>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div class="bg-kong-accent-green/20 p-4 rounded border border-kong-accent-green/30 text-center">
              <div class="text-2xl mb-2">🔒</div>
              <p class="text-sm font-semibold text-kong-accent-green">Tokens Locked Forever</p>
              <p class="text-xs text-gray-400">No unlock mechanism exists</p>
            </div>
            <div class="bg-kong-accent-green/20 p-4 rounded border border-kong-accent-green/30 text-center">
              <div class="text-2xl mb-2">📊</div>
              <p class="text-sm font-semibold text-kong-accent-green">Voting Power Active</p>
              <p class="text-xs text-gray-400">Based on USD value</p>
            </div>
            <div class="bg-kong-accent-green/20 p-4 rounded border border-kong-accent-green/30 text-center">
              <div class="text-2xl mb-2">🛡️</div>
              <p class="text-sm font-semibold text-kong-accent-green">Fully Autonomous</p>
              <p class="text-xs text-gray-400">Blackholed canister</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Cross-Site Identity Verification & Integration Section -->
    <section id="cross-site-verification" class="mb-12 bg-kong-bg-secondary rounded-2xl p-8 border border-kong-border">
      <h2 class="text-3xl font-bold text-white mb-6">🔗 Cross-Site Identity & Integration</h2>
      
      <!-- Identity Challenge Problem -->
      <div class="mb-8">
        <div class="bg-amber-900/20 border border-amber-600 rounded-lg p-6 mb-6">
          <h3 class="text-xl font-semibold text-amber-400 mb-3">⚠️ Cross-Site Identity Challenge</h3>
          <p class="text-amber-200 mb-4">
            On the Internet Computer, the same private key generates different principals on different websites due to security isolation. 
            This creates a challenge: <strong>users cannot directly prove they own locked liquidity on your site if they have a different principal.</strong>
          </p>
          <p class="text-amber-200">
            For example, if Alice locks LP tokens on KongLocker.com with principal <code class="bg-black/30 px-2 py-1 rounded text-xs">abc123...</code>, 
            she would have a completely different principal <code class="bg-black/30 px-2 py-1 rounded text-xs">xyz789...</code> on your website.
          </p>
        </div>
      </div>

      <!-- Solution: NFT Verification -->
      <div class="mb-8">
        <h3 class="text-2xl font-bold text-white mb-4">✅ Solution: NFT Screenshot Verification</h3>
        
        <div class="bg-blue-900/20 border border-blue-600 rounded-lg p-6 mb-4">
          <h4 class="text-lg font-semibold text-blue-400 mb-3">Method 1: Screenshot NFT (Simplest)</h4>
          <ol class="list-decimal list-inside text-blue-200 space-y-2">
            <li>User takes a screenshot of their locked canister on KongLocker.com</li>
            <li>User uploads the screenshot as an NFT on <strong>lbry.app</strong></li>
            <li>The NFT metadata shows it was created by their principal</li>
            <li>Verify the NFT creator matches the lock canister's creator principal</li>
            <li>✅ Ownership verified (sufficient for most use cases)</li>
          </ol>
          <p class="text-blue-300 text-sm mt-3 italic">
            Note: While screenshots can be spoofed, this method provides reasonable verification for most applications.
          </p>
        </div>

        <div class="bg-green-900/20 border border-green-600 rounded-lg p-6 mb-4">
          <h4 class="text-lg font-semibold text-green-400 mb-3">Method 2: Challenge Code NFT (More Secure)</h4>
          <ol class="list-decimal list-inside text-green-200 space-y-2">
            <li>Your site generates a unique challenge code</li>
            <li>User uploads this challenge code as an NFT on <strong>lbry.app</strong></li>
            <li>Since lbry.app uses the same authentication origin as KongLocker, principals match</li>
            <li>Verify the NFT was created by the lock canister's creator principal</li>
            <li>Confirm the NFT contains your challenge code</li>
            <li>✅ Cryptographically verified ownership!</li>
          </ol>
        </div>

        <div class="bg-yellow-900/20 border border-yellow-600 rounded-lg p-4">
          <h4 class="text-lg font-semibold text-yellow-400 mb-2">Why This Works</h4>
          <p class="text-yellow-200 text-sm">
            <strong>lbry.app</strong> uses the same authentication origin as KongLocker.com, meaning users have 
            identical principals on both sites. NFTs created on lbry.app provide an immutable, verifiable record
            of ownership that can be checked across different IC applications.
          </p>
        </div>
      </div>

      <!-- Primary Use Case: DAOPad -->
      <div class="mb-8">
        <h3 class="text-2xl font-bold text-white mb-4">🏛️ Primary Integration: DAOPad Governance</h3>
        
        <div class="bg-purple-900/20 border border-purple-600 rounded-lg p-6">
          <h4 class="text-lg font-semibold text-purple-400 mb-3">DAOPad.org - DAO Creation Platform</h4>
          <p class="text-purple-200 mb-4">
            The primary purpose of Kong Locker is to provide <strong>proof of liquidity commitment</strong> for governance rights on DAOPad.org:
          </p>
          <ul class="list-disc list-inside text-purple-200 space-y-2 ml-4">
            <li><strong>Create DAOs</strong> with voting power based on locked LP tokens</li>
            <li><strong>Treasury Management</strong> through verifiable liquidity commitments</li>
            <li><strong>Same Authentication</strong> - DAOPad uses identical principals as KongLocker and Alexandria</li>
            <li><strong>Seamless Integration</strong> - No identity verification needed between Kong Locker ↔ DAOPad</li>
          </ul>
        </div>
      </div>

      <!-- ICP Withdrawal Instructions -->
      <div class="mb-8">
        <h3 class="text-2xl font-bold text-white mb-4">💰 ICP Management & Withdrawal</h3>
        
        <div class="bg-orange-900/20 border border-orange-600 rounded-lg p-6">
          <h4 class="text-lg font-semibold text-orange-400 mb-3">⚠️ Excess ICP Cannot Be Withdrawn Here</h4>
          <p class="text-orange-200 mb-4">
            If you sent more than 2 ICP to create your lock canister, the excess ICP remains in your account but 
            <strong>cannot be withdrawn from KongLocker.com</strong>.
          </p>
          
          <div class="bg-black/30 border border-orange-500 rounded-lg p-4 mt-4">
            <h5 class="text-orange-300 font-semibold mb-2">To withdraw excess ICP:</h5>
            <ol class="list-decimal list-inside text-orange-200 space-y-1 text-sm">
              <li>Go to <strong>LBRY.app</strong> (the core authentication site)</li>
              <li>Log in with the same wallet you used here</li>
              <li>Your ICP balance will be identical (same database origin)</li>
              <li>Use LBRY.app's withdrawal functionality to send ICP to external wallets</li>
            </ol>
          </div>
        </div>
      </div>

      <!-- Integration Summary -->
      <div class="bg-gray-900/50 rounded-lg p-6">
        <h3 class="text-xl font-bold text-white mb-3">🔗 Authentication Origin Ecosystem</h3>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div class="bg-blue-900/20 border border-blue-600 rounded-lg p-4">
            <h4 class="text-blue-400 font-semibold">KongLocker.com</h4>
            <p class="text-blue-200 text-xs mt-1">Lock LP tokens, same principals as other sites</p>
          </div>
          <div class="bg-green-900/20 border border-green-600 rounded-lg p-4">
            <h4 class="text-green-400 font-semibold">lbry.app</h4>
            <p class="text-green-200 text-xs mt-1">Social verification platform, same principals</p>
          </div>
          <div class="bg-purple-900/20 border border-purple-600 rounded-lg p-4">
            <h4 class="text-purple-400 font-semibold">DAOPad.org</h4>
            <p class="text-purple-200 text-xs mt-1">DAO governance, same principals</p>
          </div>
        </div>
        <div class="mt-4 bg-yellow-900/20 border border-yellow-600 rounded-lg p-4">
          <h4 class="text-yellow-400 font-semibold">LBRY.app</h4>
          <p class="text-yellow-200 text-xs mt-1">Core site for ICP withdrawals and advanced features</p>
        </div>
      </div>
    </section>

    <!-- Developer Integration Section Header -->
    <div class="mb-6">
      <div class="border-t border-gray-700 my-8"></div>
      <button
        on:click={() => developerSectionExpanded = !developerSectionExpanded}
        class="w-full bg-gray-900/50 hover:bg-gray-800/50 border border-gray-700 rounded-2xl p-6 transition-all duration-200 group"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <span class="text-2xl">🔧</span>
            <div class="text-left">
              <h2 class="text-2xl font-bold text-white">Developer Integration</h2>
              <p class="text-gray-400 text-sm mt-1">
                API documentation, code examples, and integration guides for developers
              </p>
            </div>
          </div>
          <div class="text-gray-400 group-hover:text-white transition-colors">
            <span class="text-xl transform transition-transform duration-200 {developerSectionExpanded ? 'rotate-180' : ''}">
              ▼
            </span>
          </div>
        </div>
      </button>
    </div>

    {#if developerSectionExpanded}
    <!-- API Overview Section -->
    <section id="overview" class="mb-12 bg-[#1a1b1f] rounded-2xl p-8 border border-gray-800">
      <h2 class="text-3xl font-bold text-white mb-6">🔍 API Overview</h2>
      
      <div class="bg-blue-900/20 border border-blue-800 rounded-lg p-4 mb-6">
        <h3 class="text-lg font-semibold text-blue-400 mb-2">🔄 API Migration Update</h3>
        <p class="text-blue-200 text-sm mb-2">
          <strong>Breaking Change:</strong> Voting power and value locked queries have been moved to direct KongSwap integration for better performance.
        </p>
        <ul class="list-disc list-inside text-blue-200 text-sm space-y-1">
          <li><code>get_voting_power</code>, <code>get_all_voting_powers</code>, and <code>get_total_value_locked</code> → Moved to frontend-side KongSwap queries</li>
          <li>Query calls are now free and much faster</li>
          <li>Full LP pool breakdown available (not just USD totals)</li>
        </ul>
      </div>
      
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-gray-700">
              <th class="text-left py-3 px-4 text-gray-400">Method</th>
              <th class="text-left py-3 px-4 text-gray-400">Type</th>
              <th class="text-left py-3 px-4 text-gray-400">Interactive</th>
              <th class="text-left py-3 px-4 text-gray-400">Description</th>
            </tr>
          </thead>
          <tbody class="text-gray-300">
            <tr class="border-b border-gray-800">
              <td class="py-3 px-4 font-mono">get_my_lock_canister</td>
              <td class="py-3 px-4"><span class="bg-green-900/30 text-green-400 px-2 py-1 rounded text-xs">Query</span></td>
              <td class="py-3 px-4">✅ Yes</td>
              <td class="py-3 px-4">Get caller's lock canister</td>
            </tr>
            <tr class="border-b border-gray-800">
              <td class="py-3 px-4 font-mono">get_all_lock_canisters</td>
              <td class="py-3 px-4"><span class="bg-green-900/30 text-green-400 px-2 py-1 rounded text-xs">Query</span></td>
              <td class="py-3 px-4">✅ Yes</td>
              <td class="py-3 px-4">List all user-canister mappings</td>
            </tr>
            <tr class="border-b border-gray-800">
              <td class="py-3 px-4 font-mono">get_total_positions_count</td>
              <td class="py-3 px-4"><span class="bg-green-900/30 text-green-400 px-2 py-1 rounded text-xs">Query</span></td>
              <td class="py-3 px-4">✅ Yes</td>
              <td class="py-3 px-4">Count of unique lock positions</td>
            </tr>
            <tr class="border-b border-gray-800">
              <td class="py-3 px-4 font-mono">get_analytics_overview</td>
              <td class="py-3 px-4"><span class="bg-green-900/30 text-green-400 px-2 py-1 rounded text-xs">Query</span></td>
              <td class="py-3 px-4">✅ Yes</td>
              <td class="py-3 px-4">Complete analytics data for dashboard</td>
            </tr>
            <tr class="border-b border-gray-800">
              <td class="py-3 px-4 font-mono">get_detailed_canister_status</td>
              <td class="py-3 px-4"><span class="bg-yellow-900/30 text-yellow-400 px-2 py-1 rounded text-xs">Update</span></td>
              <td class="py-3 px-4">✅ Yes</td>
              <td class="py-3 px-4">Detailed canister information</td>
            </tr>
            <tr class="border-b border-gray-800">
              <td class="py-3 px-4 font-mono">create_lock_canister</td>
              <td class="py-3 px-4"><span class="bg-blue-900/30 text-blue-400 px-2 py-1 rounded text-xs">Update</span></td>
              <td class="py-3 px-4">❌ No</td>
              <td class="py-3 px-4">Create new lock canister</td>
            </tr>
            <tr class="border-b border-gray-800">
              <td class="py-3 px-4 font-mono">complete_my_canister_setup</td>
              <td class="py-3 px-4"><span class="bg-blue-900/30 text-blue-400 px-2 py-1 rounded text-xs">Update</span></td>
              <td class="py-3 px-4">❌ No</td>
              <td class="py-3 px-4">Complete partial setup</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <!-- Query Functions Section -->
    <section id="query-functions" class="mb-12 bg-[#1a1b1f] rounded-2xl border border-gray-800">
      <!-- Sticky Section Header -->
      <div class="sticky top-0 z-10 bg-[#1a1b1f] rounded-t-2xl p-6 border-b border-gray-800">
        <h2 class="text-3xl font-bold text-white flex items-center gap-3">
          <span class="text-2xl">🔎</span>
          <span>Query Functions</span>
          <span class="text-sm font-normal text-gray-400 ml-auto">Fast, free canister queries</span>
        </h2>
      </div>
      <div class="p-8 pt-6">
      
        <!-- get_my_lock_canister - Compact Card -->
        <div class="mb-4 bg-gray-900/50 rounded-lg border border-gray-700 overflow-hidden">
          <div class="flex items-center justify-between p-4 pb-3">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="text-lg font-bold text-white">get_my_lock_canister</h3>
                <span class="px-2 py-0.5 bg-blue-900/30 text-blue-300 text-xs rounded border border-blue-800">
                  Kong Locker: eazgb-g...
                </span>
              </div>
              <p class="text-gray-400 text-sm">Returns your lock canister Principal</p>
            </div>
            <div class="flex items-center gap-2">
              <button 
                on:click={callGetMyLockCanister}
                disabled={apiLoading['get_my_lock_canister']}
                class="px-4 py-2 bg-purple-600 hover:bg-purple-500 disabled:bg-gray-600 text-white text-sm rounded-lg transition-colors flex items-center gap-2"
              >
                {#if apiLoading['get_my_lock_canister']}
                  <Loader class="w-4 h-4 animate-spin" />
                  <span>Calling...</span>
                {:else}
                  <span>Try Now</span>
                {/if}
              </button>
            </div>
          </div>
          
          <!-- Compact Interface Display -->
          <div class="px-4 pb-3">
            <div class="bg-black/40 rounded px-3 py-2 text-xs font-mono">
              <span class="text-green-400">() -> (opt principal) query</span>
            </div>
          </div>
          
          <!-- Results Area -->
          {#if apiResults['get_my_lock_canister'] || apiErrors['get_my_lock_canister']}
            <div class="border-t border-gray-700 px-4 py-3 bg-black/20">
              {#if apiResults['get_my_lock_canister']}
                <div class="text-sm">
                  <span class="text-green-400 text-xs">✅ Result:</span>
                  <pre class="text-green-300 font-mono mt-1">{apiResults['get_my_lock_canister']}</pre>
                </div>
              {/if}
              
              {#if apiErrors['get_my_lock_canister']}
                <div class="text-sm">
                  <span class="text-red-400 text-xs">❌ Error:</span>
                  <span class="text-red-300 ml-2">{apiErrors['get_my_lock_canister']}</span>
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- get_all_lock_canisters - Compact Card -->
        <div class="mb-4 bg-gray-900/50 rounded-lg border border-gray-700 overflow-hidden">
          <div class="flex items-center justify-between p-4 pb-3">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="text-lg font-bold text-white">get_all_lock_canisters</h3>
                <span class="px-2 py-0.5 bg-blue-900/30 text-blue-300 text-xs rounded border border-blue-800">
                  Kong Locker: eazgb-g...
                </span>
              </div>
              <p class="text-gray-400 text-sm">Returns all user-to-lock-canister mappings</p>
            </div>
            <div class="flex items-center gap-2">
              <button 
                on:click={callGetAllLockCanisters}
                disabled={apiLoading['get_all_lock_canisters']}
                class="px-4 py-2 bg-purple-600 hover:bg-purple-500 disabled:bg-gray-600 text-white text-sm rounded-lg transition-colors flex items-center gap-2"
              >
                {#if apiLoading['get_all_lock_canisters']}
                  <Loader class="w-4 h-4 animate-spin" />
                  <span>Calling...</span>
                {:else}
                  <span>Try Now</span>
                {/if}
              </button>
            </div>
          </div>
          
          <!-- Compact Interface Display -->
          <div class="px-4 pb-3">
            <div class="bg-black/40 rounded px-3 py-2 text-xs font-mono">
              <span class="text-green-400">() -> (vec record &#123; principal; principal &#125;) query</span>
            </div>
          </div>
          
          <!-- Results Area -->
          {#if apiResults['get_all_lock_canisters'] || apiErrors['get_all_lock_canisters']}
            <div class="border-t border-gray-700 px-4 py-3 bg-black/20">
              {#if apiResults['get_all_lock_canisters']}
                <div class="text-sm">
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-green-400 text-xs">✅ Result ({apiResults['get_all_lock_canisters'].length} entries):</span>
                    {#if apiResults['get_all_lock_canisters'].length > 3}
                      <button 
                        on:click={() => toggleExpanded('get_all_lock_canisters_full')}
                        class="text-xs text-blue-400 hover:text-blue-300"
                      >
                        {expandedSections['get_all_lock_canisters_full'] ? '▲ Show Less' : '▼ Show All'}
                      </button>
                    {/if}
                  </div>
                  <pre class="text-green-300 font-mono text-xs max-h-32 overflow-y-auto">
{JSON.stringify(
  expandedSections['get_all_lock_canisters_full'] 
    ? apiResults['get_all_lock_canisters'] 
    : apiResults['get_all_lock_canisters'].slice(0, 3), 
  null, 2
)}</pre>
                  {#if apiResults['get_all_lock_canisters'].length > 3 && !expandedSections['get_all_lock_canisters_full']}
                    <p class="text-xs text-gray-500 mt-1">... and {apiResults['get_all_lock_canisters'].length - 3} more</p>
                  {/if}
                </div>
              {/if}
              
              {#if apiErrors['get_all_lock_canisters']}
                <div class="text-sm">
                  <span class="text-red-400 text-xs">❌ Error:</span>
                  <span class="text-red-300 ml-2">{apiErrors['get_all_lock_canisters']}</span>
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- get_total_positions_count - Compact Card -->
        <div class="mb-4 bg-gray-900/50 rounded-lg border border-gray-700 overflow-hidden">
          <div class="flex items-center justify-between p-4 pb-3">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="text-lg font-bold text-white">get_total_positions_count</h3>
                <span class="px-2 py-0.5 bg-blue-900/30 text-blue-300 text-xs rounded border border-blue-800">
                  Kong Locker: eazgb-g...
                </span>
              </div>
              <p class="text-gray-400 text-sm">Returns total number of unique lock positions</p>
            </div>
            <div class="flex items-center gap-2">
              <button 
                on:click={callGetTotalPositionsCount}
                disabled={apiLoading['get_total_positions_count']}
                class="px-4 py-2 bg-purple-600 hover:bg-purple-500 disabled:bg-gray-600 text-white text-sm rounded-lg transition-colors flex items-center gap-2"
              >
                {#if apiLoading['get_total_positions_count']}
                  <Loader class="w-4 h-4 animate-spin" />
                  <span>Calling...</span>
                {:else}
                  <span>Try Now</span>
                {/if}
              </button>
            </div>
          </div>
          
          <!-- Compact Interface Display -->
          <div class="px-4 pb-3">
            <div class="bg-black/40 rounded px-3 py-2 text-xs font-mono">
              <span class="text-green-400">() -> (nat64) query</span>
            </div>
          </div>
          
          <!-- Results Area -->
          {#if apiResults['get_total_positions_count'] || apiErrors['get_total_positions_count']}
            <div class="border-t border-gray-700 px-4 py-3 bg-black/20">
              {#if apiResults['get_total_positions_count']}
                <div class="text-sm">
                  <span class="text-green-400 text-xs">✅ Result:</span>
                  <span class="text-green-300 ml-2 font-mono">{apiResults['get_total_positions_count']} total positions</span>
                </div>
              {/if}
              
              {#if apiErrors['get_total_positions_count']}
                <div class="text-sm">
                  <span class="text-red-400 text-xs">❌ Error:</span>
                  <span class="text-red-300 ml-2">{apiErrors['get_total_positions_count']}</span>
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- get_analytics_overview - Compact Card -->
        <div class="mb-4 bg-gray-900/50 rounded-lg border border-gray-700 overflow-hidden">
          <div class="flex items-center justify-between p-4 pb-3">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="text-lg font-bold text-white">get_analytics_overview</h3>
                <span class="px-2 py-0.5 bg-blue-900/30 text-blue-300 text-xs rounded border border-blue-800">
                  Kong Locker: eazgb-g...
                </span>
              </div>
              <p class="text-gray-400 text-sm">Complete analytics data for dashboards 💡</p>
            </div>
            <div class="flex items-center gap-2">
              <button 
                on:click={callGetAnalyticsOverview}
                disabled={apiLoading['get_analytics_overview']}
                class="px-4 py-2 bg-purple-600 hover:bg-purple-500 disabled:bg-gray-600 text-white text-sm rounded-lg transition-colors flex items-center gap-2"
              >
                {#if apiLoading['get_analytics_overview']}
                  <Loader class="w-4 h-4 animate-spin" />
                  <span>Calling...</span>
                {:else}
                  <span>Try Now</span>
                {/if}
              </button>
            </div>
          </div>
          
          <!-- Compact Interface Display -->
          <div class="px-4 pb-3">
            <div class="bg-black/40 rounded px-3 py-2 text-xs font-mono">
              <span class="text-green-400">() -> (AnalyticsOverview) query</span>
            </div>
          </div>
          
          <!-- Results Area -->
          {#if apiResults['get_analytics_overview'] || apiErrors['get_analytics_overview']}
            <div class="border-t border-gray-700 px-4 py-3 bg-black/20">
              {#if apiResults['get_analytics_overview']}
                <div class="text-sm space-y-3">
                  <span class="text-green-400 text-xs">✅ Analytics Overview:</span>
                  
                  <!-- Compact Summary Stats -->
                  <div class="bg-green-900/20 border border-green-800 rounded p-3">
                    <div class="grid grid-cols-3 gap-4 text-center text-sm">
                      <div>
                        <div class="text-lg font-bold text-white">{apiResults['get_analytics_overview'].total_lock_canisters}</div>
                        <div class="text-xs text-gray-400">Total Canisters</div>
                      </div>
                      <div>
                        <div class="text-lg font-bold text-green-400">{apiResults['get_analytics_overview'].participant_count}</div>
                        <div class="text-xs text-gray-400">Participants</div>
                      </div>
                      <div>
                        <div class="text-xs text-blue-400">{apiResults['get_analytics_overview'].last_updated}</div>
                        <div class="text-xs text-gray-400">Last Updated</div>
                      </div>
                    </div>
                  </div>
                  
                  <!-- Expandable Participants -->
                  {#if apiResults['get_analytics_overview'].participants.length > 0}
                    <div class="border-t border-gray-600 pt-2">
                      <button 
                        on:click={() => toggleExpanded('analytics_participants')}
                        class="text-xs text-blue-400 hover:text-blue-300 mb-2"
                      >
                        {expandedSections['analytics_participants'] ? '▲ Hide' : '▼ Show'} Participants ({apiResults['get_analytics_overview'].showing_first})
                      </button>
                      {#if expandedSections['analytics_participants']}
                        <div class="bg-blue-900/20 border border-blue-800 rounded p-2">
                          <div class="space-y-1 font-mono text-xs">
                            {#each apiResults['get_analytics_overview'].participants as participant}
                              <div class="flex justify-between text-gray-300">
                                <span>User: {participant.user.substring(0, 12)}...</span>
                                <span class="text-gray-500">→</span>
                                <span>Lock: {participant.canister.substring(0, 12)}...</span>
                              </div>
                            {/each}
                          </div>
                        </div>
                      {/if}
                    </div>
                  {/if}
                </div>
              {/if}
              
              {#if apiErrors['get_analytics_overview']}
                <div class="text-sm">
                  <span class="text-red-400 text-xs">❌ Error:</span>
                  <span class="text-red-300 ml-2">{apiErrors['get_analytics_overview']}</span>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    </section>

    <!-- Update Functions Section -->
    <section id="update-functions" class="mb-12 bg-[#1a1b1f] rounded-2xl border border-gray-800">
      <!-- Sticky Section Header -->
      <div class="sticky top-0 z-10 bg-[#1a1b1f] rounded-t-2xl p-6 border-b border-gray-800">
        <h2 class="text-3xl font-bold text-white flex items-center gap-3">
          <span class="text-2xl">✏️</span>
          <span>Update Functions</span>
          <span class="text-sm font-normal text-gray-400 ml-auto">State-changing canister calls</span>
        </h2>
      </div>
      <div class="p-8 pt-6">
      
        <!-- get_detailed_canister_status - Compact Card -->
        <div class="mb-4 bg-gray-900/50 rounded-lg border border-gray-700 overflow-hidden">
          <div class="flex items-center justify-between p-4 pb-3">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="text-lg font-bold text-white">get_detailed_canister_status</h3>
                <span class="px-2 py-0.5 bg-yellow-900/30 text-yellow-300 text-xs rounded border border-yellow-800">
                  Kong Locker: eazgb-g...
                </span>
              </div>
              <p class="text-gray-400 text-sm">Detailed lock canister info (blackhole status, cycles, etc.)</p>
            </div>
            <div class="flex items-center gap-2">
              <button 
                on:click={callGetDetailedCanisterStatus}
                disabled={apiLoading['get_detailed_canister_status']}
                class="px-4 py-2 bg-purple-600 hover:bg-purple-500 disabled:bg-gray-600 text-white text-sm rounded-lg transition-colors flex items-center gap-2"
              >
                {#if apiLoading['get_detailed_canister_status']}
                  <Loader class="w-4 h-4 animate-spin" />
                  <span>Calling...</span>
                {:else}
                  <span>Try Now</span>
                {/if}
              </button>
              <button 
                on:click={() => toggleExpanded('detailed_canister_type')}
                class="text-xs text-blue-400 hover:text-blue-300 px-2 py-1 border border-blue-400/30 rounded"
              >
                {expandedSections['detailed_canister_type'] ? 'Hide' : 'Show'} Type
              </button>
            </div>
          </div>
          
          <!-- Type Definition (Expandable) -->
          {#if expandedSections['detailed_canister_type']}
            <div class="px-4 pb-3 border-b border-gray-700">
              <div class="bg-black/40 rounded p-3 text-xs font-mono">
                <span class="text-green-400">type DetailedCanisterStatus = &#123;</span>
                <div class="ml-2 text-green-300">
                  <div>canister_id: Principal;</div>
                  <div>is_blackholed: bool;</div>
                  <div>controller_count: nat32;</div>
                  <div>cycle_balance: nat;</div>
                  <div>memory_size: nat;</div>
                  <div>module_hash: opt blob;</div>
                </div>
                <span class="text-green-400">&#125;</span>
              </div>
            </div>
          {/if}
          
          <!-- Compact Interface Display -->
          <div class="px-4 pb-3">
            <div class="bg-black/40 rounded px-3 py-2 text-xs font-mono">
              <span class="text-yellow-400">() -> (variant &#123; Ok : DetailedCanisterStatus; Err : text &#125;) [update]</span>
            </div>
          </div>
          
          <!-- Results Area -->
          {#if apiResults['get_detailed_canister_status'] || apiErrors['get_detailed_canister_status']}
            <div class="border-t border-gray-700 px-4 py-3 bg-black/20">
              {#if apiResults['get_detailed_canister_status']}
                <div class="text-sm">
                  <span class="text-green-400 text-xs">✅ Canister Status:</span>
                  <div class="grid grid-cols-2 gap-3 mt-2 text-xs">
                    <div class="bg-gray-800/50 rounded p-2">
                      <div class="text-gray-400">ID:</div>
                      <div class="text-green-300 font-mono">{apiResults['get_detailed_canister_status'].canister_id}</div>
                    </div>
                    <div class="bg-gray-800/50 rounded p-2">
                      <div class="text-gray-400">Blackholed:</div>
                      <div class="{apiResults['get_detailed_canister_status'].is_blackholed ? 'text-green-400' : 'text-yellow-400'} font-bold">
                        {apiResults['get_detailed_canister_status'].is_blackholed ? '✅ Yes' : '⚠️ No'}
                      </div>
                    </div>
                    <div class="bg-gray-800/50 rounded p-2">
                      <div class="text-gray-400">Controllers:</div>
                      <div class="text-blue-300">{apiResults['get_detailed_canister_status'].controller_count}</div>
                    </div>
                    <div class="bg-gray-800/50 rounded p-2">
                      <div class="text-gray-400">Memory:</div>
                      <div class="text-blue-300">{apiResults['get_detailed_canister_status'].memory_size}</div>
                    </div>
                  </div>
                </div>
              {/if}
              
              {#if apiErrors['get_detailed_canister_status']}
                <div class="text-sm">
                  <span class="text-red-400 text-xs">❌ Error:</span>
                  <span class="text-red-300 ml-2">{apiErrors['get_detailed_canister_status']}</span>
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- create_lock_canister - Info Card (Non-interactive) -->
        <div class="mb-4 bg-gray-900/50 rounded-lg border border-orange-700/50 overflow-hidden">
          <div class="flex items-center justify-between p-4 pb-3">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="text-lg font-bold text-white">create_lock_canister</h3>
                <span class="px-2 py-0.5 bg-blue-900/30 text-blue-300 text-xs rounded border border-blue-800">
                  Kong Locker: eazgb-g...
                </span>
              </div>
              <p class="text-gray-400 text-sm">Creates permanent lock canister 🔒 Requires 2 ICP</p>
            </div>
            <div class="flex items-center gap-2">
              <span class="px-3 py-1 bg-orange-900/30 text-orange-400 text-xs rounded-full border border-orange-800">
                ⚠️ Payment Required
              </span>
            </div>
          </div>
          
          <!-- Warning Messages (Collapsible) -->
          <div class="px-4 pb-3">
            <button 
              on:click={() => toggleExpanded('create_canister_warnings')}
              class="text-xs text-orange-400 hover:text-orange-300 mb-2"
            >
              {expandedSections['create_canister_warnings'] ? '▲ Hide' : '▼ Show'} Important Warnings
            </button>
            {#if expandedSections['create_canister_warnings']}
              <div class="space-y-2">
                <div class="bg-red-900/20 border border-red-800 rounded p-2 text-xs">
                  <span class="text-red-400">🔒 PERMANENT:</span>
                  <span class="text-red-200">LP tokens sent to lock canisters can NEVER be withdrawn</span>
                </div>
                <div class="bg-yellow-900/20 border border-yellow-800 rounded p-2 text-xs">
                  <span class="text-yellow-400">⚠️ TESTING:</span>
                  <span class="text-yellow-200">Not available for interactive testing - requires ICP payment</span>
                </div>
              </div>
            {/if}
          </div>
          
          <!-- Compact Interface Display -->
          <div class="px-4 pb-3">
            <div class="bg-black/40 rounded px-3 py-2 text-xs font-mono">
              <span class="text-blue-400">() -> (variant &#123; Ok : principal; Err : text &#125;) [update]</span>
            </div>
          </div>
        </div>

        <!-- complete_my_canister_setup - Info Card (Non-interactive) -->
        <div class="mb-4 bg-gray-900/50 rounded-lg border border-yellow-700/50 overflow-hidden">
          <div class="flex items-center justify-between p-4 pb-3">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="text-lg font-bold text-white">complete_my_canister_setup</h3>
                <span class="px-2 py-0.5 bg-blue-900/30 text-blue-300 text-xs rounded border border-blue-800">
                  Kong Locker: eazgb-g...
                </span>
              </div>
              <p class="text-gray-400 text-sm">Recovery function for partial setups 🔧</p>
            </div>
            <div class="flex items-center gap-2">
              <span class="px-3 py-1 bg-yellow-900/30 text-yellow-400 text-xs rounded-full border border-yellow-800">
                🔧 Recovery Only
              </span>
            </div>
          </div>
          
          <!-- Usage Info (Collapsible) -->
          <div class="px-4 pb-3">
            <button 
              on:click={() => toggleExpanded('setup_recovery_info')}
              class="text-xs text-yellow-400 hover:text-yellow-300 mb-2"
            >
              {expandedSections['setup_recovery_info'] ? '▲ Hide' : '▼ Show'} When to Use
            </button>
            {#if expandedSections['setup_recovery_info']}
              <div class="bg-yellow-900/20 border border-yellow-800 rounded p-2 text-xs text-yellow-200">
                Use when canister creation partially failed. This completes: code installation, ICP funding, KongSwap registration, and blackholing.
              </div>
            {/if}
          </div>
          
          <!-- Compact Interface Display -->
          <div class="px-4 pb-3">
            <div class="bg-black/40 rounded px-3 py-2 text-xs font-mono">
              <span class="text-blue-400">() -> (variant &#123; Ok : text; Err : text &#125;) [update]</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- KongSwap Direct Integration Section -->
    <section id="kongswap-integration" class="mb-12 bg-[#1a1b1f] rounded-2xl border border-gray-800">
      <!-- Sticky Section Header -->
      <div class="sticky top-0 z-10 bg-[#1a1b1f] rounded-t-2xl p-6 border-b border-gray-800">
        <h2 class="text-3xl font-bold text-white flex items-center gap-3">
          <span class="text-2xl">🔗</span>
          <span>KongSwap Direct Queries</span>
          <span class="text-sm font-normal text-gray-400 ml-auto">Frontend-side LP data queries</span>
        </h2>
      </div>
      <div class="p-8 pt-6">
      
        <!-- Performance Benefits Summary -->
        <div class="mb-6 bg-green-900/20 border border-green-800 rounded-lg">
          <div class="p-4 pb-3">
            <button 
              on:click={() => toggleExpanded('performance_benefits')}
              class="w-full text-left flex items-center justify-between"
            >
              <h3 class="text-lg font-semibold text-green-400">🚀 Performance Upgrade</h3>
              <span class="text-green-400">{expandedSections['performance_benefits'] ? '▲' : '▼'}</span>
            </button>
            {#if expandedSections['performance_benefits']}
              <div class="mt-3 pt-3 border-t border-green-800">
                <p class="text-green-200 text-sm mb-3">Direct KongSwap queries from frontend provide:</p>
                <div class="grid grid-cols-2 gap-2 text-sm">
                  <div class="bg-green-800/20 rounded p-2">
                    <div class="text-green-300 font-medium">Free queries</div>
                    <div class="text-green-200 text-xs">No cycle costs</div>
                  </div>
                  <div class="bg-green-800/20 rounded p-2">
                    <div class="text-green-300 font-medium">Faster responses</div>
                    <div class="text-green-200 text-xs">Parallel requests</div>
                  </div>
                  <div class="bg-green-800/20 rounded p-2">
                    <div class="text-green-300 font-medium">Detailed breakdowns</div>
                    <div class="text-green-200 text-xs">Full LP pool data</div>
                  </div>
                  <div class="bg-green-800/20 rounded p-2">
                    <div class="text-green-300 font-medium">Real-time data</div>
                    <div class="text-green-200 text-xs">No backend caching</div>
                  </div>
                </div>
              </div>
            {/if}
          </div>
        </div>
      
        <!-- Integration Architecture (Collapsible) -->
        <div class="mb-6 bg-gray-900/50 rounded-lg border border-gray-700">
          <div class="p-4 pb-3">
            <button 
              on:click={() => toggleExpanded('integration_architecture')}
              class="w-full text-left flex items-center justify-between"
            >
              <h3 class="text-lg font-bold text-white">Integration Architecture</h3>
              <span class="text-gray-400">{expandedSections['integration_architecture'] ? '▲' : '▼'}</span>
            </button>
            <p class="text-gray-400 text-sm mt-1">3-step pattern for LP data queries</p>
            {#if expandedSections['integration_architecture']}
              <div class="mt-4 pt-3 border-t border-gray-700">
                <div class="bg-black/40 rounded p-3">
                  <pre class="text-sm text-blue-400 font-mono overflow-x-auto">{`// Step 1: Get lock canister from Kong Locker
const lockCanister = await kongLocker.get_my_lock_canister();

// Step 2: Query KongSwap directly for LP positions
const positions = await kongSwap.user_balances(lockCanister.toText());

// Step 3: Calculate voting power client-side
const votingPower = positions.reduce((sum, p) => sum + p.usd_balance * 100, 0);`}</pre>
                </div>
              </div>
            {/if}
          </div>
        </div>
      
        <!-- KongSwap user_balances API - Compact Info Card -->
        <div class="mb-4 bg-gray-900/50 rounded-lg border border-gray-700">
          <div class="p-4">
            <div class="flex items-center gap-2 mb-1">
              <h3 class="text-lg font-bold text-white">KongSwap user_balances API</h3>
              <span class="px-2 py-0.5 bg-purple-900/30 text-purple-300 text-xs rounded border border-purple-800">
                KongSwap: 2ipq2-u...
              </span>
            </div>
            <p class="text-gray-400 text-sm mb-3">Direct query to KongSwap for LP positions</p>
            
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div class="bg-black/40 rounded p-3">
                <div class="text-xs text-gray-500 mb-1">Candid Interface</div>
                <div class="text-xs font-mono text-green-400">user_balances : (text) -> (variant &#123; Ok : vec UserBalancesReply; Err : text &#125;) query</div>
              </div>
              <div class="bg-black/40 rounded p-3">
                <div class="text-xs text-gray-500 mb-1">KongSwap Canister ID</div>
                <div class="text-xs font-mono text-purple-400">2ipq2-uqaaa-aaaar-qailq-cai</div>
              </div>
            </div>
          </div>
        </div>
      
        <!-- Frontend Service Implementation (Collapsible) -->
        <div class="mb-6 bg-gray-900/50 rounded-lg border border-gray-700">
          <div class="p-4 pb-3">
            <button 
              on:click={() => toggleExpanded('frontend_service_impl')}
              class="w-full text-left flex items-center justify-between"
            >
              <h3 class="text-lg font-bold text-white">Frontend Service Implementation</h3>
              <span class="text-gray-400">{expandedSections['frontend_service_impl'] ? '▲' : '▼'}</span>
            </button>
            <p class="text-gray-400 text-sm mt-1">Ready-to-use TypeScript service</p>
            {#if expandedSections['frontend_service_impl']}
              <div class="mt-4 pt-3 border-t border-gray-700">
                <div class="bg-black/40 rounded p-3 relative">
                  <button 
                    on:click={() => copyCode(`import { KongSwapDirectService } from './services/kongSwapDirect';

const kongSwap = new KongSwapDirectService();

// Get LP positions for a lock canister
const positions = await kongSwap.getLPPositions(lockCanisterPrincipal);

// Calculate voting power (USD * 100)
const votingPower = kongSwap.calculateVotingPower(positions);

// Get user's complete LP breakdown
const breakdown = await kongSwap.getUserLPBreakdown(userPrincipal, kongLocker);`, 'kongswap-service')}
                    class="absolute top-2 right-2 text-xs px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
                  >
                    {copySuccess === 'kongswap-service' ? '✅ Copied!' : '📋 Copy'}
                  </button>
                  <pre class="text-sm text-purple-400 font-mono overflow-x-auto pr-16">{`import { KongSwapDirectService } from './services/kongSwapDirect';

const kongSwap = new KongSwapDirectService();

// Get LP positions for a lock canister
const positions = await kongSwap.getLPPositions(lockCanisterPrincipal);

// Calculate voting power (USD * 100)
const votingPower = kongSwap.calculateVotingPower(positions);

// Get user's complete LP breakdown
const breakdown = await kongSwap.getUserLPBreakdown(userPrincipal, kongLocker);`}</pre>
                </div>
              </div>
            {/if}
          </div>
        </div>
      
        <!-- Available Methods (Collapsible) -->
        <div class="mb-6 bg-gray-900/50 rounded-lg border border-gray-700">
          <div class="p-4 pb-3">
            <button 
              on:click={() => toggleExpanded('available_methods')}
              class="w-full text-left flex items-center justify-between"
            >
              <h3 class="text-lg font-bold text-white">Available Methods</h3>
              <span class="text-gray-400">{expandedSections['available_methods'] ? '▲' : '▼'}</span>
            </button>
            <p class="text-gray-400 text-sm mt-1">5 methods for LP data management</p>
            {#if expandedSections['available_methods']}
              <div class="mt-4 pt-3 border-t border-gray-700">
                <div class="overflow-x-auto">
                  <table class="w-full text-sm">
                    <thead>
                      <tr class="border-b border-gray-700">
                        <th class="text-left py-2 px-3 text-gray-400 text-xs">Method</th>
                        <th class="text-left py-2 px-3 text-gray-400 text-xs">Purpose</th>
                        <th class="text-left py-2 px-3 text-gray-400 text-xs">Returns</th>
                      </tr>
                    </thead>
                    <tbody class="text-gray-300">
                      <tr class="border-b border-gray-800">
                        <td class="py-2 px-3 font-mono text-xs">getLPPositions(principal)</td>
                        <td class="py-2 px-3 text-xs">Get detailed LP positions</td>
                        <td class="py-2 px-3 text-xs">LPReply[]</td>
                      </tr>
                      <tr class="border-b border-gray-800">
                        <td class="py-2 px-3 font-mono text-xs">calculateVotingPower(positions)</td>
                        <td class="py-2 px-3 text-xs">Calculate voting power</td>
                        <td class="py-2 px-3 text-xs">number (cents)</td>
                      </tr>
                      <tr class="border-b border-gray-800">
                        <td class="py-2 px-3 font-mono text-xs">getAllVotingPowers(lockCanisters)</td>
                        <td class="py-2 px-3 text-xs">Batch calculate all powers</td>
                        <td class="py-2 px-3 text-xs">[Principal, number][]</td>
                      </tr>
                      <tr class="border-b border-gray-800">
                        <td class="py-2 px-3 font-mono text-xs">getTotalValueLocked(lockCanisters)</td>
                        <td class="py-2 px-3 text-xs">Calculate total USD value</td>
                        <td class="py-2 px-3 text-xs">number (cents)</td>
                      </tr>
                      <tr class="border-b border-gray-800">
                        <td class="py-2 px-3 font-mono text-xs">getUserLPBreakdown(user, kongLocker)</td>
                        <td class="py-2 px-3 text-xs">Complete user analysis</td>
                        <td class="py-2 px-3 text-xs">UserLPBreakdown</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            {/if}
          </div>
        </div>
      
        <!-- Migration Guide (Collapsible) -->
        <div class="mb-6 bg-blue-900/20 border border-blue-800 rounded-lg">
          <div class="p-4 pb-3">
            <button 
              on:click={() => toggleExpanded('migration_guide')}
              class="w-full text-left flex items-center justify-between"
            >
              <h3 class="text-lg font-semibold text-blue-400">💡 Migration Guide</h3>
              <span class="text-blue-400">{expandedSections['migration_guide'] ? '▲' : '▼'}</span>
            </button>
            <p class="text-blue-200 text-sm mt-1">From old backend calls to new direct queries</p>
            {#if expandedSections['migration_guide']}
              <div class="mt-4 pt-3 border-t border-blue-800">
                <div class="bg-black/40 rounded p-3">
                  <pre class="text-sm text-gray-300 font-mono overflow-x-auto">{`// OLD: Backend inter-canister calls (removed)
const result = await kongLocker.get_voting_power(principal);
const allPowers = await kongLocker.get_all_voting_powers();
const totalLocked = await kongLocker.get_total_value_locked();

// NEW: Direct KongSwap queries
const kongSwap = new KongSwapDirectService();
const lockCanister = await kongLocker.get_my_lock_canister();
const positions = await kongSwap.getLPPositions(lockCanister[0]);
const votingPower = kongSwap.calculateVotingPower(positions);

// For all users
const lockCanisters = await kongLocker.get_all_lock_canisters();
const allPowers = await kongSwap.getAllVotingPowers(lockCanisters);
const totalLocked = await kongSwap.getTotalValueLocked(lockCanisters.map(c => c[1]));`}</pre>
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </section>

    <!-- Lock Canister Verification Section -->
    <section id="canister-verification" class="mb-12 bg-[#1a1b1f] rounded-2xl border border-gray-800">
      <!-- Sticky Section Header -->
      <div class="sticky top-0 z-10 bg-[#1a1b1f] rounded-t-2xl p-6 border-b border-gray-800">
        <h2 class="text-3xl font-bold text-white flex items-center gap-3">
          <span class="text-2xl">🔒</span>
          <span>Lock Canister Verification</span>
          <span class="text-sm font-normal text-gray-400 ml-auto">Verify authenticity before sending LP tokens</span>
        </h2>
      </div>
      <div class="p-8 pt-6">

        <!-- Critical Warning -->
        <div class="mb-6 bg-red-900/20 border border-red-600 rounded-lg p-4">
          <div class="flex items-start gap-3">
            <span class="text-2xl">⚠️</span>
            <div>
              <h3 class="text-lg font-bold text-red-400 mb-2">CRITICAL: Always Verify Before Sending Tokens</h3>
              <p class="text-red-200 text-sm">
                Never trust a canister claiming to be a Kong Locker without verification.
                Always check the module hash before sending LP tokens. Tokens sent to unverified canisters may be lost forever.
              </p>
            </div>
          </div>
        </div>

        <!-- Expected Module Hash -->
        <div class="mb-6 bg-green-900/20 border border-green-600 rounded-lg p-6">
          <h3 class="text-xl font-bold text-green-400 mb-3">✅ Expected Module Hash</h3>
          <p class="text-green-200 mb-3">All legitimate Kong Locker lock canisters have this exact module hash:</p>
          <div class="bg-black/50 rounded-lg p-4 font-mono text-sm text-green-300 break-all">
            0x8a4e009fbb6fa0aa5b92cc3fa623cc9d2fea1a0c7ca679a7d589fe1fa96b37f4
          </div>
        </div>

        <!-- Quick Verification -->
        <div class="mb-6 bg-gray-900/50 rounded-lg border border-gray-700 p-6">
          <h3 class="text-xl font-bold text-white mb-4">🔍 Quick Verification</h3>

          <div class="mb-4">
            <h4 class="text-base font-semibold text-gray-300 mb-2">Check any lock canister's module hash:</h4>
            <pre class="bg-black rounded-lg p-4 overflow-x-auto text-sm">
<code class="text-gray-300"><span class="text-gray-500"># Check your lock canister's module hash</span>
<span class="text-green-400">dfx</span> canister <span class="text-blue-400">--network</span> ic info <span class="text-yellow-400">YOUR-CANISTER-ID</span>

<span class="text-gray-500"># Expected output:</span>
Controllers:                    <span class="text-gray-500"># Should be empty (blackholed)</span>
Module hash: <span class="text-green-400">0x8a4e009f...</span>      <span class="text-gray-500"># Should match hash above</span></code></pre>
          </div>
        </div>

        <!-- Verify Against GitHub Source -->
        <div class="mb-6 bg-gray-900/50 rounded-lg border border-gray-700 p-6">
          <h3 class="text-xl font-bold text-white mb-4">🛠️ Verify Against GitHub Source</h3>
          <p class="text-gray-300 mb-4">Build from source to confirm the hash matches:</p>

          <pre class="bg-black rounded-lg p-4 overflow-x-auto text-sm mb-4">
<code class="text-gray-300"><span class="text-gray-500"># Clone and build from GitHub</span>
<span class="text-green-400">git</span> clone https://github.com/AlexandriaDAO/daopad.git
<span class="text-green-400">cd</span> daopad
<span class="text-green-400">cargo</span> build <span class="text-blue-400">--target</span> wasm32-unknown-unknown <span class="text-blue-400">--release</span> \
  <span class="text-blue-400">-p</span> lock_canister <span class="text-blue-400">--locked</span>

<span class="text-gray-500"># Check the SHA256 hash</span>
<span class="text-green-400">sha256sum</span> target/wasm32-unknown-unknown/release/lock_canister.wasm

<span class="text-gray-500"># Expected: 8a4e009fbb6fa0aa5b92cc3fa623cc9d2fea1a0c7ca679a7d589fe1fa96b37f4</span></code></pre>
        </div>

        <!-- Security Guarantees -->
        <div class="mb-6">
          <h3 class="text-xl font-bold text-white mb-4">🛡️ Security Guarantees</h3>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
            <div class="bg-blue-900/20 border border-blue-600 rounded-lg p-4 text-center">
              <div class="text-2xl mb-2">📖</div>
              <h4 class="font-bold text-blue-400 text-sm mb-1">Open Source</h4>
              <p class="text-xs text-gray-400">Runs exact GitHub code</p>
            </div>
            <div class="bg-purple-900/20 border border-purple-600 rounded-lg p-4 text-center">
              <div class="text-2xl mb-2">🔒</div>
              <h4 class="font-bold text-purple-400 text-sm mb-1">Immutable</h4>
              <p class="text-xs text-gray-400">Blackholed, no changes</p>
            </div>
            <div class="bg-red-900/20 border border-red-600 rounded-lg p-4 text-center">
              <div class="text-2xl mb-2">🚫</div>
              <h4 class="font-bold text-red-400 text-sm mb-1">No Backdoors</h4>
              <p class="text-xs text-gray-400">No unlock functions</p>
            </div>
            <div class="bg-yellow-900/20 border border-yellow-600 rounded-lg p-4 text-center">
              <div class="text-2xl mb-2">♾️</div>
              <h4 class="font-bold text-yellow-400 text-sm mb-1">Permanent</h4>
              <p class="text-xs text-gray-400">Tokens locked forever</p>
            </div>
            <div class="bg-green-900/20 border border-green-600 rounded-lg p-4 text-center">
              <div class="text-2xl mb-2">✅</div>
              <h4 class="font-bold text-green-400 text-sm mb-1">Verifiable</h4>
              <p class="text-xs text-gray-400">Hash proves integrity</p>
            </div>
          </div>
        </div>


        <!-- Red Flags -->
        <div class="bg-red-900/20 border border-red-600 rounded-lg p-6">
          <h3 class="text-xl font-bold text-red-400 mb-4">⚠️ Red Flags to Watch For</h3>
          <p class="text-red-200 mb-3">Never interact with a lock canister that:</p>
          <ul class="space-y-2 text-red-200">
            <li class="flex items-start gap-2">
              <span class="text-red-400">•</span>
              <span>Has a different module hash than expected</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="text-red-400">•</span>
              <span>Still has controllers (not blackholed)</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="text-red-400">•</span>
              <span>Claims to have unlock functionality</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="text-red-400">•</span>
              <span>Asks for permissions beyond receiving tokens</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="text-red-400">•</span>
              <span>Was not created through the official Kong Locker factory</span>
            </li>
          </ul>
        </div>
      </div>
    </section>

    <!-- LP Position Breakdown Demo Section -->
    <section id="lp-position-demo" class="mb-12 bg-[#1a1b1f] rounded-2xl border border-gray-800">
      <!-- Sticky Section Header -->
      <div class="sticky top-0 z-10 bg-[#1a1b1f] rounded-t-2xl p-6 border-b border-gray-800">
        <h2 class="text-3xl font-bold text-white flex items-center gap-3">
          <span class="text-2xl">🎮</span>
          <span>LP Position Breakdown Demo</span>
          <span class="text-sm font-normal text-gray-400 ml-auto">Interactive implementation guide</span>
        </h2>
      </div>
      <div class="p-8 pt-6">
      
        <!-- Implementation Guide Summary -->
        <div class="mb-6 bg-gradient-to-r from-green-900/20 to-blue-900/20 border border-green-800/50 rounded-lg">
          <div class="p-4 pb-3">
            <button 
              on:click={() => toggleExpanded('implementation_guide')}
              class="w-full text-left flex items-center justify-between"
            >
              <h3 class="text-lg font-semibold text-green-400">🎯 Implementation Guide</h3>
              <span class="text-green-400">{expandedSections['implementation_guide'] ? '▲' : '▼'}</span>
            </button>
            <p class="text-green-200 text-sm mt-1">4-step LP position breakdown implementation</p>
            {#if expandedSections['implementation_guide']}
              <div class="mt-4 pt-3 border-t border-green-800">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
                  <div class="bg-green-800/20 rounded p-3">
                    <div class="text-green-300 font-medium mb-1">Step 1: Get Lock Canister</div>
                    <div class="text-green-200 text-xs">Query Kong Locker backend for canister ID</div>
                  </div>
                  <div class="bg-blue-800/20 rounded p-3">
                    <div class="text-blue-300 font-medium mb-1">Step 2: Query KongSwap</div>
                    <div class="text-blue-200 text-xs">Get detailed LP positions directly</div>
                  </div>
                  <div class="bg-purple-800/20 rounded p-3">
                    <div class="text-purple-300 font-medium mb-1">Step 3: Format Data</div>
                    <div class="text-purple-200 text-xs">Display individual position details</div>
                  </div>
                  <div class="bg-orange-800/20 rounded p-3">
                    <div class="text-orange-300 font-medium mb-1">Step 4: Calculate Totals</div>
                    <div class="text-orange-200 text-xs">Voting power and USD values</div>
                  </div>
                </div>
              </div>
            {/if}
          </div>
        </div>
      
        <!-- Test LP Position Query -->
        <div class="mb-6 bg-gray-900/50 rounded-lg border border-gray-700">
          <div class="p-4">
            <h3 class="text-lg font-bold text-white mb-2">Test LP Position Query</h3>
            <p class="text-gray-400 text-sm mb-4">
              Enter a lock canister Principal ID to see detailed LP positions via KongSwap's <code class="bg-black/50 px-1 py-0.5 rounded text-xs">user_balances</code>
            </p>
            
            <!-- Implementation Pattern (Collapsible) -->
            <div class="mb-4">
              <button 
                on:click={() => toggleExpanded('demo_implementation_pattern')}
                class="text-sm text-blue-400 hover:text-blue-300 mb-2"
              >
                {expandedSections['demo_implementation_pattern'] ? '▲ Hide' : '▼ Show'} Implementation Pattern
              </button>
              {#if expandedSections['demo_implementation_pattern']}
                <div class="bg-blue-900/20 border border-blue-800 rounded p-3">
                  <div class="bg-black/40 rounded p-3">
                    <pre class="text-xs text-blue-400 font-mono overflow-x-auto">{`// Step 1: Get user's lock canister from Kong Locker
const lockCanister = await kongLocker.get_my_lock_canister();

// Step 2: Query KongSwap directly for detailed positions  
const result = await kongSwap.user_balances(lockCanister[0].toText());

// Step 3: Extract LP position details
const positions = result.Ok.map(reply => reply.LP);

// Step 4: Display each position with full details
positions.forEach(pos => {
  console.log(\`Pool: \${pos.name}\`);
  console.log(\`LP Tokens: \${pos.balance}\`);
  console.log(\`USD Value: $\${pos.usd_balance}\`);
  console.log(\`Tokens: \${pos.amount_0} \${pos.symbol_0} + \${pos.amount_1} \${pos.symbol_1}\`);
});`}</pre>
                  </div>
                </div>
              {/if}
            </div>
        
        <!-- Interactive Test -->
        <div class="bg-purple-900/20 border border-purple-800 rounded-lg p-4 mb-4">
          <div class="mb-3">
            <h4 class="text-sm font-semibold text-purple-400 mb-2">🎮 Try it now!</h4>
            <div class="flex gap-2">
              <input
                type="text"
                bind:value={lockCanisterInput}
                placeholder="Enter lock canister Principal ID (e.g., abc12-def34...)"
                class="flex-1 px-3 py-2 bg-black/50 border border-gray-700 rounded-lg text-white placeholder-gray-500 text-sm focus:outline-none focus:border-purple-500"
              />
              <button 
                on:click={getLPPositions}
                disabled={lpPositionsLoading}
                class="px-4 py-2 bg-purple-600 hover:bg-purple-500 disabled:bg-gray-600 disabled:cursor-not-allowed text-white text-sm rounded-lg transition-colors flex items-center gap-2"
              >
                {#if lpPositionsLoading}
                  <Loader class="w-4 h-4 animate-spin" />
                  <span>Querying...</span>
                {:else}
                  <span>Get Positions</span>
                {/if}
              </button>
            </div>
          </div>
          
          {#if lpPositionsError}
            <div class="bg-red-900/20 rounded p-3 mt-3">
              <p class="text-xs text-red-400">Error: {lpPositionsError}</p>
            </div>
          {/if}
          
          {#if lpPositions.length > 0}
            <div class="mt-4 space-y-4">
              <!-- Summary Stats -->
              <div class="bg-green-900/20 border border-green-800 rounded-lg p-4">
                <h4 class="text-sm font-semibold text-green-400 mb-2">📊 Summary</h4>
                <div class="grid grid-cols-3 gap-4 text-sm">
                  <div class="text-center">
                    <div class="text-lg font-bold text-white">{lpPositions.length}</div>
                    <div class="text-xs text-gray-400">LP Pools</div>
                  </div>
                  <div class="text-center">
                    <div class="text-lg font-bold text-green-400">${calculateTotalUSDValue(lpPositions).toFixed(2)}</div>
                    <div class="text-xs text-gray-400">Total Value</div>
                  </div>
                  <div class="text-center">
                    <div class="text-lg font-bold text-blue-400">{calculateVotingPower(lpPositions).toLocaleString()}</div>
                    <div class="text-xs text-gray-400">Voting Power</div>
                  </div>
                </div>
              </div>
              
              <!-- Individual Positions -->
              <div class="space-y-3">
                <h4 class="text-sm font-semibold text-white">🔒 Individual LP Positions</h4>
                {#each lpPositions as position, index}
                  <div class="bg-black/30 border border-gray-700 rounded-lg p-4">
                    <div class="flex justify-between items-start mb-3">
                      <div>
                        <h5 class="font-semibold text-white">{position.name}</h5>
                        <p class="text-sm text-gray-400">Pool: {position.symbol}</p>
                      </div>
                      <div class="text-right">
                        <div class="text-lg font-bold text-green-400">${position.usd_balance.toFixed(2)}</div>
                        <div class="text-xs text-gray-400">{((position.usd_balance / calculateTotalUSDValue(lpPositions)) * 100).toFixed(1)}% of total</div>
                      </div>
                    </div>
                    
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3 text-sm">
                      <div class="bg-gray-800/50 rounded p-3">
                        <div class="text-xs text-gray-400 mb-1">LP Tokens</div>
                        <div class="font-semibold text-white">{formatTokenAmount(position.balance, 'LP')}</div>
                      </div>
                      <div class="bg-gray-800/50 rounded p-3">
                        <div class="text-xs text-gray-400 mb-1">{position.symbol_0}</div>
                        <div class="font-semibold text-blue-400">{formatTokenAmount(position.amount_0, position.symbol_0)}</div>
                        <div class="text-xs text-gray-500">${position.usd_amount_0.toFixed(2)}</div>
                      </div>
                      <div class="bg-gray-800/50 rounded p-3">
                        <div class="text-xs text-gray-400 mb-1">{position.symbol_1}</div>
                        <div class="font-semibold text-orange-400">{formatTokenAmount(position.amount_1, position.symbol_1)}</div>
                        <div class="text-xs text-gray-500">${position.usd_amount_1.toFixed(2)}</div>
                      </div>
                    </div>
                    
                    <div class="mt-3 pt-3 border-t border-gray-700">
                      <div class="text-xs text-gray-400">
                        Token ID: {position.lp_token_id.toString()} • 
                        Last Updated: {new Date(Number(position.ts)).toLocaleDateString()}
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
              
              <!-- Implementation Code -->
              <div class="bg-yellow-900/20 border border-yellow-800 rounded-lg p-4">
                <h4 class="text-sm font-semibold text-yellow-400 mb-2">📋 Implementation Code</h4>
                <div class="bg-black/50 rounded-lg p-3">
                  <pre class="text-sm text-gray-300 font-mono overflow-x-auto">{`// This is what your app would do:

// 1. Query KongSwap for this lock canister
const result = await kongSwap.user_balances("${lockCanisterInput}");

// 2. Extract positions 
const positions = result.Ok.map(reply => reply.LP);
// Found ${lpPositions.length} LP position${lpPositions.length === 1 ? '' : 's'}

// 3. Calculate totals
const totalUSD = positions.reduce((sum, pos) => sum + pos.usd_balance, 0);
// Total: $${calculateTotalUSDValue(lpPositions).toFixed(2)}

const votingPower = Math.floor(totalUSD * 100);
// Voting Power: ${calculateVotingPower(lpPositions).toLocaleString()}

// 4. Display each position
positions.forEach((pos, i) => {
  console.log(\`Position \${i + 1}: \${pos.name}\`);
  console.log(\`  LP Tokens: \${pos.balance}\`);
  console.log(\`  Value: $\${pos.usd_balance}\`);
  console.log(\`  Composition: \${pos.amount_0} \${pos.symbol_0} + \${pos.amount_1} \${pos.symbol_1}\`);
});`}</pre>
                </div>
              </div>
            </div>
          {/if}
        </div>
      </div>
      
      <div class="bg-orange-900/20 border border-orange-800 rounded-lg p-4">
        <h3 class="text-lg font-semibold text-orange-400 mb-2">⚠️ Important Notes for Implementation</h3>
        <ul class="list-disc list-inside text-orange-200 text-sm space-y-1">
          <li><strong>Error Handling:</strong> Always check for "User not found" errors - lock canisters need to be registered on KongSwap first</li>
          <li><strong>Data Freshness:</strong> LP values and token amounts reflect real-time market prices from KongSwap</li>
          <li><strong>Voting Power:</strong> Calculated as total USD value × 100 (to preserve 2 decimal places as integer)</li>
          <li><strong>Performance:</strong> This is a direct query call - no cycles required, fast response times</li>
          <li><strong>Integration:</strong> Combine with Kong Locker's <code>get_my_lock_canister()</code> for complete user experience</li>
        </ul>
      </div>
        </div>
      </div>
    </section>

    <!-- Data Types Section -->
    <section id="types" class="mb-12 bg-[#1a1b1f] rounded-2xl p-8 border border-gray-800">
      <h2 class="text-3xl font-bold text-white mb-6">📊 Data Types</h2>
      
      <div class="mb-8 bg-gray-900/50 rounded-lg p-6">
        <h3 class="text-xl font-bold text-white mb-3">LPReply</h3>
        <p class="text-gray-400 mb-4">Represents LP token position data from KongSwap (used in direct queries).</p>
        
        <div class="bg-black/50 rounded-lg p-4 mb-4">
          <pre class="text-sm text-green-400 font-mono overflow-x-auto">{`type LPReply = {
  name: string;           // "ICP/ckUSDT LP Token"
  symbol: string;         // "ICP_ckUSDT"
  lp_token_id: bigint;    // LP token ID
  balance: number;        // LP token balance
  usd_balance: number;    // Total USD value
  
  // First token details
  chain_0: string;        // "IC"
  symbol_0: string;       // "ICP"
  address_0: string;      // "ryjl3-tyaaa-aaaaa-aaaba-cai"
  amount_0: number;       // 50.25
  usd_amount_0: number;   // 1250.50
  
  // Second token details
  chain_1: string;        // "IC"
  symbol_1: string;       // "ckUSDT"
  address_1: string;      // Token canister
  amount_1: number;       // Amount of second token
  usd_amount_1: number;   // USD value of second token
  
  ts: bigint;             // Timestamp
}`}</pre>
        </div>
      </div>
      
      <div class="mb-8 bg-gray-900/50 rounded-lg p-6">
        <h3 class="text-xl font-bold text-white mb-3">UserBalancesReply</h3>
        <p class="text-gray-400 mb-4">Enum wrapper for balance responses from KongSwap.</p>
        
        <div class="bg-black/50 rounded-lg p-4">
          <pre class="text-sm text-green-400 font-mono">{`type UserBalancesReply = variant {
  LP: LPReply;  // Currently only LP tokens are supported
}`}</pre>
        </div>
      </div>
      
      <div class="mb-8 bg-gray-900/50 rounded-lg p-6">
        <h3 class="text-xl font-bold text-white mb-3">DetailedCanisterStatus</h3>
        <p class="text-gray-400 mb-4">Comprehensive status information about a lock canister.</p>
        
        <div class="bg-black/50 rounded-lg p-4">
          <pre class="text-sm text-green-400 font-mono overflow-x-auto">{`type DetailedCanisterStatus = {
  canister_id: principal;     // The lock canister's ID
  is_blackholed: bool;        // true if no controllers
  controller_count: nat32;    // Should be 0 when blackholed
  cycle_balance: nat;         // Remaining cycles
  memory_size: nat;           // Memory usage in bytes
  module_hash: opt blob;      // WASM module hash
}`}</pre>
        </div>
      </div>
      
      <div class="mb-8 bg-gray-900/50 rounded-lg p-6">
        <h3 class="text-xl font-bold text-white mb-3">AnalyticsOverview</h3>
        <p class="text-gray-400 mb-4">Complete analytics data for building dashboards and exploring the platform.</p>
        
        <div class="bg-black/50 rounded-lg p-4">
          <pre class="text-sm text-green-400 font-mono overflow-x-auto">{`type AnalyticsOverview = {
  total_lock_canisters: nat64;                    // Total number of canisters
  participants: vec record { principal; principal }; // (user, lock_canister) pairs
  last_updated: nat64;                            // Timestamp in nanoseconds
}`}</pre>
        </div>
        
        <div class="bg-blue-900/20 border border-blue-800 rounded-lg p-3 mt-4">
          <h4 class="text-sm font-semibold text-blue-400 mb-2">💡 Analytics Use Cases</h4>
          <ul class="list-disc list-inside text-blue-200 text-sm space-y-1">
            <li><strong>Dashboard foundation:</strong> Get all participants with a single fast query</li>
            <li><strong>Scalable design:</strong> Efficient even with 100+ users</li>
            <li><strong>Progressive disclosure:</strong> Show list immediately, load details on-demand</li>
            <li><strong>Platform insights:</strong> Track growth and participation trends</li>
          </ul>
        </div>
      </div>
    </section>

    <!-- Integration Examples Section -->
    <section id="examples" class="mb-12 bg-[#1a1b1f] rounded-2xl p-8 border border-gray-800">
      <h2 class="text-3xl font-bold text-white mb-6">💡 Integration Examples</h2>
      
      <div class="mb-8 bg-gray-900/50 rounded-lg p-6">
        <h3 class="text-xl font-bold text-white mb-3">Complete Analytics Dashboard Implementation</h3>
        <p class="text-gray-400 mb-4">Build a scalable analytics dashboard that efficiently handles 100+ users.</p>
        
        <div class="bg-black/50 rounded-lg p-4">
          <pre class="text-sm text-yellow-400 font-mono overflow-x-auto">{`import { Principal } from '@dfinity/principal';
import { createActor } from './actor';

// Complete analytics dashboard implementation
async function buildAnalyticsDashboard() {
  const kongLocker = await createActor(KONG_LOCKER_ID);
  const kongSwap = await createActor(KONG_SWAP_ID);
  
  // Step 1: Get complete overview with single fast query (≤500ms)
  const overview = await kongLocker.get_analytics_overview();
  console.log(\`Found \${overview.total_lock_canisters} lock canisters\`);
  
  // Step 2: Display participants list immediately
  const participantList = overview.participants.map(([user, lockCanister]) => ({
    userPrincipal: user,
    lockCanister: lockCanister,
    userDisplay: truncatePrincipal(user.toText()),
    canisterDisplay: truncatePrincipal(lockCanister.toText()),
    status: 'unknown' // Will be updated on-demand
  }));
  
  // Step 3: Calculate basic system stats
  const systemStats = {
    totalParticipants: overview.participants.length,
    lastUpdated: new Date(Number(overview.last_updated) / 1_000_000)
  };
  
  // Step 4: Optional - Preload top 10 canisters for better UX
  const topCanisters = overview.participants.slice(0, 10);
  const preloadPromises = topCanisters.map(async ([user, canister]) => {
    try {
      const result = await kongSwap.user_balances(canister.toText());
      if ('Ok' in result) {
        const positions = result.Ok.map(reply => reply.LP);
        const totalUSD = positions.reduce((sum, pos) => sum + pos.usd_balance, 0);
        return { canister, totalUSD, positions };
      }
    } catch (error) {
      console.debug('Preload failed for:', canister.toText());
      return null;
    }
  });
  
  // Don't await preload - let it happen in background
  Promise.allSettled(preloadPromises).then(results => {
    console.log('Preloaded', results.filter(r => r.status === 'fulfilled').length, 'canisters');
  });
  
  return { participantList, systemStats, overview };
}

// On-demand detail loading for specific canister
async function loadCanisterDetails(lockCanisterId: Principal) {
  const kongSwap = await createActor(KONG_SWAP_ID);
  
  try {
    // Query KongSwap for this specific canister (≤2s)
    const result = await kongSwap.user_balances(lockCanisterId.toText());
    
    if ('Err' in result) {
      if (result.Err.includes("User not found")) {
        return {
          status: 'unregistered',
          message: 'Lock canister not registered on KongSwap yet'
        };
      }
      throw new Error(result.Err);
    }
    
    const positions = result.Ok.map(reply => reply.LP);
    
    if (positions.length === 0) {
      return {
        status: 'empty',
        message: 'No LP tokens locked yet'
      };
    }
    
    // Calculate metrics
    const totalUSD = positions.reduce((sum, pos) => sum + pos.usd_balance, 0);
    const votingPower = Math.floor(totalUSD * 100);
    
    return {
      status: 'active',
      positions,
      summary: {
        poolCount: positions.length,
        totalUSD,
        votingPower
      }
    };
    
  } catch (error) {
    return {
      status: 'error',
      message: error.message
    };
  }
}

function truncatePrincipal(principal: string): string {
  if (principal.length <= 20) return principal;
  return \`\${principal.substring(0, 8)}...\${principal.substring(principal.length - 6)}\`;
}`}</pre>
        </div>
      </div>
      
      <div class="mb-8 bg-gray-900/50 rounded-lg p-6">
        <h3 class="text-xl font-bold text-white mb-3">LP Position Breakdown for User Dashboard</h3>
        <p class="text-gray-400 mb-4">Complete example showing how to display detailed LP positions in your application.</p>
        
        <div class="bg-black/50 rounded-lg p-4">
          <pre class="text-sm text-yellow-400 font-mono overflow-x-auto">{`import { Principal } from '@dfinity/principal';
import { createActor } from './actor';

// Complete LP position dashboard implementation
async function displayUserLPDashboard(userPrincipal: Principal) {
  const kongLocker = await createActor(KONG_LOCKER_ID);
  const kongSwap = await createActor(KONG_SWAP_ID);
  
  try {
    // Step 1: Get user's lock canister from Kong Locker
    const lockCanisterResult = await kongLocker.get_my_lock_canister();
    
    if (!lockCanisterResult[0]) {
      return { hasLockCanister: false, message: "No lock canister found" };
    }
    
    const lockCanisterId = lockCanisterResult[0];
    console.log("Lock canister:", lockCanisterId.toText());
    
    // Step 2: Query KongSwap directly for detailed LP positions
    const balancesResult = await kongSwap.user_balances(lockCanisterId.toText());
    
    if ('Err' in balancesResult) {
      if (balancesResult.Err.includes("User not found")) {
        return { 
          hasLockCanister: true, 
          isRegistered: false,
          message: "Lock canister not registered on KongSwap yet" 
        };
      }
      throw new Error(balancesResult.Err);
    }
    
    // Step 3: Extract and process LP positions
    const lpPositions = balancesResult.Ok.map(reply => reply.LP);
    
    if (lpPositions.length === 0) {
      return {
        hasLockCanister: true,
        isRegistered: true,
        hasPositions: false,
        message: "No LP tokens locked yet"
      };
    }
    
    // Step 4: Calculate totals and format data
    const totalUSDValue = lpPositions.reduce((sum, pos) => sum + pos.usd_balance, 0);
    const votingPower = Math.floor(totalUSDValue * 100);
    
    // Step 5: Format positions for display
    const formattedPositions = lpPositions.map((position, index) => ({
      id: position.lp_token_id.toString(),
      poolName: position.name,
      poolSymbol: position.symbol,
      lpTokenBalance: position.balance,
      usdValue: position.usd_balance,
      percentOfTotal: (position.usd_balance / totalUSDValue) * 100,
      underlyingTokens: [
        {
          symbol: position.symbol_0,
          amount: position.amount_0,
          usdValue: position.usd_amount_0
        },
        {
          symbol: position.symbol_1, 
          amount: position.amount_1,
          usdValue: position.usd_amount_1
        }
      ],
      lastUpdated: new Date(Number(position.ts))
    }));
    
    return {
      hasLockCanister: true,
      isRegistered: true,
      hasPositions: true,
      lockCanisterId: lockCanisterId.toText(),
      summary: {
        totalPools: lpPositions.length,
        totalUSDValue,
        votingPower
      },
      positions: formattedPositions
    };
    
  } catch (error) {
    console.error('Failed to load LP dashboard:', error);
    return { error: error.message };
  }
}

// Usage in your React/Svelte/Vue component:
async function loadDashboard() {
  const dashboard = await displayUserLPDashboard(userPrincipal);
  
  if (dashboard.error) {
    setError(dashboard.error);
    return;
  }
  
  if (!dashboard.hasLockCanister) {
    showCreateCanisterButton();
    return;
  }
  
  if (!dashboard.isRegistered) {
    showSetupRequiredMessage();
    return;
  }
  
  if (!dashboard.hasPositions) {
    showAddLiquidityPrompt();
    return;
  }
  
  // Display the complete dashboard
  setDashboardData(dashboard);
}`}</pre>
        </div>
      </div>
      
      <div class="mb-8 bg-gray-900/50 rounded-lg p-6">
        <h3 class="text-xl font-bold text-white mb-3">Complete Lock Flow</h3>
        <p class="text-gray-400 mb-4">Full example of creating a lock and transferring LP tokens.</p>
        
        <div class="bg-black/50 rounded-lg p-4">
          <pre class="text-sm text-yellow-400 font-mono overflow-x-auto">{`import { Principal } from '@dfinity/principal';
import { createActor } from './actor';
import { KongSwapDirectService } from './services/kongSwapDirect';

async function lockLPTokens(amount: bigint) {
  const kongLocker = await createActor(KONG_LOCKER_ID);
  const icpLedger = await createActor(ICP_LEDGER_ID);
  const kongSwap = new KongSwapDirectService();
  
  // Step 1: Check if user already has a lock canister
  const existingCanister = await kongLocker.get_my_lock_canister();
  if (existingCanister.length > 0) {
    console.log("Using existing canister:", existingCanister[0].toText());
    const lockCanisterId = existingCanister[0];
    
    // Transfer LP tokens to existing canister
    await transferLPTokens(lockCanisterId, amount);
    return lockCanisterId;
  }
  
  // Step 2: Approve 2 ICP for new canister creation
  console.log("Approving 2 ICP...");
  await icpLedger.icrc2_approve({
    spender: { owner: Principal.fromText(KONG_LOCKER_ID), subaccount: [] },
    amount: 200_010_000n, // 2 ICP + fees
    fee: [10_000n],
    expires_at: [],
    memo: [],
    created_at_time: []
  });
  
  // Step 3: Create lock canister
  console.log("Creating lock canister...");
  const createResult = await kongLocker.create_lock_canister();
  
  if ('Err' in createResult) {
    throw new Error(createResult.Err);
  }
  
  const lockCanisterId = createResult.Ok;
  console.log("Lock canister created:", lockCanisterId.toText());
  
  // Step 4: Transfer LP tokens to lock canister
  await transferLPTokens(lockCanisterId, amount);
  
  // Step 5: Verify lock using direct KongSwap query
  const positions = await kongSwap.getLPPositions(lockCanisterId);
  const votingPower = kongSwap.calculateVotingPower(positions);
  
  const usdValue = votingPower / 100;
  console.log(\`Locked value: $\${usdValue.toFixed(2)}\`);
  
  return lockCanisterId;
}`}</pre>
        </div>
      </div>
      
      <div class="mb-8 bg-gray-900/50 rounded-lg p-6">
        <h3 class="text-xl font-bold text-white mb-3">Voting Power Dashboard</h3>
        <p class="text-gray-400 mb-4">Example of displaying all users' voting powers using direct KongSwap queries.</p>
        
        <div class="bg-black/50 rounded-lg p-4">
          <pre class="text-sm text-yellow-400 font-mono overflow-x-auto">{`import { KongSwapDirectService } from './services/kongSwapDirect';

async function displayVotingPowerLeaderboard() {
  const kongLocker = await createActor(KONG_LOCKER_ID);
  const kongSwap = new KongSwapDirectService();
  
  // Get all lock canisters
  const lockCanisters = await kongLocker.get_all_lock_canisters();
  
  // Get voting powers using direct KongSwap queries
  const allPowers = await kongSwap.getAllVotingPowers(lockCanisters);
  
  // Sort by voting power (descending)
  const sorted = allPowers.sort((a, b) => {
    const powerA = Number(a[1]);
    const powerB = Number(b[1]);
    return powerB - powerA;
  });
  
  // Display leaderboard
  console.log("🏆 Voting Power Leaderboard");
  console.log("===========================");
  
  sorted.slice(0, 10).forEach((entry, index) => {
    const [principal, power] = entry;
    const usdValue = Number(power) / 100;
    
    console.log(
      \`\${index + 1}. \${principal.toText().slice(0, 10)}... - $\${usdValue.toLocaleString()}\`
    );
  });
  
  // Calculate total
  const totalCents = allPowers.reduce((sum, entry) => {
    return sum + Number(entry[1]);
  }, 0);
  
  const totalUSD = totalCents / 100;
  console.log(\`\\nTotal Value Locked: $\${totalUSD.toLocaleString()}\`);
  console.log(\`Total Participants: \${allPowers.length}\`);
}`}</pre>
        </div>
      </div>
    </section>

    <!-- Error Handling Section -->
    <section id="errors" class="mb-12 bg-[#1a1b1f] rounded-2xl p-8 border border-gray-800">
      <h2 class="text-3xl font-bold text-white mb-6">⚠️ Error Handling</h2>
      
      <div class="mb-6">
        <h3 class="text-xl font-bold text-white mb-4">Common Errors</h3>
        
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-gray-700">
                <th class="text-left py-3 px-4 text-gray-400">Error Message</th>
                <th class="text-left py-3 px-4 text-gray-400">Cause</th>
                <th class="text-left py-3 px-4 text-gray-400">Solution</th>
              </tr>
            </thead>
            <tbody class="text-gray-300">
              <tr class="border-b border-gray-800">
                <td class="py-3 px-4 font-mono text-red-400">"You already have a lock canister"</td>
                <td class="py-3 px-4">Attempting to create second canister</td>
                <td class="py-3 px-4">Use existing canister via get_my_lock_canister()</td>
              </tr>
              <tr class="border-b border-gray-800">
                <td class="py-3 px-4 font-mono text-red-400">"Payment failed: InsufficientAllowance"</td>
                <td class="py-3 px-4">ICP not approved</td>
                <td class="py-3 px-4">Approve 2 ICP before calling create_lock_canister</td>
              </tr>
              <tr class="border-b border-gray-800">
                <td class="py-3 px-4 font-mono text-red-400">"No lock canister found"</td>
                <td class="py-3 px-4">User hasn't created canister</td>
                <td class="py-3 px-4">Call create_lock_canister() first</td>
              </tr>
              <tr class="border-b border-gray-800">
                <td class="py-3 px-4 font-mono text-red-400">"User not found" (KongSwap)</td>
                <td class="py-3 px-4">Not registered on KongSwap</td>
                <td class="py-3 px-4">Call complete_my_canister_setup() or fund canister</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
      
      <div class="bg-gray-900/50 rounded-lg p-6">
        <h3 class="text-xl font-bold text-white mb-3">Error Handling Best Practices</h3>
        
        <div class="bg-black/50 rounded-lg p-4">
          <pre class="text-sm text-yellow-400 font-mono overflow-x-auto">{`// Always wrap calls in try-catch and handle Result variants
async function safeGetVotingPower(lockCanisterPrincipal: Principal) {
  try {
    const kongSwap = new KongSwapDirectService();
    const positions = await kongSwap.getLPPositions(lockCanisterPrincipal);
    const votingPower = kongSwap.calculateVotingPower(positions);
    
    return { success: true, value: votingPower };
  } catch (error: any) {
    // Handle specific errors
    if (error.message.includes("User not found")) {
      return { success: false, error: "NOT_REGISTERED", message: error.message };
    } else {
      return { success: false, error: "UNKNOWN", message: error.message };
    }
  }
}

// Kong Locker backend errors
async function safeGetCanisterStatus() {
  try {
    const result = await kongLocker.get_detailed_canister_status();
    
    if ('Ok' in result) {
      return { success: true, data: result.Ok };
    } else {
      return { success: false, error: result.Err };
    }
  } catch (error: any) {
    return { success: false, error: error.message };
  }
}`}</pre>
        </div>
      </div>
    </section>
    {/if}

    <!-- Footer -->
    <div class="bg-gradient-to-r from-gray-800 to-gray-900 rounded-2xl p-6 text-center">
      <p class="text-gray-400 mb-2">Kong Locker Interactive API Documentation v2.1</p>
      <p class="text-sm text-gray-500">
        Kong Locker: <code class="bg-black/30 px-2 py-1 rounded">eazgb-giaaa-aaaap-qqc2q-cai</code>
      </p>
      <p class="text-sm text-gray-500">
        KongSwap: <code class="bg-black/30 px-2 py-1 rounded">2ipq2-uqaaa-aaaar-qailq-cai</code>
      </p>
      <p class="text-sm text-gray-500 mt-2">
        Need help? Visit <a href="https://kongswap.io" class="text-blue-400 hover:text-blue-300">KongSwap.io</a>
      </p>
    </div>
  </main>
</div>

<style>
  /* Custom scrollbar for code blocks */
  pre::-webkit-scrollbar {
    height: 6px;
  }
  
  pre::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.1);
    border-radius: 3px;
  }
  
  pre::-webkit-scrollbar-thumb {
    background: rgba(100, 100, 100, 0.5);
    border-radius: 3px;
  }
  
  pre::-webkit-scrollbar-thumb:hover {
    background: rgba(100, 100, 100, 0.7);
  }
  
  /* Smooth transitions */
  * {
    scroll-behavior: smooth;
  }
</style>