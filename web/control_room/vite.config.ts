import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// base './' so the built site works from any folder or static host
export default defineConfig({
  base: './',
  plugins: [react()],
  server: { port: 5173, host: '127.0.0.1' },
  build: { target: 'es2020', chunkSizeWarningLimit: 700 },
});
