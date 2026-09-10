import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    // Hosted Windows CI can be materially slower than local runs when the
    // workspace suites execute together. Preserve a hard bound, but avoid the
    // 5s default producing false negatives for otherwise healthy E2E flows.
    testTimeout: 15_000,
  }
});
