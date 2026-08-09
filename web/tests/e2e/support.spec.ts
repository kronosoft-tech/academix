import { test, expect } from '@playwright/test';

test.describe('Support — Public Page Access', () => {
  test('unauthenticated access to /dashboard/support redirects to login', async ({ page }) => {
    await page.goto('/dashboard/support');
    await page.waitForURL('**/auth/login');
    expect(page.url()).toContain('/auth/login');
  });

  test('unauthenticated access to /dashboard/support/new redirects to login', async ({ page }) => {
    await page.goto('/dashboard/support/new');
    await page.waitForURL('**/auth/login');
    expect(page.url()).toContain('/auth/login');
  });

  test('unauthenticated access to /dashboard/support/chat redirects to login', async ({ page }) => {
    await page.goto('/dashboard/support/chat');
    await page.waitForURL('**/auth/login');
    expect(page.url()).toContain('/auth/login');
  });
});

test.describe('PQRS Support System — DB-dependent', () => {
  // These tests require an authenticated session and a running Turso DB.
  // To run: set TURSO_URL and TURSO_AUTH_TOKEN, seed a test user, and set auth cookie.

  test.skip('creates a new support ticket', async ({ page }) => {
    // Requires: authenticated customer session
    // 1. Navigate to /dashboard/support/new
    // 2. Fill form: type=petition, subject, description
    // 3. Submit
    // 4. Verify redirect to support list
    await page.goto('/dashboard/support/new');
    await page.selectOption('select[name="type"]', 'petition');
    await page.fill('input[name="subject"]', 'Test petition subject');
    await page.fill('textarea[name="description"]', 'This is a test petition description for E2E testing.');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/dashboard/support');
    expect(page.url()).toContain('/dashboard/support');
  });

  test.skip('displays ticket list with status badges', async ({ page }) => {
    // Requires: authenticated customer with existing tickets
    await page.goto('/dashboard/support');
    await expect(page.locator('table, [role="list"]')).toBeVisible();
  });
});

test.describe('AI Chat — DB-dependent', () => {
  // These tests require an authenticated session and AI provider API keys.
  // To run: set TURSO_URL, GROQ_API_KEY or CEREBRAS_API_KEY, and auth cookie.

  test.skip('sends a message and receives AI response via SSE', async ({ page }) => {
    // Requires: authenticated session + AI provider keys
    await page.goto('/dashboard/support/chat');
    await page.fill('input[name="message"], textarea[name="message"]', 'Hello, I need help');
    await page.click('button[type="submit"]');
    // Wait for SSE response to render
    await expect(page.locator('[data-role="assistant"]')).toBeVisible({ timeout: 15000 });
  });
});
