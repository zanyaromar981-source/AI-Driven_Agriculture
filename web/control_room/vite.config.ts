import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// The API lives on the backend server. In development, /v1 is proxied there so the browser sees one
// address (no CORS needed). A built site calls VITE_API_URL directly; that address must be in the
// server's HTTP__CORS_ORIGINS list (FRONTEND.md section 3).
const API = process.env.API_TARGET || 'http://95.217.14.92:8790';

export default defineConfig({
  base: './',
  plugins: [react()],
  server: { port: 5173, host: '127.0.0.1', proxy: { '/v1': { target: API, changeOrigin: true } } },
  preview: { port: 4173, host: '127.0.0.1', proxy: { '/v1': { target: API, changeOrigin: true } } },
  build: { target: 'es2020', chunkSizeWarningLimit: 700 },
});
