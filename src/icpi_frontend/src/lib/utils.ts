import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatNumber(num: number | string, decimals: number = 2): string {
  const value = typeof num === 'string' ? parseFloat(num) : num;

  if (isNaN(value)) return '0';

  if (value >= 1e9) {
    return `$${(value / 1e9).toFixed(2)}B`;
  } else if (value >= 1e6) {
    return `$${(value / 1e6).toFixed(2)}M`;
  } else if (value >= 1e3) {
    return `$${(value / 1e3).toFixed(2)}K`;
  }

  return `$${value.toFixed(decimals)}`;
}

export function formatPercentage(value: number): string {
  const formatted = value.toFixed(2);
  const prefix = value >= 0 ? '+' : '';
  return `${prefix}${formatted}%`;
}

export function shortenAddress(address: string, chars = 4): string {
  return `${address.substring(0, chars)}...${address.substring(address.length - chars)}`;
}

// Token formatting utilities

// ckUSDT uses 6 decimals
export function formatCkUSDT(amount: bigint): string {
  const decimals = 6;
  const divisor = 10n ** BigInt(decimals);
  const whole = amount / divisor;
  const fraction = amount % divisor;
  return `${whole}.${fraction.toString().padStart(decimals, '0')}`;
}

// Most other tokens use 8 decimals (ALEX, ZERO, KONG, BOB)
export function formatToken(amount: bigint, symbol?: string): string {
  const decimals = 8;
  const divisor = 10n ** BigInt(decimals);
  const whole = amount / divisor;
  const fraction = amount % divisor;
  return `${whole}.${fraction.toString().padStart(decimals, '0')}`;
}

// ICPI uses 8 decimals like ICP
export function formatICPI(amount: bigint): string {
  return formatToken(amount, 'ICPI');
}