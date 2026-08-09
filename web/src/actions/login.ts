import { defineAction, ActionError } from 'astro:actions';
import { z } from 'astro/zod';
import { db } from '../lib/db';
import { verifyPassword, signToken, setAuthCookie } from '../lib/auth';

export const login = defineAction({
  accept: 'form',
  input: z.object({
    email: z.string().email('Ingresa un email válido'),
    password: z.string().min(1, 'La contraseña es requerida'),
  }),
  handler: async (input, context) => {
    // Step 1: Find user in control plane users table
    const userResult = await db.execute({
      sql: 'SELECT id, email, password_hash, name, role FROM users WHERE email = ?',
      args: [input.email],
    });

    if (userResult.rows.length === 0) {
      throw new ActionError({
        code: 'UNAUTHORIZED',
        message: 'Credenciales inválidas',
      });
    }

    const user = userResult.rows[0];

    // Step 2: Verify password against control plane hash
    const isValid = await verifyPassword(
      input.password,
      user.password_hash as string
    );

    if (!isValid) {
      throw new ActionError({
        code: 'UNAUTHORIZED',
        message: 'Credenciales inválidas',
      });
    }

    // Step 3: Resolve user's individual database from user_databases table
    const dbMapping = await db.execute({
      sql: 'SELECT db_url, db_token, academy_name FROM user_databases WHERE email = ?',
      args: [input.email],
    });

    if (dbMapping.rows.length === 0) {
      throw new ActionError({
        code: 'NOT_FOUND',
        message: 'No se encontró la base de datos de tu academia. Contacta soporte.',
      });
    }

    const mapping = dbMapping.rows[0];

    // Step 4: Sign JWT with user info + their DB connection details
    const token = await signToken({
      sub: user.id as string,
      email: user.email as string,
      role: user.role as string,
      type: 'customer',
      dbUrl: mapping.db_url as string,
      dbToken: mapping.db_token as string,
      academyName: mapping.academy_name as string,
    });

    setAuthCookie(context.cookies, token);

    return { success: true };
  },
});
