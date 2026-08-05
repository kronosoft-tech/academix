import { defineAction, ActionError } from 'astro:actions';
import { z } from 'astro/zod';
import { db } from '../lib/db';
import { sendEmail } from '../lib/email';

const SITE_URL = import.meta.env.SITE_URL || 'http://localhost:4321';

export const passwordReset = defineAction({
  accept: 'form',
  input: z.object({
    email: z.string().email('Ingresa un email válido'),
  }),
  handler: async (input) => {
    const result = await db.execute({
      sql: 'SELECT id FROM users WHERE email = ?',
      args: [input.email],
    });

    // Always return success to prevent email enumeration
    if (result.rows.length === 0) {
      return { success: true, message: 'Si el email existe, recibirás un enlace de restablecimiento' };
    }

    const userId = result.rows[0].id as string;
    const id = crypto.randomUUID();
    const token = crypto.randomUUID();
    const expiresAt = new Date(Date.now() + 60 * 60 * 1000).toISOString(); // 1 hour

    await db.execute({
      sql: `INSERT INTO password_resets (id, user_id, token, expires_at)
            VALUES (?, ?, ?, ?)`,
      args: [id, userId, token, expiresAt],
    });

    // Send reset email (best-effort — don't fail the action if email fails)
    await sendEmail({
      to: input.email,
      subject: 'Academix — Restablecer contraseña',
      html: `
        <h2>Restablece tu contraseña</h2>
        <p>Haz clic en el siguiente enlace para restablecer tu contraseña:</p>
        <p><a href="${SITE_URL}/auth/reset-password?token=${token}">Restablecer contraseña</a></p>
        <p>Este enlace expira en 1 hora.</p>
        <p>Si no solicitaste este cambio, ignora este correo.</p>
      `,
    });

    return { success: true, message: 'Si el email existe, recibirás un enlace de restablecimiento' };
  },
});
