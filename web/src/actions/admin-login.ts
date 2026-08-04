import { defineAction, ActionError } from 'astro:actions';
import { z } from 'astro/zod';
import { db } from '../lib/db';
import { verifyPassword, signToken, setAuthCookie } from '../lib/auth';

export const adminLogin = defineAction({
  accept: 'form',
  input: z.object({
    email: z.string().email('Ingresa un email válido'),
    password: z.string().min(1, 'La contraseña es requerida'),
  }),
  handler: async (input, context) => {
    const result = await db.execute({
      sql: 'SELECT id, email, password_hash, name, role, is_active FROM web_admins WHERE email = ?',
      args: [input.email],
    });

    if (result.rows.length === 0) {
      throw new ActionError({
        code: 'UNAUTHORIZED',
        message: 'Credenciales inválidas',
      });
    }

    const admin = result.rows[0];

    if (!admin.is_active) {
      throw new ActionError({
        code: 'UNAUTHORIZED',
        message: 'Cuenta desactivada',
      });
    }

    const isValid = await verifyPassword(
      input.password,
      admin.password_hash as string
    );

    if (!isValid) {
      throw new ActionError({
        code: 'UNAUTHORIZED',
        message: 'Credenciales inválidas',
      });
    }

    const token = await signToken({
      sub: admin.id as string,
      email: admin.email as string,
      role: admin.role as string,
      type: 'admin',
    });

    setAuthCookie(context.cookies, token);

    return { success: true };
  },
});
