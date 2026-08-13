import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

/**
 * Registration → per-user Turso DB provisioning → dashboard (R2/R4/R8).
 *
 * This spec is gated on the real Turso platform env vars: without
 * TURSO_API_TOKEN / TURSO_ORG / TURSO_GROUP the register action fails closed
 * by design (R6/D6), so the flow cannot complete. When the gate is not met
 * the whole suite is skipped — same env-gating style as the DB-dependent
 * tests in auth.spec.ts.
 *
 * Real side effects per run: a unique per-user Turso database plus rows in
 * the shared DB (user + user_databases + trial). The unique email and
 * academyName keep runs from colliding; failed runs get a best-effort DB
 * DELETE from the register action itself (R5).
 */

const PROVISIONING_VARS = ['TURSO_API_TOKEN', 'TURSO_ORG', 'TURSO_GROUP'] as const;

/**
 * Playwright does not auto-load web/.env (no dotenv dependency in this
 * package), so read the provisioning vars from it explicitly — the gate then
 * reflects exactly what the Astro dev server sees (Vite loads .env itself).
 */
function loadProvisioningEnv(): void {
  for (const name of PROVISIONING_VARS) {
    if (process.env[name]) continue;
    try {
      const envPath = fileURLToPath(new URL('../../.env', import.meta.url));
      const lines = readFileSync(envPath, 'utf8').split('\n');
      for (const line of lines) {
        const match = line.match(/^([A-Z0-9_]+)=(.*)$/);
        if (match && match[1] === name && !process.env[name]) {
          process.env[name] = match[2].replace(/^["']|["']$/g, '');
        }
      }
    } catch {
      // .env missing — leave vars unset so the skip gate below applies.
    }
  }
}

loadProvisioningEnv();

const hasProvisioningEnv = PROVISIONING_VARS.every((name) =>
  Boolean(process.env[name])
);

const uniqueSuffix =
  Date.now().toString(36) + Math.random().toString(36).slice(2, 6);

test.describe('Registration — per-user DB provisioning', () => {
  // Gated on the real Turso env (R6/D6): without these vars the register
  // action fails closed, so the flow cannot complete. Same env-gating style
  // as the DB-dependent tests in auth.spec.ts.
  test.skip(
    !hasProvisioningEnv,
    'requires TURSO_API_TOKEN/TURSO_ORG/TURSO_GROUP — skipped without real Turso env'
  );

  test('registers with academyName + confirmPassword and loads the dashboard via getUserDb', async ({
    page,
  }) => {
      test.setTimeout(180_000);

      const academyName = `E2E Academy ${uniqueSuffix}`;
      const email = `e2e-provisioning-${uniqueSuffix}@example.com`;
      const password = 'SecurePass123!';

      await page.goto('/auth/register');

      await page.getByLabel('Nombre completo').fill('E2E Registration User');
      await page.getByLabel('Email').fill(email);
      await page.getByLabel('Contraseña', { exact: true }).fill(password);
      await page.getByLabel('Nombre de la academia').fill(academyName);
      await page.getByLabel('Confirmar contraseña').fill(password);

      await page.getByRole('button', { name: 'Crear cuenta' }).click();

      // Provisioning (create DB + full-access token + 20 migrations over the
      // network) can exceed the default navigation timeout on cold starts.
      await page.waitForURL('**/dashboard', { timeout: 120_000 });
      expect(page.url()).toContain('/dashboard');

      // The dashboard renders stats from the per-user DB via getUserDb() —
      // absence of the "Error de conexión" block proves the JWT dbUrl/dbToken
      // claims connect to the freshly provisioned database (R4/R8).
      await expect(
        page.getByRole('heading', { name: 'Panel de Control' })
      ).toBeVisible();
      await expect(page.getByText('Error de conexión')).toHaveCount(0);
    });
  }
);
