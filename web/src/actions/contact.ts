import { defineAction, ActionError } from 'astro:actions';
import { z } from 'astro/zod';
import { sendEmail } from '../lib/email';

const SUPPORT_EMAIL = import.meta.env.SUPPORT_EMAIL || 'support@academix.app';

export const contact = defineAction({
  accept: 'form',
  input: z.object({
    name: z.string().min(2, 'El nombre debe tener al menos 2 caracteres'),
    email: z.string().email('Ingresa un email válido'),
    message: z.string().min(10, 'El mensaje debe tener al menos 10 caracteres'),
  }),
  handler: async (input) => {
    try {
      // Best-effort delivery — still return success to the user even if email fails
      await sendEmail({
        to: SUPPORT_EMAIL,
        subject: `[Academix Contact] Mensaje de ${input.name}`,
        html: `
          <h2>Nuevo mensaje de contacto</h2>
          <p><strong>Nombre:</strong> ${input.name}</p>
          <p><strong>Email:</strong> ${input.email}</p>
          <p><strong>Mensaje:</strong></p>
          <p>${input.message}</p>
        `,
      });

      return { success: true, message: 'Mensaje enviado correctamente' };
    } catch {
      throw new ActionError({
        code: 'INTERNAL_SERVER_ERROR',
        message: 'No se pudo enviar el mensaje. Intenta de nuevo más tarde.',
      });
    }
  },
});
