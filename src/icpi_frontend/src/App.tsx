import { useState, useEffect } from 'react';
import { AuthClient } from '@dfinity/auth-client';
import { HttpAgent, Actor, Identity } from '@dfinity/agent';
import { QueryClient, QueryClientProvider, useQueryClient } from '@tanstack/react-query';

import { idlFactory as icpiIdlFactory } from 'declarations/icpi_backend/icpi_backend.did.js';
import { canisterId as icpiCanisterId } from 'declarations/icpi_backend';

import { Button } from './components/ui/button';
import { Dashboard } from './components/Dashboard';
import { Documentation } from './components/Documentation';
import { FullPageSkeleton } from './components/LoadingStates';
import { ErrorBoundary } from './components/ErrorBoundary';
import {
  useIndexState,
  useRebalancerStatus,
  useTradeHistory,
  useTVLData,
  useHoldings,
  useAllocation,
  useActualAllocations,
  useMintICPI,
  useRedeemICPI,
  useManualRebalance,
  useUserWalletBalances,
  useTransferToken,
  useTotalSupply,
  UserTokenBalance,
  QUERY_KEYS
} from './hooks/useICPI';
import { SendTokenModal } from './components/SendTokenModal';

// Create a client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      gcTime: 5 * 60_000,
      refetchOnWindowFocus: false,
      retry: 3,
    },
  },
});

// Helper function to create actor with authentication (P1: Extract to reduce duplication)
async function createActorWithAuth(identity: Identity): Promise<{ agent: HttpAgent; actor: Actor }> {
  const isLocal = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';
  const host = isLocal ? 'http://localhost:4943' : 'https://icp-api.io';

  const agent = new HttpAgent({
    identity,
    host,
    ingressExpiryMs: 5 * 60 * 1000, // 5 minutes for long-running update calls
  });

  // P1: Make fetchRootKey await consistent
  if (isLocal) {
    await agent.fetchRootKey().catch(console.error);
  }

  const actor = Actor.createActor(icpiIdlFactory, {
    agent,
    canisterId: icpiCanisterId,
  });

  return { agent, actor };
}

