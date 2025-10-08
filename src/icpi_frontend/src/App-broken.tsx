import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { InternetIdentityProvider, useInternetIdentity } from 'ic-use-internet-identity';

import { Dashboard } from './components/Dashboard';
import { Documentation } from './components/Documentation';
import { FullPageSkeleton } from './components/LoadingStates';
import { ErrorBoundary } from './components/ErrorBoundary';
import { Button } from './components/ui/button';

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
} from './hooks/useICPI';
import { SendTokenModal } from './components/SendTokenModal';
import { useState } from 'react';
import { useICPIBackend } from './hooks/actors';
import { canisterId as icpiCanisterId } from 'declarations/icpi_backend';

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
  const { identity, clear, login } = useInternetIdentity();
  const { actor: icpiActor, authenticated } = useICPIBackend();

  const [autoRebalance, setAutoRebalance] = useState(true);
  const [sendModalToken, setSendModalToken] = useState<UserTokenBalance | null>(null);
  const [currentView, setCurrentView] = useState<'dashboard' | 'docs'>('dashboard');

  // Derive principal from identity
  const principal = identity?.getPrincipal().toString() || '';

  // Use React Query hooks - these now use the actor hook internally
  const { data: indexState, isLoading: indexLoading } = useIndexState();
  const { data: rebalancerStatus } = useRebalancerStatus();
  const { data: tvlData } = useTVLData();
  const { data: holdings } = useHoldings();
  const { data: allocations } = useAllocation();
  const { data: actualAllocations } = useActualAllocations();
  const { data: totalSupply } = useTotalSupply();

  const mintMutation = useMintICPI();
  const redeemMutation = useRedeemICPI();
  const rebalanceMutation = useManualRebalance();

  // Wallet balance hooks
  const { data: walletBalances, isLoading: balancesLoading } = useUserWalletBalances(principal);
  const transferMutation = useTransferToken();

  const handleLogin = async () => {
    const isLocal = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';

    await login({
      identityProvider: isLocal ? 'http://localhost:4943' : 'https://identity.ic0.app',
      maxTimeToLive: BigInt(7 * 24 * 60 * 60 * 1000 * 1000 * 1000), // 7 days
    });
  };

  const handleLogout = async () => {
    await clear();
  };

  if (!authenticated || !identity) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-[#000000]">
        <div className="w-full max-w-sm border border-[#1f1f1f] bg-[#0a0a0a] p-6">
          <div className="space-y-4">
            <h1 className="text-xl font-mono text-white tracking-wider">ICPI</h1>
            <p className="text-xs font-mono text-[#999999]">
              Internet Computer Portfolio Index
            </p>
            <Button
              onClick={handleLogin}
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

  // Show loading skeleton while data loads
  if (!indexState || indexLoading) {
    return <FullPageSkeleton />;
  }

  const portfolioData = {
    portfolioValue: indexState.total_value,
    indexPrice: (totalSupply && totalSupply > 0) ? indexState.total_value / totalSupply : 1.0,
    totalSupply: totalSupply || 0,
    apy: 0,
    dailyChange: 0,
    priceChange: 0,
  };

  const rebalancingData = {
    nextRebalance: new Date(Date.now() + 3600000),
    nextAction: rebalancerStatus?.next_action || null,
    history: rebalancerStatus?.history || [],
    isRebalancing: rebalanceMutation.isPending,
    autoEnabled: autoRebalance,
  };

  const handleMint = async (amount: number) => {
    await mintMutation.mutateAsync(amount);
  };

  const handleRedeem = async (amount: number) => {
    await redeemMutation.mutateAsync(amount);
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
    queryClient.invalidateQueries({ queryKey: ['userWalletBalances'] });
  };

  // Derive balances from walletBalances
  const userICPIBalance = walletBalances?.find(b => b.symbol === 'ICPI')?.balanceFormatted || 0;
  const userUSDTBalance = walletBalances?.find(b => b.symbol === 'ckUSDT')?.balanceFormatted || 0;

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
              onClick={handleLogout}
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
          onDisconnect={handleLogout}
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
    <InternetIdentityProvider>
      <QueryClientProvider client={queryClient}>
        <ErrorBoundary>
          <AppContent />
        </ErrorBoundary>
      </QueryClientProvider>
    </InternetIdentityProvider>
  );
}

export default App;
