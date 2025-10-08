import { writable, derived, get } from 'svelte/store';
import { Principal } from '@dfinity/principal';
import { authStore } from './auth';
import { lpLockingService, LPLockingService } from '../services/lpLocking';
import { balanceStore, BalanceService } from '../services/balance';
import { kongSwapService } from '../services/kongSwapDirect';

// Required ICP amount for creating lock canister (2 ICP in e8s)
const REQUIRED_ICP_E8S = 2n * 100_000_000n; // 2 ICP = 200,000,000 e8s

// LP Locking Canister ID (where we send the 2 ICP)
const LP_LOCKING_CANISTER_ID = 'eazgb-giaaa-aaaap-qqc2q-cai';

// Retry configuration
const MAX_RETRIES = 3;
const BASE_DELAY_MS = 1000; // 1 second

export interface LPPosition {
  name: string;           // "ALEX_ckUSDT LP Token"
  symbol: string;         // "ALEX_ckUSDT"
  lp_token_id: bigint;   // LP token ID
  balance: number;        // LP token balance
  usd_balance: number;    // Total USD value
  chain_0: string;        // "IC"
  symbol_0: string;       // "ALEX"
  address_0: string;      // Token canister
  amount_0: number;       // Amount of first token
  usd_amount_0: number;   // USD value of first token
  chain_1: string;        // "IC"
  symbol_1: string;       // "ckUSDT"
  address_1: string;      // Token canister
  amount_1: number;       // Amount of second token
  usd_amount_1: number;   // USD value of second token
  ts: bigint;            // Timestamp
}

export interface UserLockState {
  canisterId: Principal | null;
  votingPower: number;
  lpPositions: LPPosition[];
  isCreating: boolean;
  creationStep: string;
  error: string | null;
  isLoading: boolean;
  needsSetup: boolean;
  isRegisteredOnKongSwap: boolean;
  isBlackholed: boolean;
  hasIcpFunding: boolean;
  // Mock data for testing State D
  useMockData: boolean;
}

const initialState: UserLockState = {
  canisterId: null,
  votingPower: 0,
  lpPositions: [],
  isCreating: false,
  creationStep: '',
  error: null,
  isLoading: false,
  needsSetup: false,
  isRegisteredOnKongSwap: false,
  isBlackholed: false,
  hasIcpFunding: false,
  useMockData: false,
};

export const userLockStore = writable<UserLockState>(initialState);

// Retry utility functions
function isRetryableError(error: Error): boolean {
  const message = error.message.toLowerCase();
  
  // Retryable errors (network, temporary failures)
  if (message.includes('network') || 
      message.includes('timeout') || 
      message.includes('temporarily unavailable') ||
      message.includes('connection') ||
      message.includes('service unavailable') ||
      message.includes('internal server error')) {
    return true;
  }
  
  // Non-retryable errors (authentication, insufficient funds, etc.)
  if (message.includes('insufficient funds') ||
      message.includes('authentication') ||
      message.includes('unauthorized') ||
      message.includes('bad fee') ||
      message.includes('already exists')) {
    return false;
  }
  
  // Default to non-retryable for safety
  return false;
}

function calculateBackoffDelay(attempt: number): number {
  // Exponential backoff with jitter: baseDelay * 2^attempt + random(0, 1000ms)
  const exponentialDelay = BASE_DELAY_MS * Math.pow(2, attempt);
  const jitter = Math.random() * 1000;
  return Math.min(exponentialDelay + jitter, 30000); // Cap at 30 seconds
}

async function withRetry<T>(
  operation: () => Promise<T>,
  operationName: string,
  updateStatus?: (status: string) => void
): Promise<T> {
  let lastError: Error;
  
  for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
    try {
      if (attempt > 0) {
        updateStatus?.(`Retrying ${operationName} (attempt ${attempt + 1}/${MAX_RETRIES + 1})...`);
        const delay = calculateBackoffDelay(attempt - 1);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
      
      return await operation();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      
      if (attempt === MAX_RETRIES || !isRetryableError(lastError)) {
        // Final attempt failed or error is not retryable
        throw lastError;
      }
      
      console.warn(`${operationName} failed (attempt ${attempt + 1}), retrying:`, lastError.message);
    }
  }
  
  throw lastError;
}

