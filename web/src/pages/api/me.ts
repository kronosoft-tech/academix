export const prerender = false;

import type { APIRoute } from 'astro';
import { getFullTokenPayload } from '../../lib/auth';

export const GET: APIRoute = async ({ cookies }) => {
  const payload = await getFullTokenPayload(cookies);

  if (!payload) {
    return new Response(JSON.stringify({ authenticated: false }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  return new Response(
    JSON.stringify({
      authenticated: true,
      name: payload.academyName,
      email: payload.email,
      role: payload.role,
    }),
    {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    }
  );
};
