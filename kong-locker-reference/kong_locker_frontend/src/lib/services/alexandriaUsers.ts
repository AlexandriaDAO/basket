import { HttpAgent, Actor } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';

// Alexandria Users canister ID
const ALEXANDRIA_USERS_ID = 'yo4hu-nqaaa-aaaap-qkmoq-cai';

// User data type
export interface AlexandriaUser {
  updated_at: bigint;
  principal: Principal;
  librarian: boolean;
  username: string;
  name: string;
  created_at: bigint;
  avatar: string;
}

// Cache for user data
class UserCache {
  private cache = new Map<string, { username: string; timestamp: number }>();
  private readonly TTL = 30 * 60 * 1000; // 30 minutes

  get(principal: string): string | null {
    const entry = this.cache.get(principal);
    if (!entry) return null;

    if (Date.now() - entry.timestamp > this.TTL) {
      this.cache.delete(principal);
      return null;
    }

    return entry.username;
  }

  set(principal: string, username: string): void {
    this.cache.set(principal, {
      username,
      timestamp: Date.now()
    });
  }

  clear(): void {
    this.cache.clear();
  }
}

// Alexandria Users IDL
const alexandriaUsersIdl = ({ IDL }: any) => {
  const User = IDL.Record({
    'updated_at': IDL.Nat64,
    'principal': IDL.Principal,
    'librarian': IDL.Bool,
    'username': IDL.Text,
    'name': IDL.Text,
    'created_at': IDL.Nat64,
    'avatar': IDL.Text,
  });

  const Result = IDL.Variant({
    'Ok': User,
    'Err': IDL.Text,
  });

  return IDL.Service({
    'get_user': IDL.Func([IDL.Principal], [Result], ['query']),
  });
};

export class AlexandriaUsersService {
  private actor: any;
  private cache = new UserCache();

  constructor() {
    const host = 'https://icp0.io';
    const agent = new HttpAgent({ host });

    this.actor = Actor.createActor(alexandriaUsersIdl, {
      agent,
      canisterId: ALEXANDRIA_USERS_ID,
    });
  }

  // Get username for a principal
  async getUsername(principal: Principal): Promise<string | null> {
    // Check cache first
    const cached = this.cache.get(principal.toText());
    if (cached) {
      return cached;
    }

    try {
      const result = await this.actor.get_user(principal);

      if ('Ok' in result) {
        const username = result.Ok.username;
        // Cache the result
        this.cache.set(principal.toText(), username);
        return username;
      } else {
        // User not found or error
        return null;
      }
    } catch (error) {
      console.error('Failed to fetch username for', principal.toText(), error);
      return null;
    }
  }

  // Batch fetch usernames (parallel requests)
  async getUsernames(principals: Principal[]): Promise<Map<string, string>> {
    const usernameMap = new Map<string, string>();

    // Filter out cached values first
    const uncachedPrincipals: Principal[] = [];
    for (const principal of principals) {
      const cached = this.cache.get(principal.toText());
      if (cached) {
        usernameMap.set(principal.toText(), cached);
      } else {
        uncachedPrincipals.push(principal);
      }
    }

    // Fetch uncached usernames in parallel
    if (uncachedPrincipals.length > 0) {
      const promises = uncachedPrincipals.map(principal =>
        this.getUsername(principal).then(username => ({
          principal: principal.toText(),
          username
        }))
      );

      const results = await Promise.allSettled(promises);

      for (const result of results) {
        if (result.status === 'fulfilled' && result.value.username) {
          usernameMap.set(result.value.principal, result.value.username);
        }
      }
    }

    return usernameMap;
  }

  // Clear cache
  clearCache(): void {
    this.cache.clear();
  }
}

// Singleton instance
export const alexandriaUsersService = new AlexandriaUsersService();