// Derived store to determine the current state for the UI
export const userState = derived(
  [authStore, userLockStore],
  ([$authStore, $userLockStore]) => {
    if (!$authStore.isAuthenticated) {
      return 'not-connected';
    }
    
    if ($userLockStore.isLoading) {
      return 'loading';
    }
    
    if (!$userLockStore.canisterId) {
      return 'no-canister';
    }
    
    if ($userLockStore.isCreating) {
      return 'creating';
    }
    
    if ($userLockStore.needsSetup) {
      return 'needs-setup';
    }
    
    if ($userLockStore.votingPower > 0 || $userLockStore.useMockData) {
      return 'has-liquidity';
    }
    
    return 'ready';
  }
);

export class UserLockService {
  // Load user's lock canister and state
  async loadUserState(userPrincipal: Principal): Promise<void> {
    userLockStore.update(state => ({ 
      ...state, 
      isLoading: true, 
      error: null 
    }));

    try {
      // Get authenticated identity to make the call
      const authState = get(authStore);
      
      // Verify authentication is still valid before using the identity
      if (!authState.authClient) {
        throw new Error('Auth client not initialized');
      }
      
      const isAuthenticated = await authState.authClient.isAuthenticated();
      if (!isAuthenticated || !authState.identity) {
        // Clear stale auth state
        authStore.update(state => ({
          ...state,
          isAuthenticated: false,
          identity: null,
          principal: null
        }));
        throw new Error('Authentication expired. Please log in again.');
      }
      
      // Create an authenticated lpLocking service for this user
      const authenticatedLpService = lpLockingService.createAuthenticatedService(authState.identity);
      
      // Check if user has a lock canister with retry
      const canisterId = await withRetry(
        () => authenticatedLpService.fetchUserLockCanister(userPrincipal),
        'fetch user lock canister'
      );
      
      if (canisterId) {
        console.log(`Found lock canister for user: ${canisterId.toText()}`);
        
        // Check if canister is registered by checking ALEX balance
        // If there's ALEX, it must have successfully registered and swapped
        let isRegistered = false;
        try {
          const balanceService = new BalanceService(authState.identity);
          const alexBalance = await balanceService.getAlexBalance(canisterId);
          isRegistered = alexBalance > 0n;
          console.log(`ALEX balance: ${alexBalance}, registered: ${isRegistered}`);
        } catch (error) {
          console.error('Failed to check ALEX balance:', error);
          // If we can't check, assume not registered (safer)
          isRegistered = false;
        }
        
        // Get detailed canister status
        let isBlackholed = false;
        let hasIcpFunding = false;
        
        // Check blackhole status
        try {
          const detailedStatus = await authenticatedLpService.fetchDetailedCanisterStatus();
          console.log('Detailed status response:', detailedStatus);
          if (detailedStatus && 'Ok' in detailedStatus) {
            isBlackholed = detailedStatus.Ok.is_blackholed;
            console.log(`Canister blackhole status from backend: ${isBlackholed}`);
          } else if (detailedStatus && 'Err' in detailedStatus) {
            console.error('Detailed status error:', detailedStatus.Err);
            // If we can't get status, assume it's blackholed (safer assumption)
            isBlackholed = true;
          }
        } catch (error) {
          console.error('Could not fetch detailed canister status:', error);
          // If the call fails, assume blackholed (safer)
          isBlackholed = true;
        }
        
        // Check ICP balance of the lock canister
        try {
          const balanceService = new BalanceService(authState.identity);
          const icpBalance = await balanceService.getIcpBalance(canisterId);
          // Need at least 0.99 ICP (99_000_000 e8s) for registration
          hasIcpFunding = icpBalance >= 99_000_000n;
          console.log(`Lock canister ICP balance: ${icpBalance} e8s, has funding: ${hasIcpFunding}`);
        } catch (error) {
          console.warn('Could not check ICP balance:', error);
        }
        
        // Get LP positions and calculate voting power if registered
        let votingPower = 0;
        let lpPositions: LPPosition[] = [];
        if (isRegistered) {
          try {
            // Fetch detailed LP positions from KongSwap directly
            lpPositions = await withRetry(
              () => kongSwapService.getLPPositions(canisterId),
              'fetch LP positions'
            );
            
            // Calculate voting power from LP positions
            votingPower = kongSwapService.calculateVotingPower(lpPositions);
            
            console.log(`Found ${lpPositions.length} LP positions, total voting power: ${votingPower}`);
          } catch (error) {
            console.warn('Could not fetch LP positions:', error);
            // Fallback to old method if new method fails
            try {
              votingPower = await withRetry(
                () => authenticatedLpService.fetchVotingPower(userPrincipal),
                'fetch voting power (fallback)'
              );
            } catch (fallbackError) {
              console.warn('Fallback voting power fetch also failed:', fallbackError);
            }
          }
        }
        
        // Determine if setup is needed
        // Setup is complete only when both registered AND blackholed
        // ICP funding is irrelevant once registered (consumed in swap)
        const needsSetup = !isRegistered || !isBlackholed;
        
        userLockStore.update(state => ({
          ...state,
          canisterId,
          votingPower,
          lpPositions,
          isLoading: false,
          isRegisteredOnKongSwap: isRegistered,
          isBlackholed,
          hasIcpFunding,
          needsSetup,
        }));
        
        // Store canister ID in localStorage for persistence
        if (typeof window !== 'undefined') {
          localStorage.setItem(`lockCanister_${userPrincipal.toText()}`, canisterId.toText());
        }
        
      } else {
        // No canister yet
        userLockStore.update(state => ({
          ...state,
          canisterId: null,
          votingPower: 0,
          lpPositions: [],
          isLoading: false,
        }));
      }
    } catch (error) {
      console.error('Failed to load user state:', error);
      userLockStore.update(state => ({
        ...state,
        error: error instanceof Error ? error.message : 'Failed to load user state',
        isLoading: false,
      }));
    }
  }