function AppContent() {
  const [authClient, setAuthClient] = useState<AuthClient | null>(null);
  const [identity, setIdentity] = useState<Identity | null>(null);
  const [principal, setPrincipal] = useState<string>('');
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [actor, setActor] = useState<Actor | null>(null);
  const [agent, setAgent] = useState<HttpAgent | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);

  // Performance timing - mark page load start
  useEffect(() => {
    performance.mark('app-start');
  }, []);

  // Atomic initialization - create actor in single useEffect to avoid race condition
  useEffect(() => {
    async function initialize() {
      try {
        const client = await AuthClient.create({
          idleOptions: {
            idleTimeout: 1000 * 60 * 60 * 24 * 7, // 7 days
            disableDefaultIdleCallback: true,
            disableIdle: false,
          }
        });

        setAuthClient(client);
        const isAuth = await client.isAuthenticated();

        if (isAuth) {
          const identity = client.getIdentity();
          const newPrincipal = identity.getPrincipal().toString();

          // Create actor atomically with auth state using helper function
          const { agent: newAgent, actor: newActor } = await createActorWithAuth(identity);

          // Set all auth-related state atomically
          setIdentity(identity);
          setPrincipal(newPrincipal);
          setAgent(newAgent);
          setActor(newActor);
          setIsAuthenticated(true);
        }

        // Mark as initialized regardless of auth status
        setIsInitialized(true);
      } catch (error) {
        console.warn('Failed to initialize:', error);
        setIsInitialized(true); // Still mark as initialized to show error state
      }
    }

    initialize();
  }, []);

  const login = async () => {
    if (!authClient) return;

    const isLocal = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';
    const identityProvider = isLocal ? 'http://localhost:4943' : 'https://identity.ic0.app';
    const weekInNanoSeconds = BigInt(7 * 24 * 60 * 60 * 1000 * 1000 * 1000);

    await authClient.login({
      identityProvider,
      maxTimeToLive: weekInNanoSeconds,
      onSuccess: async () => {
        const identity = authClient.getIdentity();
        const newPrincipal = identity.getPrincipal().toString();

        // Create actor atomically on login using helper function
        const { agent: newAgent, actor: newActor } = await createActorWithAuth(identity);

        setIdentity(identity);
        setPrincipal(newPrincipal);
        setAgent(newAgent);
        setActor(newActor);
        setIsAuthenticated(true);
      },
    });
  };

  const logout = async () => {
    if (!authClient) return;
    await authClient.logout();
    setIdentity(null);
    setPrincipal('');
    setIsAuthenticated(false);
    setActor(null);
    setAgent(null);
  };

  // Don't render anything until initialization is complete
  if (!isInitialized) {
    return <FullPageSkeleton />;
  }

  if (!isAuthenticated) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-[#000000]">
        <div className="w-full max-w-sm border border-[#1f1f1f] bg-[#0a0a0a] p-6">
          <div className="space-y-4">
            <h1 className="text-xl font-mono text-white tracking-wider">ICPI</h1>
            <p className="text-xs font-mono text-[#999999]">
              Internet Computer Portfolio Index
            </p>
            <Button
              onClick={login}
              className="w-full"
              size="sm"
              variant="primary"
            >
              CONNECT WALLET
            </Button>
          </div>
        </div>
      </div>
    );
  }

  // Don't render dashboard hooks until actor is ready
  if (!actor || !agent) {
    return <FullPageSkeleton />;
  }

  // Render dashboard with guaranteed non-null actor and agent
  return (
    <DashboardContent
      actor={actor}
      agent={agent}
      principal={principal}
      logout={logout}
    />
  );
}

