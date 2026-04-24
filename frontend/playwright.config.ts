import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for LIMMA Frontend Verification Test Suite.
 *
 * Expects:
 *   - Frontend dev server running on http://localhost:3000
 *   - Backend API server running on http://localhost:8900
 */
export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false, // Run serially — scans are heavy
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: [
    ['html', { open: 'never' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['list'],
  ],
  timeout: 300_000, // 5 minutes per test — scans can be long
  expect: {
    timeout: 60_000,
  },
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    actionTimeout: 30_000,
    navigationTimeout: 30_000,
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