  // Create a new lock canister
  async createLockCanister(): Promise<void> {
    // SAFETY CHECK: First verify user doesn't already have a canister
    const authState = get(authStore);
    
    // Verify authentication is still valid
    if (!authState.authClient) {
      userLockStore.update(state => ({
        ...state,
        error: 'Auth client not initialized',
      }));
      return;
    }
    
    const isAuthenticated = await authState.authClient.isAuthenticated();
    if (!isAuthenticated || !authState.principal) {
      // Clear stale auth state
      authStore.update(state => ({
        ...state,
        isAuthenticated: false,
        identity: null,
        principal: null
      }));
      userLockStore.update(state => ({
        ...state,
        error: 'Authentication expired. Please log in again.',
      }));
      return;
    }
    
    // Check if user already has a canister
    userLockStore.update(state => ({ 
      ...state, 
      isLoading: true, 
      error: null 
    }));
    
    try {
      // Create authenticated service to check for existing canister
      const authenticatedLpService = lpLockingService.createAuthenticatedService(authState.identity);
      const existingCanister = await authenticatedLpService.fetchUserLockCanister(authState.principal);
      if (existingCanister) {
        // User already has a canister! Load its state instead
        console.log('User already has a lock canister:', existingCanister.toText());
        await this.loadUserState(authState.principal);
        userLockStore.update(state => ({
          ...state,
          error: 'You already have a lock canister. Loading its state...',
          isLoading: false,
        }));
        return;
      }
    } catch (error) {
      console.error('Error checking for existing canister:', error);
    }
    
    // Check if user has enough ICP
    const currentBalance = get(balanceStore);
    
    if (currentBalance.icpBalance < REQUIRED_ICP_E8S) {
      const currentIcp = Number(currentBalance.icpBalance) / 100_000_000;
      userLockStore.update(state => ({
        ...state,
        error: `Insufficient ICP balance. You have ${currentIcp.toFixed(2)} ICP but need 2 ICP to create a lock canister.`,
        isLoading: false,
      }));
      return;
    }

    userLockStore.update(state => ({ 
      ...state, 
      isCreating: true, 
      creationStep: 'Processing payment...',
      error: null 
    }));

    try {
      // Get user's identity to create balance service for ICP transfer
      const authState = get(authStore);
      if (!authState.identity) {
        throw new Error('User identity not available');
      }

      const balanceService = new BalanceService(authState.identity);
      const lpLockingPrincipal = Principal.fromText(LP_LOCKING_CANISTER_ID);

      // Step 1: Approve the LP Locking canister to spend 2 ICP with retry
      const updateStatus = (status: string) => {
        userLockStore.update(state => ({ ...state, creationStep: status }));
      };
      
      updateStatus('Approving 2 ICP payment...');
      
      // Call icrc2_approve with retry mechanism
      const approvalBlockIndex = await withRetry(
        () => balanceService.approveIcp(lpLockingPrincipal, REQUIRED_ICP_E8S + 10_000n), // 2 ICP + fee
        'ICP approval',
        updateStatus
      );
      
      // Wait a moment for the approval to be processed
      userLockStore.update(state => ({ 
        ...state, 
        creationStep: 'Waiting for approval confirmation...' 
      }));
      await new Promise(resolve => setTimeout(resolve, 2000)); // Wait 2 seconds

      // Step 2: Call the backend to create the lock canister (it will pull the ICP) with retry
      updateStatus('Creating canister...');
      
      // Create an authenticated service to call the backend
      const authenticatedLpService = new LPLockingService(authState.identity);
      const canisterId = await withRetry(
        () => authenticatedLpService.createLockCanister(),
        'canister creation',
        updateStatus
      );
      
      // Update UI with additional steps
      userLockStore.update(state => ({ 
        ...state, 
        creationStep: 'Installing code...' 
      }));
      await new Promise(resolve => setTimeout(resolve, 500));
      
      userLockStore.update(state => ({ 
        ...state, 
        creationStep: 'Saving address...' 
      }));
      await new Promise(resolve => setTimeout(resolve, 500));
      
      userLockStore.update(state => ({ 
        ...state, 
        creationStep: 'Funding with ICP...' 
      }));
      await new Promise(resolve => setTimeout(resolve, 500));
      
      userLockStore.update(state => ({ 
        ...state, 
        creationStep: 'Registering with KongSwap...' 
      }));
      await new Promise(resolve => setTimeout(resolve, 500));
      
      userLockStore.update(state => ({ 
        ...state, 
        creationStep: 'Blackholing canister...' 
      }));
      await new Promise(resolve => setTimeout(resolve, 500));
      
      // Complete creation - update basic state first
      userLockStore.update(state => ({
        ...state,
        isCreating: false,
        creationStep: '',
        canisterId: canisterId,
      }));
      
      // Now reload full state to get registration status, blackhole status, etc.
      if (authState.principal) {
        await this.loadUserState(authState.principal);
      }
      
    } catch (error) {
      console.error('Failed to create lock canister:', error);
      userLockStore.update(state => ({
        ...state,
        error: error instanceof Error ? error.message : 'Failed to create lock canister',
        isCreating: false,
        creationStep: '',
      }));
    }
  }