// Separate component that only renders when actor is guaranteed to be ready
// This prevents the race condition where hooks fire with null actor
function DashboardContent({
  actor,
  agent,
  principal,
  logout,
}: {
  actor: Actor;
  agent: HttpAgent;
  principal: string;
  logout: () => Promise<void>;
}) {
  const queryClient = useQueryClient();
  const [autoRebalance, setAutoRebalance] = useState(true);
  const [sendModalToken, setSendModalToken] = useState<UserTokenBalance | null>(null);
  const [currentView, setCurrentView] = useState<'dashboard' | 'docs'>('dashboard');

  // Now hooks fire ONCE with valid actor, not multiple times with null
  const { data: indexState, isLoading: indexLoading } = useIndexState(actor);
  const { data: rebalancerStatus } = useRebalancerStatus(actor);
  const { data: tradeHistory } = useTradeHistory(actor);
  const { data: tvlData } = useTVLData(actor);
  const { data: holdings } = useHoldings(actor);
  const { data: allocations } = useAllocation(actor);
  const { data: actualAllocations } = useActualAllocations(actor, icpiCanisterId, agent);
  const { data: totalSupply } = useTotalSupply(actor);

  const mintMutation = useMintICPI(actor, agent);
  const redeemMutation = useRedeemICPI(actor, agent);
  const rebalanceMutation = useManualRebalance(actor);

  // Wallet balance hooks
  const { data: walletBalances, isLoading: balancesLoading } = useUserWalletBalances(
    actor,
    principal,
    agent
  );
  const transferMutation = useTransferToken(agent);

  // Show skeleton while initial query is loading
  if (!indexState || indexLoading) {
    return <FullPageSkeleton />;
  }

  const portfolioData = {
    portfolioValue: indexState.total_value,
    indexPrice: (totalSupply && totalSupply > 0) ? indexState.total_value / totalSupply : 1.0, // NAV per token
    totalSupply: totalSupply || 0,
    apy: 0, // TODO: Calculate from historical data
    dailyChange: 0, // TODO: Calculate from historical data
    priceChange: 0, // TODO: Calculate from historical data
  };

  const rebalancingData = {
    nextRebalance: rebalancerStatus?.next_rebalance?.[0]
      ? new Date(Number(rebalancerStatus.next_rebalance[0] / 1_000_000n))
      : new Date(Date.now() + 3600000),
    nextAction: null, // Will compute from first deviation in future enhancement
    history: rebalancerStatus?.recent_history || [],
    isRebalancing: rebalanceMutation.isLoading,
    autoEnabled: autoRebalance,
  };

  const handleMint = async (amount: number) => {
    await mintMutation.mutateAsync(amount);
    // Refresh wallet balances after mint
    queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_WALLET_BALANCES] })
  };

  const handleRedeem = async (amount: number) => {
    await redeemMutation.mutateAsync(amount);
    // Refresh wallet balances after redeem
    queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_WALLET_BALANCES] })
  };

  const handleManualRebalance = async () => {
    await rebalanceMutation.mutateAsync();
  };

  const handleSendToken = (symbol: string) => {
    const token = walletBalances?.find(t => t.symbol === symbol);
    if (token) {
      setSendModalToken(token);
    }
  };

  const handleTransferSubmit = async (recipient: string, amount: string) => {
    if (!sendModalToken) return;

    await transferMutation.mutateAsync({
      tokenCanisterId: sendModalToken.canisterId,
      recipient,
      amount,
      decimals: sendModalToken.decimals,
    });
  };

  const handleRefreshBalances = () => {
    queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_WALLET_BALANCES] })
  };

  // Derive balances from walletBalances (single source of truth)
  const userICPIBalance = walletBalances?.find(b => b.symbol === 'ICPI')?.balanceFormatted || 0
  const userUSDTBalance = walletBalances?.find(b => b.symbol === 'ckUSDT')?.balanceFormatted || 0

  return (
    <>
      {/* Navigation Bar */}
      <div className="sticky top-0 z-50 border-b border-[#1f1f1f] bg-[#000000]">
        <div className="container px-3 py-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-6">
              <h1 className="text-lg font-mono font-bold text-white">ICPI</h1>
              <div className="flex gap-1">
                <Button
                  variant={currentView === 'dashboard' ? 'primary' : 'ghost'}
                  size="sm"
                  onClick={() => setCurrentView('dashboard')}
                  className="text-xs"
                >
                  DASHBOARD
                </Button>
                <Button
                  variant={currentView === 'docs' ? 'primary' : 'ghost'}
                  size="sm"
                  onClick={() => setCurrentView('docs')}
                  className="text-xs"
                >
                  DOCS
                </Button>
              </div>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={logout}
              className="text-xs"
            >
              DISCONNECT
            </Button>
          </div>
        </div>
      </div>

      {/* Main Content */}
      {currentView === 'dashboard' ? (
        <Dashboard
          principal={principal}
          tvl={portfolioData.portfolioValue}
          portfolioData={portfolioData}
          allocations={actualAllocations || []}
          rebalancingData={rebalancingData}
          userICPIBalance={userICPIBalance}
          userUSDTBalance={userUSDTBalance}
          tokenHoldings={holdings || []}
          walletBalances={walletBalances || []}
          tradeHistory={tradeHistory || []}
          onDisconnect={logout}
          onMint={handleMint}
          onRedeem={handleRedeem}
          onManualRebalance={handleManualRebalance}
          onToggleAutoRebalance={setAutoRebalance}
          onSendToken={handleSendToken}
          onRefreshBalances={handleRefreshBalances}
        />
      ) : (
        <Documentation />
      )}

      {sendModalToken && (
        <SendTokenModal
          token={sendModalToken}
          userPrincipal={principal}
          onClose={() => setSendModalToken(null)}
          onSend={handleTransferSubmit}
        />
      )}
    </>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ErrorBoundary>
        <AppContent />
      </ErrorBoundary>
    </QueryClientProvider>
  );
}

export default App;