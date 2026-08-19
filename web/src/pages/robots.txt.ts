import type { APIRoute } from 'astro';
import { SITE_URL } from '../lib/site';

export const prerender = true;

const ROBOTS_TXT = [
  'User-agent: *',
  'Allow: /',
  '',
  'User-agent: GPTBot',
  'Allow: /',
  '',
  'User-agent: ClaudeBot',
  'Allow: /',
  '',
  'User-agent: PerplexityBot',
  'Allow: /',
  '',
  'Disallow: /api/',
  'Disallow: /dashboard',
  'Disallow: /admin',
  '',
  `Sitemap: ${SITE_URL}/sitemap-index.xml`,
  '',
].join('\n');

export const GET: APIRoute = async () => {
  return new Response(ROBOTS_TXT, {
    headers: { 'Content-Type': 'text/plain; charset=utf-8' },
  });
};