  // Complete canister setup
  async completeSetup(): Promise<void> {
    userLockStore.update(state => ({ 
      ...state, 
      isLoading: true,
      error: null 
    }));

    try {
      const authState = get(authStore);
      if (!authState.identity) {
        throw new Error('Not authenticated');
      }
      
      // Create authenticated service
      const authenticatedLpService = lpLockingService.createAuthenticatedService(authState.identity);
      
      const result = await withRetry(
        () => authenticatedLpService.completeMyCanisterSetup(),
        'complete canister setup'
      );
      
      // After completion, reload the full state to get updated status
      if (authState.principal) {
        await this.loadUserState(authState.principal);
      }
      
      // Show success message
      userLockStore.update(state => ({
        ...state,
        error: null,
        needsSetup: false,
      }));
      
      console.log('Setup completed successfully:', result);
    } catch (error) {
      console.error('Failed to complete setup:', error);
      userLockStore.update(state => ({
        ...state,
        error: error instanceof Error ? error.message : 'Failed to complete setup',
        isLoading: false,
      }));
    }
  }

  // Clear any error state
  clearError(): void {
    userLockStore.update(state => ({ ...state, error: null }));
  }

  // Reset to initial state (for logout)
  reset(): void {
    userLockStore.set(initialState);
  }

