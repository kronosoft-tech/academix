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
    const result = await db.execute({
      sql: 'SELECT id, email, password_hash, name, role FROM users WHERE email = ?',
      args: [input.email],
    });

    if (result.rows.length === 0) {
      throw new ActionError({
        code: 'UNAUTHORIZED',
        message: 'Credenciales inválidas',
      });
    }

    const user = result.rows[0];
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

    const token = await signToken({
      sub: user.id as string,
      email: user.email as string,
      role: user.role as string,
      type: 'customer',
    });

    setAuthCookie(context.cookies, token);

    return { success: true };
  },
});
