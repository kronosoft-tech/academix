import { defineAction, ActionError } from 'astro:actions';
import { z } from 'astro/zod';
import { db } from '../lib/db';
import { hashPassword, signToken, setAuthCookie } from '../lib/auth';

export const register = defineAction({
  accept: 'form',
  input: z.object({
    name: z.string().min(2, 'El nombre debe tener al menos 2 caracteres'),
    email: z.string().email('Ingresa un email válido'),
    password: z.string().min(8, 'La contraseña debe tener al menos 8 caracteres'),
  }),
  handler: async (input, context) => {
    // Check if email already exists
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

    await db.execute({
      sql: `INSERT INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, 'Admin', 1, ?, ?)`,
      args: [id, input.email, passwordHash, input.name, now, now],
    });

    const token = await signToken({
      sub: id,
      email: input.email,
      role: 'Admin',
      type: 'customer',
    });

    setAuthCookie(context.cookies, token);

    return { success: true };
  },
});
