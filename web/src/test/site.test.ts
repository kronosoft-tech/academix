import { describe, it, expect, vi, beforeEach } from 'vitest';

// site.ts captures SITE_URL at module load; reset the module registry so each
// test can control process.env.SITE_URL and re-import it.
describe('site URL helpers', () => {
  beforeEach(() => {
    vi.resetModules();
    delete process.env.SITE_URL;
  });

  it('falls back to localhost:4321 when SITE_URL is unset', async () => {
    const { SITE_URL } = await import('../lib/site');
    expect(SITE_URL).toBe('http://localhost:4321');
  });

  it('reads SITE_URL from the environment and strips trailing slashes', async () => {
    process.env.SITE_URL = 'https://academix.example.com/';
    const { SITE_URL } = await import('../lib/site');
    expect(SITE_URL).toBe('https://academix.example.com');
  });

  it('absoluteUrl joins the site URL with a leading-slash path', async () => {
    process.env.SITE_URL = 'https://academix.example.com';
    const { absoluteUrl } = await import('../lib/site');
    expect(absoluteUrl('/blog')).toBe('https://academix.example.com/blog');
  });

  it('absoluteUrl normalizes a path without leading slash', async () => {
    process.env.SITE_URL = 'https://academix.example.com';
    const { absoluteUrl } = await import('../lib/site');
    expect(absoluteUrl('blog/que-es-un-sga')).toBe('https://academix.example.com/blog/que-es-un-sga');
  });

  it('absoluteUrl strips trailing slashes from the path', async () => {
    process.env.SITE_URL = 'https://academix.example.com';
    const { absoluteUrl } = await import('../lib/site');
    expect(absoluteUrl('/blog/')).toBe('https://academix.example.com/blog');
  });

  it('absoluteUrl uses the localhost fallback in development', async () => {
    const { absoluteUrl } = await import('../lib/site');
    expect(absoluteUrl('/pricing')).toBe('http://localhost:4321/pricing');
  });
});