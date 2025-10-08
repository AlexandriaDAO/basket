/**
 * Purely dynamic token color generation utility
 * NO hardcoded values - everything generated dynamically from token symbols
 */

export interface TokenColors {
  primary: string;
  background: string;
  border: string;
  text: string;
}

/**
 * Generate completely dynamic colors for any token symbol using deterministic hashing
 * Same token symbol will ALWAYS produce the same colors
 * NO hardcoded tokens - works for any token in the ecosystem
 */
export function getTokenColors(symbol: string): TokenColors {
  if (!symbol) {
    return {
      primary: 'hsl(0, 0%, 60%)',
      background: 'hsl(0, 0%, 15%)',
      border: 'hsl(0, 0%, 30%)',
      text: 'hsl(0, 0%, 60%)'
    };
  }

  // Create deterministic hash from symbol
  let hash = 0;
  for (let i = 0; i < symbol.length; i++) {
    const char = symbol.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32bit integer
  }
  
  // Use golden ratio for optimal color distribution
  const goldenRatio = 137.508;
  const hue = (Math.abs(hash) * goldenRatio) % 360;
  
  // Generate appealing color schemes
  const saturation = 65 + (Math.abs(hash) % 25); // 65-90% for vibrant colors
  const lightness = 55 + (Math.abs(hash) % 20);   // 55-75% for good contrast
  
  return {
    primary: `hsl(${hue}, ${saturation}%, ${lightness}%)`,
    background: `hsl(${hue}, 30%, 12%)`,
    border: `hsl(${hue}, 40%, 25%)`,
    text: `hsl(${hue}, ${saturation}%, ${lightness}%)`
  };
}

/**
 * Generate dynamic icon for any token symbol
 * Uses first letter and some algorithmic variation
 */
export function generateTokenIcon(symbol: string): string {
  if (!symbol) return '🪙';
  
  // Use first letter for basic icon selection
  const firstLetter = symbol[0].toLowerCase();
  const hash = symbol.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
  
  // Generate different icon styles based on hash
  const iconVariation = hash % 6;
  
  switch (iconVariation) {
    case 0: return '🪙'; // Coin
    case 1: return '💎'; // Diamond
    case 2: return '⚡'; // Lightning
    case 3: return '🔥'; // Fire
    case 4: return '⭐'; // Star
    case 5: return '💫'; // Sparkle
    default: return '🪙';
  }
}