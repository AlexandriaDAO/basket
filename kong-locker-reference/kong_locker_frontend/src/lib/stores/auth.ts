import { writable, derived } from 'svelte/store';
import { AuthClient } from '@dfinity/auth-client';
import type { Identity } from '@dfinity/agent';
import type { Principal } from '@dfinity/principal';

// Derivation origin for consistent identity across core apps
const DERIVATION_ORIGIN = 'https://yj5ba-aiaaa-aaaap-qkmoa-cai.icp0.io';

export interface AuthState {
  authClient: AuthClient | null;
  identity: Identity | null;
  principal: Principal | null;
  isAuthenticated: boolean;
  isInitialized: boolean;
  isLoading: boolean;
  error: string | null;
}

const initialState: AuthState = {
  authClient: null,
  identity: null,
  principal: null,
  isAuthenticated: false,
  isInitialized: false,
  isLoading: false,
  error: null,
};

export const authStore = writable<AuthState>(initialState);

class AuthService {
  private authClient: AuthClient | null = null;

  async init(): Promise<void> {
    authStore.update(state => ({ ...state, isLoading: true }));
    
    try {
      this.authClient = await AuthClient.create({
        idleOptions: {
          disableDefaultIdleCallback: true,
        },
        // Use derivation origin for consistent identity across core apps
        keyType: 'Ed25519',
      });
      
      if (await this.authClient.isAuthenticated()) {
        const identity = this.authClient.getIdentity();
        const principal = identity.getPrincipal();
        
        authStore.update(state => ({
          ...state,
          authClient: this.authClient,
          identity,
          principal,
          isAuthenticated: true,
          isInitialized: true,
          isLoading: false,
          error: null,
        }));
      } else {
        authStore.update(state => ({
          ...state,
          authClient: this.authClient,
          isInitialized: true,
          isLoading: false,
          error: null,
        }));
      }
    } catch (error) {
      console.error('Auth initialization failed:', error);
      authStore.update(state => ({
        ...state,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Authentication initialization failed',
      }));
    }
  }

  async login(): Promise<void> {
    if (!this.authClient) {
      await this.init();
    }

    if (!this.authClient) {
      throw new Error('Auth client not initialized');
    }

    authStore.update(state => ({ ...state, isLoading: true }));

    return new Promise((resolve, reject) => {
      this.authClient!.login({
        identityProvider: 'https://identity.internetcomputer.org',
        derivationOrigin: DERIVATION_ORIGIN,
        maxTimeToLive: BigInt(7) * BigInt(24) * BigInt(3600) * BigInt(1000000000), // 7 days in nanoseconds
        onSuccess: () => {
          const identity = this.authClient!.getIdentity();
          const principal = identity.getPrincipal();
          
          authStore.update(state => ({
            ...state,
            identity,
            principal,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          }));
          resolve();
        },
        onError: (error) => {
          console.error('Login failed:', error);
          authStore.update(state => ({
            ...state,
            isLoading: false,
            error: error || 'Login failed',
          }));
          reject(new Error(error || 'Login failed'));
        }
      });
    });
  }

  async logout(): Promise<void> {
    if (this.authClient) {
      await this.authClient.logout();
    }
    
    authStore.update(state => ({
      ...state,
      identity: null,
      principal: null,
      isAuthenticated: false,
      error: null,
    }));
  }

  clearError(): void {
    authStore.update(state => ({ ...state, error: null }));
  }
}

// Create singleton instance
export const authService = new AuthService();

// Derived stores for convenience
export const isAuthenticated = derived(authStore, $auth => $auth.isAuthenticated);
export const principal = derived(authStore, $auth => $auth.principal);
export const identity = derived(authStore, $auth => $auth.identity);
export const isAuthLoading = derived(authStore, $auth => $auth.isLoading);
export const authError = derived(authStore, $auth => $auth.error);
export const isAuthInitialized = derived(authStore, $auth => $auth.isInitialized);