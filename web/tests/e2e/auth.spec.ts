import { test, expect } from '@playwright/test';

test.describe('Auth — Public Route Protection', () => {
  test('unauthenticated access to /dashboard redirects to /auth/login', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForURL('**/auth/login');
    expect(page.url()).toContain('/auth/login');
  });

  test('unauthenticated access to /admin redirects to /admin/login', async ({ page }) => {
    await page.goto('/admin');
    await page.waitForURL('**/admin/login');
    expect(page.url()).toContain('/admin/login');
  });

  test('login page renders with email and password fields', async ({ page }) => {
    await page.goto('/auth/login');
    await expect(page.locator('input[type="email"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
  });

  test('register page renders with form fields', async ({ page }) => {
    await page.goto('/auth/register');
    await expect(page.locator('input[type="email"]')).toBeVisible();
    // R1: the registration form has two password inputs (password +
    // confirmPassword) plus academyName — assert each explicitly.
    await expect(page.locator('input[name="password"]')).toBeVisible();
    await expect(page.locator('input[name="confirmPassword"]')).toBeVisible();
    await expect(page.locator('input[name="academyName"]')).toBeVisible();
  });

  test('admin login page renders correctly', async ({ page }) => {
    await page.goto('/admin/login');
    await expect(page.locator('input[type="email"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
  });
});

test.describe('Auth — DB-dependent flows', () => {
  // These tests require a running Turso DB with seeded test data.
  // To run: set TURSO_URL and TURSO_AUTH_TOKEN env vars pointing to a test DB.

  test.skip('register flow creates account and redirects to dashboard', async ({ page }) => {
    // Requires: empty test DB, TURSO_URL set
    await page.goto('/auth/register');
    await page.fill('input[name="name"]', 'Test User');
    await page.fill('input[type="email"]', `test-${Date.now()}@example.com`);
    await page.fill('input[type="password"]', 'SecurePass123!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/dashboard');
    expect(page.url()).toContain('/dashboard');
  });

  test.skip('login with valid credentials redirects to dashboard', async ({ page }) => {
    // Requires: seeded user in test DB
    await page.goto('/auth/login');
    await page.fill('input[type="email"]', 'seeded@example.com');
    await page.fill('input[type="password"]', 'SeededPass123!');
    await page.click('button[type="submit"]');
    await page.waitForURL('**/dashboard');
    expect(page.url()).toContain('/dashboard');
  });

  test.skip('login with invalid credentials shows error', async ({ page }) => {
    // Requires: TURSO_URL set (will attempt auth against real DB)
    await page.goto('/auth/login');
    await page.fill('input[type="email"]', 'nonexistent@example.com');
    await page.fill('input[type="password"]', 'WrongPass123!');
    await page.click('button[type="submit"]');
    await expect(page.locator('[role="alert"], .error')).toBeVisible();
  });

  test.skip('cross-domain access: customer token on /admin returns 403', async ({ page }) => {
    // Requires: authenticated customer session cookie set
    // 1. Login as customer
    // 2. Navigate to /admin
    // 3. Expect 403 or redirect
  });

  test.skip('password reset shows success message regardless of email existence', async ({ page }) => {
    // Requires: TURSO_URL set
    await page.goto('/auth/reset-password');
    await page.fill('input[type="email"]', 'anyone@example.com');
    await page.click('button[type="submit"]');
    await expect(page.getByText(/enlace de restablecimiento/i)).toBeVisible();
  });
});
