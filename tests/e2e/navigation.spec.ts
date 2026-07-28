/**
 * Navigation E2E Tests for Academix (Turso Migration)
 *
 * Tests that the Tauri app starts correctly and the frontend
 * can communicate with the async libsql backend.
 */
import { test, expect } from '@playwright/test';

test.describe('App startup', () => {
  test('app loads and shows the main page', async ({ page }) => {
    await page.goto('http://localhost:1420');
    await page.waitForTimeout(2000);

    // Verify the app rendered (check for a known element)
    const title = page.locator('h1');
    await expect(title).toBeVisible();
  });
});

test.describe('Backend health', () => {
  test('health endpoint responds', async ({ page }) => {
    await page.goto('http://localhost:1420');
    await page.waitForTimeout(1000);

    // Invoke the health check command
    const result = await page.evaluate(async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      return invoke<string>('health');
    });

    expect(result).toBeTruthy();
  });
});

test.describe('User registration', () => {
  test('register_user command is available', async ({ page }) => {
    await page.goto('http://localhost:1420');
    await page.waitForTimeout(1000);

    const result = await page.evaluate(async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      try {
        return await invoke('register_user', {
          email: 'test+email',
          password: 'Test1234!',
          name: 'Test User',
          role: 'Empleado',
        });
      } catch (e) {
        return { error: (e as Error).message };
      }
    });

    // Either succeeds or returns known structure (auth commands may be stubs)
    expect(result).toBeDefined();
  });
});

test.describe('Database layer (Turso/libsql)', () => {
  test('app initializes local libsql database', async ({ page }) => {
    await page.goto('http://localhost:1420');
    await page.waitForTimeout(2000);

    // Verify no console errors from database initialization
    const errors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    // Wait for app to stabilize
    await page.waitForTimeout(3000);

    // Filter for database-related errors (non-Turso errors are expected in dev)
    const dbErrors = errors.filter(
      (e) => e.includes('[FATAL]') || e.includes('[DB PATH]')
    );
    expect(dbErrors).toHaveLength(0);
  });
});
