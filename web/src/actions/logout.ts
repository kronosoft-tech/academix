import { defineAction } from 'astro:actions';
import { clearAuthCookie } from '../lib/auth';

export const logout = defineAction({
  accept: 'form',
  input: undefined,
  handler: async (_input, context) => {
    clearAuthCookie(context.cookies);
    return { success: true };
  },
});
