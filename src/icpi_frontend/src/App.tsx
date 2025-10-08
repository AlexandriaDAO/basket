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

function AppContent() {
  const queryClient = useQueryClient()
  const [authClient, setAuthClient] = useState<AuthClient | null>(null);
  const [identity, setIdentity] = useState<Identity | null>(null);
  const [principal, setPrincipal] = useState<string>('');
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [actor, setActor] = useState<Actor | null>(null);
  const [agent, setAgent] = useState<HttpAgent | null>(null);
  const [autoRebalance, setAutoRebalance] = useState(true);
  const [sendModalToken, setSendModalToken] = useState<UserTokenBalance | null>(null);
  const [currentView, setCurrentView] = useState<'dashboard' | 'docs'>('dashboard');

  // Performance timing - mark page load start
  useEffect(() => {
    performance.mark('app-start');
  }, []);

  // Use React Query hooks
  const { data: indexState, isLoading: indexLoading } = useIndexState(actor);
  const { data: rebalancerStatus } = useRebalancerStatus(actor);
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

  useEffect(() => {
    AuthClient.create({
      idleOptions: {
        idleTimeout: 1000 * 60 * 60 * 24 * 7, // 7 days
        disableDefaultIdleCallback: true,
        disableIdle: false,
      }
    }).then(async (client) => {
      setAuthClient(client);
      const isAuth = await client.isAuthenticated();
      if (isAuth) {
        const identity = client.getIdentity();
        setIdentity(identity);
        setPrincipal(identity.getPrincipal().toString());
        setIsAuthenticated(true);
      }
    });
  }, []);

  useEffect(() => {
    if (isAuthenticated && identity) {
      createActor();
    }
  }, [isAuthenticated, identity]);

  const login = async () => {
    if (!authClient) return;

    const isLocal = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';
    const identityProvider = isLocal ? 'http://localhost:4943' : 'https://identity.ic0.app';
    const weekInNanoSeconds = BigInt(7 * 24 * 60 * 60 * 1000 * 1000 * 1000);

    await authClient.login({
      identityProvider,
      maxTimeToLive: weekInNanoSeconds,
      onSuccess: () => {
        const identity = authClient.getIdentity();
        setIdentity(identity);
        setPrincipal(identity.getPrincipal().toString());
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
  };

  const createActor = () => {
    if (!identity) throw new Error('Not authenticated');

    const isLocal = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';
    const host = isLocal ? 'http://localhost:4943' : 'https://icp-api.io';

    // Create authenticated agent with explicit polling configuration
    const newAgent = new HttpAgent({
      identity,
      host,
      // Increase ingress expiry to 5 minutes for long-running update calls
      ingressExpiryMs: 5 * 60 * 1000,
    });

    if (isLocal) {
      newAgent.fetchRootKey().catch(console.error);
    }

    // Use authenticated actor (needed for update calls like get_index_state)
    const newActor = Actor.createActor(icpiIdlFactory, {
      agent: newAgent,
      canisterId: icpiCanisterId,
    });

    setAgent(newAgent);
    setActor(newActor);
    return newActor;
  };

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

  // Prepare data for Dashboard - no mock data, fail visibly if data unavailable
  // Note: rebalancerStatus is optional as it may timeout, don't block UI on it
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
    nextRebalance: new Date(Date.now() + 3600000), // TODO: Get from rebalancer status
    nextAction: rebalancerStatus?.next_action || null,
    history: rebalancerStatus?.history || [],
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

  if (!indexState || indexLoading) {
    return <FullPageSkeleton />;
  }

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