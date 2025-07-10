/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{svelte,js,ts}'],
  theme: {
    colors: {
      // Include default colors plus our custom ones
      transparent: 'transparent',
      current: 'currentColor',
      black: '#000000',
      white: '#ffffff',
      gray: {
        50: '#f9fafb',
        100: '#f3f4f6',
        200: '#e5e7eb',
        300: '#d1d5db',
        400: '#9ca3af',
        500: '#6b7280',
        600: '#4b5563',
        700: '#374151',
        800: '#1f2937',
        900: '#111827',
      },
      // Our terminal colors
      'term-bg': '#000000',
      'term-fg': '#00ff00',
      'term-dim': '#00aa00',
      'term-bright': '#55ff55',
      'term-selection': '#00ff0033',
      'term-cursor': '#00ff00',
      'term-border': '#333333',
    },
    extend: {
      fontFamily: {
        'mono': ['SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', 'Consolas', 'monospace'],
      },
      fontSize: {
        'xs': '10px',
        'sm': '12px',
        'base': '14px',
      }
    }
  },
  plugins: []
}
