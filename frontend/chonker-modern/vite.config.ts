import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  server: {
    port: 5173,
    strictPort: true,
    host: 'localhost',
    hmr: {
      overlay: false  // Disable error overlay that blocks
    },
    watch: {
      usePolling: true,  // Force polling for better file watching
      interval: 100      // Check every 100ms
    }
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'esnext',
    minify: 'esbuild'
  },
  optimizeDeps: {
    exclude: ['svelte'],  // Don't pre-bundle Svelte
    force: true           // Force re-optimization
  },
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_'],
  // Disable caching in development
  define: {
    __VITE_IS_PRODUCTION__: JSON.stringify(false)
  }
})