  // Mock data methods for testing State D
  enableMockData(): void {
    const mockPositions: LPPosition[] = [
      {
        name: "ALEX_ckUSDT LP Token",
        symbol: "ALEX_ckUSDT",
        lp_token_id: 1655n,
        balance: 4.0,
        usd_balance: 3.46,
        chain_0: "IC",
        symbol_0: "ALEX",
        address_0: "xkbqi-2qaaa-aaaar-qahqq-cai",
        amount_0: 9.59,
        usd_amount_0: 1.74,
        chain_1: "IC", 
        symbol_1: "ckUSDT",
        address_1: "cngnf-vqaaa-aaaag-qcqfq-cai",
        amount_1: 1.72,
        usd_amount_1: 1.72,
        ts: BigInt(Date.now())
      },
      {
        name: "ALEX_ICP LP Token",
        symbol: "ALEX_ICP", 
        lp_token_id: 1656n,
        balance: 2.0,
        usd_balance: 3.85,
        chain_0: "IC",
        symbol_0: "ALEX",
        address_0: "xkbqi-2qaaa-aaaar-qahqq-cai",
        amount_0: 10.61,
        usd_amount_0: 1.93,
        chain_1: "IC",
        symbol_1: "ICP",
        address_1: "ryjl3-tyaaa-aaaaa-aaaba-cai", 
        amount_1: 0.390290,
        usd_amount_1: 1.93,
        ts: BigInt(Date.now())
      },
      {
        name: "BOB_ICP LP Token",
        symbol: "BOB_ICP",
        lp_token_id: 1662n,
        balance: 0.786633,
        usd_balance: 1.94,
        chain_0: "IC",
        symbol_0: "BOB",
        address_0: "7pail-xaaaa-aaaas-aabmq-cai",
        amount_0: 4.47,
        usd_amount_0: 0.97,
        chain_1: "IC",
        symbol_1: "ICP",
        address_1: "ryjl3-tyaaa-aaaaa-aaaba-cai",
        amount_1: 0.20,
        usd_amount_1: 0.97,
        ts: BigInt(Date.now())
      }
    ];
    
    userLockStore.update(state => ({
      ...state,
      useMockData: true,
      canisterId: Principal.fromText('rdmx6-jaaaa-aaaah-qcaaa-cai'), // Mock canister ID
      votingPower: 925, // Updated total voting power (731 + 194 for BOB)
      lpPositions: mockPositions,
    }));
  }

  disableMockData(): void {
    userLockStore.update(state => ({
      ...state,
      useMockData: false,
      // Keep actual canister ID and voting power if they exist, otherwise reset
      votingPower: state.votingPower > 0 && !state.useMockData ? state.votingPower : 0,
    }));
  }
}

export const userLockService = new UserLockService();