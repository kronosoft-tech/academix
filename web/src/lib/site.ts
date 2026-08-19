/**
 * Site URL helpers (seo R1/R7).
 *
 * SITE_URL is env-driven: Vercel sets it to the real deployment domain at
 * build time; local development falls back to http://localhost:4321 when it
 * is unset. Never hardcode a guessed production domain here.
 */
export const SITE_URL: string =
  (process.env.SITE_URL ?? '').replace(/\/+$/, '') || 'http://localhost:4321';

/**
 * Build an absolute URL under SITE_URL from a site path.
 * Trailing-slash-safe: "/blog", "blog", and "/blog/" all normalize.
 */
export function absoluteUrl(path: string): string {
  const normalized = path.startsWith('/') ? path : `/${path}`;
  return `${SITE_URL}${normalized.replace(/\/+$/, '')}`;
}