import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:4321',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    // Astro 7 auto-daemonizes `astro dev` in agent/CI environments (am-i-vibing
    // detection), which makes the spawned process exit early and Playwright's
    // webServer fail. Pin ASTRO_DEV_BACKGROUND=1 to force foreground mode.
    command: 'ASTRO_DEV_BACKGROUND=1 bun run dev',
    url: 'http://localhost:4321',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
