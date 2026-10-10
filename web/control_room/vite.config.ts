import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// base './' so the built site works from any folder or static host
export default defineConfig({
  base: './',
  plugins: [react()],
  // /v1 goes to the backend, which sends no CORS header, so the browser cannot call it directly
  server: { port: 5173, host: '127.0.0.1', proxy: { '/v1': 'http://95.217.14.92:8790' } },
  build: { target: 'es2020', chunkSizeWarningLimit: 700 },
});
