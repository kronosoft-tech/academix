import { defineAction, ActionError } from 'astro:actions';
import { z } from 'astro/zod';
import { db } from '../lib/db';

export const pqrs = {
  createTicket: defineAction({
    accept: 'form',
    input: z.object({
      type: z.enum(['petition', 'complaint', 'claim', 'suggestion']),
      subject: z.string().min(3, 'El asunto debe tener al menos 3 caracteres'),
      description: z.string().min(10, 'La descripción debe tener al menos 10 caracteres'),
    }),
    handler: async (input, context) => {
      const user = context.locals.user;
      if (!user) {
        throw new ActionError({
          code: 'UNAUTHORIZED',
          message: 'Debes iniciar sesión',
        });
      }

      const id = crypto.randomUUID();
      const now = new Date().toISOString();

      await db.execute({
        sql: `INSERT INTO pqrs_tickets (id, user_id, type, subject, description, status, created_at, updated_at)
              VALUES (?, ?, ?, ?, ?, 'open', ?, ?)`,
        args: [id, user.id, input.type, input.subject, input.description, now, now],
      });

      return { ticketId: id };
    },
  }),

  updateTicketStatus: defineAction({
    input: z.object({
      ticketId: z.string().uuid(),
      status: z.enum(['in_progress', 'resolved']),
    }),
    handler: async (input, context) => {
      const admin = context.locals.admin;
      if (!admin) {
        throw new ActionError({
          code: 'UNAUTHORIZED',
          message: 'Solo administradores pueden cambiar el estado',
        });
      }

      const now = new Date().toISOString();
      const resolvedAt = input.status === 'resolved' ? now : null;

      await db.execute({
        sql: `UPDATE pqrs_tickets
              SET status = ?, updated_at = ?, resolved_at = COALESCE(?, resolved_at)
              WHERE id = ?`,
        args: [input.status, now, resolvedAt, input.ticketId],
      });

      return { success: true };
    },
  }),
};
