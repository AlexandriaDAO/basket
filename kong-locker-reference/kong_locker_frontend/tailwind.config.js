/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        // KongSwap Background Colors - Much better contrast for UI elements
        'kong-bg': {
          primary: 'rgb(13 17 28)',      // #0D111C - Dark background
          secondary: 'rgb(28 35 51)',    // #1C2333 - Darker blue-gray for main panels
          tertiary: 'rgb(39 48 66)',     // #273042 - Medium contrast for inner cards
          surface: 'rgb(51 62 84)',      // #333E54 - Good contrast for nested elements
        },
        
        // Text Colors - Exact KongSwap values
        'kong-text': {
          primary: 'rgb(255 255 255)',    // #FFFFFF - Pure white
          secondary: 'rgb(176 182 197)',  // #B0B6C5 - Light blue-gray
          disabled: 'rgb(107 114 128)',   // #6B7280 - Gray
        },
        
        // Brand Colors - Proper lime and orange like KongSwap
        'kong-accent': {
          green: 'rgb(50 205 50)',        // #32CD32 - True lime green (more yellow)
          blue: 'rgb(0 149 235)',         // #0095EB - KongSwap's brand blue
          orange: 'rgb(255 165 0)',       // #FFA500 - Bitcoin orange accent
        },
        
        // Semantic Colors - More natural colors with proper contrast
        'kong-success': {
          DEFAULT: 'rgb(50 205 50)',      // #32CD32 - True lime green
          hover: 'rgb(34 139 34)',        // #228B22 - Forest green hover
        },
        'kong-error': {
          DEFAULT: 'rgb(255 69 69)',      // #FF4545 - KongSwap's red
          hover: 'rgb(225 29 72)',        // #E11D48
        },
        'kong-warning': {
          DEFAULT: 'rgb(255 165 0)',      // #FFA500 - Bitcoin orange
          hover: 'rgb(255 140 0)',        // #FF8C00 - Dark orange hover
        },
        'kong-info': {
          DEFAULT: 'rgb(0 149 235)',      // #0095EB - KongSwap's blue
          hover: 'rgb(37 99 235)',        // #2563EB
        },
        
        // UI Elements - Better border contrast
        'kong-border': {
          DEFAULT: 'rgb(65 72 88)',       // #414858 - Much more visible borders
          light: 'rgb(75 82 98)',         // #4B5262 - Light borders for contrast
        },
      },
      
      fontFamily: {
        'exo': ['Exo 2', 'Space Grotesk', 'system-ui', 'sans-serif'],
        'space': ['Space Grotesk', 'system-ui', 'sans-serif'],
        'pixel': ['"Press Start 2P"', 'monospace'],
        'mono': ['"Roboto Mono"', '"Fira Code"', 'Courier', 'monospace'],
      },
      
      borderRadius: {
        'kong-panel': '0.75rem',    // 12px - main panel radius
        'kong-button': '90rem',     // nearly full rounded for buttons
      },
      
      animation: {
        'glow': 'glow 2s ease-in-out infinite',
        'shine': 'shine 2s infinite linear',
        'shimmer': 'shimmer 2s infinite',
        'spin-slow': 'spin 2s linear infinite',
      },
      
      keyframes: {
        glow: {
          '0%': { 
            filter: 'drop-shadow(0 0 2px rgb(50 205 50 / 0.5)) brightness(0.95)',
            opacity: '0.8',
            transform: 'scale(0.98)',
          },
          '50%': { 
            filter: 'drop-shadow(0 0 5px rgb(50 205 50 / 0.9)) brightness(1.1)',
            opacity: '1',
            transform: 'scale(1.02)',
          },
          '100%': { 
            filter: 'drop-shadow(0 0 2px rgb(50 205 50 / 0.5)) brightness(0.95)',
            opacity: '0.8',
            transform: 'scale(0.98)',
          },
        },
        shine: {
          '0%': { transform: 'translateX(-100%)' },
          '100%': { transform: 'translateX(50%)' },
        },
        shimmer: {
          '100%': { transform: 'translateX(100%)' },
        },
      },
      
      backgroundImage: {
        'gradient-kong': 'linear-gradient(135deg, rgb(0 149 235) 0%, rgb(50 205 50) 100%)',
        'gradient-button': 'linear-gradient(135deg, rgb(50 205 50) 0%, rgb(34 139 34) 100%)',
        'gradient-orange': 'linear-gradient(135deg, rgb(255 165 0) 0%, rgb(255 140 0) 100%)',
        'gradient-error': 'linear-gradient(135deg, rgb(255 69 69) 0%, rgb(225 29 72) 100%)',
        'gradient-page': 'linear-gradient(135deg, #0D111C 0%, #141A26 25%, #202632 50%, #141A26 75%, #0D111C 100%)',
      },
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography'),
  ],
};