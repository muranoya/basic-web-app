import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

export default defineConfig({
  base: '/',
  publicDir: 'assets',
  plugins: [react()],
  server: {
    host: false,
    hmr: false,
  },
  build: {
    manifest: true,
    outDir: 'dist',
    rollupOptions: {
      input: '/src/main.tsx',
    },
    modulePreload: {
      polyfill: false,
    },
  },
});
