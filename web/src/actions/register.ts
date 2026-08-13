import { defineAction, ActionError, type ActionAPIContext } from 'astro:actions';
import { z } from 'astro/zod';
import { createClient } from '@libsql/client';
import { db } from '../lib/db';
import { hashPassword, signToken, setAuthCookie } from '../lib/auth';
import {
  provisionUser,
  deleteDatabase,
  getTursoEnv,
  ProvisioningError,
  type ProvisionResult,
} from '../lib/provisioning';
import { createTrialSubscription } from '../lib/payments/lifecycle';

/**
 * Registration input contract (R1): `name`, `email`, `password`, `academyName`
 * and `confirmPassword`. The per-user DB slug is derived server-side from
 * `academyName` via `generateDbSlug` — no client-supplied slug field.
 */
export const registerSchema = z
  .object({
    name: z.string().min(2, 'El nombre debe tener al menos 2 caracteres'),
    email: z.string().email('Ingresa un email válido'),
    password: z.string().min(8, 'La contraseña debe tener al menos 8 caracteres'),
    academyName: z
      .string()
      .trim()
      .min(2, 'El nombre de la academia debe tener al menos 2 caracteres'),
    confirmPassword: z.string().min(1, 'Confirma tu contraseña'),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: 'Las contraseñas no coinciden',
    path: ['confirmPassword'],
  });

export type RegisterInput = z.infer<typeof registerSchema>;

/**
 * Register a new academy.
 *
 * Ordering follows R4 (design data flow): email-exists check (CONFLICT before
 * any provisioning) → hash password → provisionUser (env gate → create → token
 * → migrate 001–020) → per-user `users` row (role 'Admin') → shared `users`
 * row → `user_databases` INSERT OR REPLACE (incl. academy_name) → shared trial
 * subscription (R7 — no subscription rows in the per-user DB) → JWT signed
 * with `dbUrl`/`dbToken`/`academyName` (resolves the previous TS2345).
 *
 * Fail closed (R5/R6): on any failure the user gets no JWT and a clear Spanish
 * error; if the per-user DB was already created, a best-effort DELETE is
 * attempted before rethrowing.
 */
export async function registerHandler(
  input: RegisterInput,
  context: ActionAPIContext
): Promise<{ success: true }> {
  // 1. Email existence check — CONFLICT before any provisioning (R4).
  const existing = await db.execute({
    sql: 'SELECT id FROM users WHERE email = ?',
    args: [input.email],
  });

  if (existing.rows.length > 0) {
    throw new ActionError({
      code: 'CONFLICT',
      message: 'Ya existe una cuenta con este email',
    });
  }

  const id = crypto.randomUUID();
  const passwordHash = await hashPassword(input.password);
  const now = new Date().toISOString();

  let provisioned: ProvisionResult | null = null;

  try {
    // 2. Provision the per-user DB: env gate → create → token → migrate (R4/R6).
    provisioned = await provisionUser(input.academyName);

    // 3. Per-user Admin row (R4). The per-user DB belongs to the academy; its
    // Admin is the user registering right now.
    const userClient = createClient({ url: provisioned.dbUrl, authToken: provisioned.dbToken });
    try {
      await userClient.execute({
        sql: `INSERT INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
              VALUES (?, ?, ?, ?, 'Admin', 1, ?, ?)`,
        args: [id, input.email, passwordHash, input.name, now, now],
      });
    } finally {
      try {
        userClient.close();
      } catch {
        // Closing the per-user connection is best-effort bookkeeping only.
      }
    }

    // 4. Shared users row (R4) — control-plane identity.
    await db.execute({
      sql: `INSERT INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, 'Admin', 1, ?, ?)`,
      args: [id, input.email, passwordHash, input.name, now, now],
    });

    // 5. user_databases mapping — INSERT OR REPLACE, includes academy_name (R4).
    const env = getTursoEnv();
    await db.execute({
      sql: `INSERT OR REPLACE INTO user_databases
            (user_id, email, academy_name, db_url, db_token, org, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)`,
      args: [id, input.email, input.academyName, provisioned.dbUrl, provisioned.dbToken, env.org, now],
    });

    // 6. Shared trial subscription only (R4/R7) — the per-user DB receives no
    // subscription/payment rows; the shared DB stays the payment source of truth.
    await createTrialSubscription(id, 'trial', null);

    // 7. JWT with the per-user DB connection claims (R4).
    const token = await signToken({
      sub: id,
      email: input.email,
      role: 'Admin',
      type: 'customer',
      dbUrl: provisioned.dbUrl,
      dbToken: provisioned.dbToken,
      academyName: input.academyName,
    });

    setAuthCookie(context.cookies, token);

    return { success: true };
  } catch (err) {
    // 8. Best-effort cleanup for failures after database creation (R5).
    // provisionUser already DELETEs its own post-create failures; this catch
    // covers the per-user/shared writes, trial creation and signToken.
    if (provisioned) {
      try {
        const env = getTursoEnv();
        await deleteDatabase(env.org, provisioned.dbName);
      } catch {
        // Best-effort only — never mask the original error (fail closed).
      }
    }

    if (err instanceof ProvisioningError) {
      throw mapProvisioningError(err);
    }
    if (err instanceof ActionError) {
      throw err;
    }
    throw new ActionError({
      code: 'INTERNAL_SERVER_ERROR',
      message: 'No se pudo completar el registro, intenta de nuevo',
    });
  }
}

/** Map provisioning failures to user-facing errors (design error matrix). */
function mapProvisioningError(err: ProvisioningError): ActionError {
  switch (err.code) {
    case 'MISSING_ENV':
      // R6/D6: fail closed — never silently degrade to shared-only.
      return new ActionError({
        code: 'INTERNAL_SERVER_ERROR',
        message: 'Registro no disponible temporalmente',
      });
    case 'AUTH':
    case 'MIGRATION':
      // Post-creation failures inside provisionUser — it already DELETEd the DB.
      return new ActionError({
        code: 'INTERNAL_SERVER_ERROR',
        message: 'No se pudo completar el registro, intenta de nuevo',
      });
    case 'CONFLICT':
    case 'HTTP':
    case 'RATE_LIMIT':
    default:
      // createDatabase failures — no DB was created, no cleanup needed.
      return new ActionError({
        code: 'INTERNAL_SERVER_ERROR',
        message: 'No se pudo crear tu academia, intenta de nuevo',
      });
  }
}

export const register = defineAction({
  accept: 'form',
  input: registerSchema,
  handler: registerHandler,
});
