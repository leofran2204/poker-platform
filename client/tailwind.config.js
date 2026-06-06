/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        poker: {
          green: '#1a6b3c',
          'green-dark': '#0d4d2b',
          'green-light': '#228b4a',
          gold: '#d4a843',
          'gold-dark': '#b8922e',
          red: '#c0392b',
          blue: '#2980b9',
          black: '#1a1a2e',
          'black-light': '#16213e',
          felt: '#0a5c36',
        },
      },
      fontFamily: {
        poker: ['"Playfair Display"', 'Georgia', 'serif'],
      },
      animation: {
        'card-deal': 'cardDeal 0.5s ease-out',
        'chip-stack': 'chipStack 0.3s ease-out',
        'pulse-glow': 'pulseGlow 2s infinite',
      },
      keyframes: {
        cardDeal: {
          '0%': { transform: 'translateY(-100px) rotate(-10deg)', opacity: '0' },
          '100%': { transform: 'translateY(0) rotate(0deg)', opacity: '1' },
        },
        chipStack: {
          '0%': { transform: 'translateY(-20px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        pulseGlow: {
          '0%, 100%': { boxShadow: '0 0 5px rgba(212, 168, 67, 0.5)' },
          '50%': { boxShadow: '0 0 20px rgba(212, 168, 67, 0.8)' },
        },
      },
    },
  },
  plugins: [],
}