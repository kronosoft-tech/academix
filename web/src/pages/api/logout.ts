export const prerender = false;

import type { APIRoute } from 'astro';
import { clearAuthCookie } from '../../lib/auth';

export const POST: APIRoute = async ({ cookies, redirect }) => {
  clearAuthCookie(cookies);
  return redirect('/', 302);
};
