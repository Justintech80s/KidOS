import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  test: {
    environment: 'jsdom',
    setupFiles: './src/test-setup.ts',
    // Windows hosted runners execute the shell and E2E suites in parallel and
    // can exceed Vitest's 5s default even when the same tests complete locally.
    // Keep a bounded timeout while avoiding false failures under CI load.
    testTimeout: 15_000,
  },
});
