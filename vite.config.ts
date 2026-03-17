import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    svelte({
      compilerOptions: {
        // Enable runtime optimizations
        dev: process.env.NODE_ENV !== 'production',
      },
      // Aggressive hot reload
      hot: {
        preserveLocalState: true,
        noReload: true,
      },
    }),
  ],

  // Enable advanced caching
  cacheDir: './node_modules/.vite-cache',

  // Vite options tailored for Tauri development
  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },

  // Optimize dependencies for faster loading
  optimizeDeps: {
    include: [
      '@tauri-apps/api',
      '@tauri-apps/plugin-dialog',
      'lucide-svelte',
    ],
    // Don't exclude monaco-editor - let Vite handle the dependency
  },

  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,

    // Advanced optimizations
    reportCompressedSize: false, // Faster builds
    modulePreload: {
      polyfill: false, // We control the environment
    },
    rollupOptions: {
      output: {
        // Manual chunks for better code splitting
        manualChunks: {
          'monaco': ['monaco-editor'],
          'xterm': ['@xterm/xterm', '@xterm/addon-fit', '@xterm/addon-web-links'],
          'lucide': ['lucide-svelte'],
        },
      },
    },
    // Increase chunk size warning limit for large bundles like Monaco
    chunkSizeWarningLimit: 1000,
    // Enable CSS code splitting
    cssCodeSplit: true,
  },

  worker: {
    format: 'es',
  },
});
