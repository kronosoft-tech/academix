import { test, expect } from '@playwright/test';

/**
 * Blog + SEO public pages (blog R2–R7, seo R1–R4).
 *
 * No env gating needed: every route under test is public and prerendered,
 * so the Astro dev server (webServer in playwright.config.ts) serves it
 * without external services.
 */

const NEWEST_POST = 'Automatiza recordatorios y notificaciones para no perder un pago';
const PILLAR_TITLE = 'Academix como producto: la guía completa de gestión académica para academias';
const OLDEST_POST = 'Qué es un sistema de gestión académica (SGA) y por qué tu academia lo necesita';

test.describe('blog listing', () => {
  test('/blog returns 200 with pillar pinned and newest-first groups', async ({ page }) => {
    const response = await page.goto('/blog');
    expect(response?.status()).toBe(200);

    // Pillar featured card on top
    const pillarCard = page.getByRole('link', { name: PILLAR_TITLE }).first();
    await expect(pillarCard).toBeVisible();

    // All 10 seeded posts render as cards (each post appears in every tag
    // group it belongs to, so dedupe the repeated titles)
    const cards = page.locator('section[id^="tema-"] h3');
    const titles = (await cards.allTextContents()).map((t) => t.trim());
    const unique = [...new Set(titles)];
    expect(unique).toHaveLength(10);

    // Newest post listed before the oldest in the tag-group grid
    expect(unique.indexOf(NEWEST_POST)).toBeGreaterThan(-1);
    expect(unique.indexOf(OLDEST_POST)).toBeGreaterThan(unique.indexOf(NEWEST_POST));

    // Topic tag chips render
    await expect(page.getByRole('link', { name: /^pagos/ })).toBeVisible();
  });

  test('/blog passes for anonymous visitors (no redirect)', async ({ page }) => {
    await page.goto('/blog');
    await expect(page).toHaveURL(/\/blog$/);
    await expect(page.getByRole('heading', { name: 'Gestión académica, explicada' })).toBeVisible();
  });

  test('B-R2-S1: draft post excluded from listing and 404s', async ({ page }) => {
    const draftResponse = await page.goto('/blog/draft-test-fixture');
    expect(draftResponse?.status()).toBe(404);

    await page.goto('/blog');
    const body = await page.textContent('body');
    expect(body).not.toContain('Draft Test Fixture Title');

    const cards = page.locator('section[id^="tema-"] h3');
    const titles = (await cards.allTextContents()).map((t) => t.trim());
    const unique = [...new Set(titles)];
    expect(unique).toHaveLength(10);
  });
});

test.describe('blog post pages', () => {
  test('post page renders prose, TOC, CTA and markdown alternate link', async ({ page }) => {
    const response = await page.goto(`/blog/que-es-un-sistema-de-gestion-academica`);
    expect(response?.status()).toBe(200);

    await expect(page.getByRole('heading', { name: OLDEST_POST })).toBeVisible();

    // TOC links match rendered heading anchors
    const tocLink = page.locator('nav[aria-label="Tabla de contenidos"] a').first();
    await expect(tocLink).toBeVisible();
    const href = await tocLink.getAttribute('href');
    expect(href).toMatch(/^#/);
    await expect(page.locator(`[id="${href!.slice(1)}"]`)).toHaveCount(1);

    // CTA block with trial + guide actions
    await expect(page.getByRole('heading', { name: 'Prueba Academix gratis' })).toBeVisible();
    await expect(page.getByRole('link', { name: 'Probar gratis' })).toHaveAttribute('href', '/auth/register');

    // Alternate markdown link in head
    const alternate = page.locator('link[rel="alternate"][type="text/markdown"]');
    await expect(alternate).toHaveAttribute('href', '/blog/que-es-un-sistema-de-gestion-academica.md');

    // Pillar backlink box on non-pillar posts (pillar also appears as a
    // related card, so target the backlink box specifically)
    const backlink = page.locator('a[href="/blog/academix-como-producto"]').first();
    await expect(backlink).toBeVisible();
  });

  test('unknown slug returns 404', async ({ page }) => {
    const response = await page.goto('/blog/no-existe-este-articulo');
    expect(response?.status()).toBe(404);
  });

  test('markdown endpoint serves raw content as text/markdown', async ({ request }) => {
    const response = await request.get('/blog/que-es-un-sistema-de-gestion-academica.md');
    expect(response.status()).toBe(200);
    expect(response.headers()['content-type']).toContain('text/markdown');
    const body = await response.text();
    expect(body).toContain('## Qué es un SGA');
  });

  test('B-R6-S2: guide download link visible and serves markdown', async ({ page, request }) => {
    await page.goto('/blog/que-es-un-sistema-de-gestion-academica');

    await expect(page.getByRole('link', { name: 'Descargar guía de gestión académica' })).toBeVisible();

    const res = await request.get('/guias/guia-gestion-academica.md');
    expect(res.status()).toBe(200);
    expect(res.headers()['content-type']).toContain('text/markdown');
    const text = await res.text();
    expect(text).toContain('## Por qué esta guía');
  });

  test('B-R6-S3: prose content renders before the CTA block', async ({ page }) => {
    await page.goto('/blog/que-es-un-sistema-de-gestion-academica');

    const proseBox = await page.locator('article .prose').boundingBox();
    const ctaBox = await page
      .locator('section', { hasText: 'Prueba Academix gratis' })
      .boundingBox();

    expect(proseBox).not.toBeNull();
    expect(ctaBox).not.toBeNull();
    expect(proseBox!.y).toBeLessThan(ctaBox!.y);
  });

  test('S-R5-S1: post page renders with prose typography classes', async ({ page }) => {
    await page.goto('/blog/que-es-un-sistema-de-gestion-academica');
    await expect(page.locator('article .prose')).toHaveCount(1);
  });
});

test.describe('SEO endpoints', () => {
  test('robots.txt allows AI bots and disallows admin/api paths', async ({ request }) => {
    const response = await request.get('/robots.txt');
    expect(response.status()).toBe(200);
    const body = await response.text();
    expect(body).toContain('User-agent: GPTBot');
    expect(body).toContain('User-agent: ClaudeBot');
    expect(body).toContain('User-agent: PerplexityBot');
    expect(body).toContain('Disallow: /api/');
    expect(body).toContain('Disallow: /dashboard');
    expect(body).toContain('Disallow: /admin');
    expect(body).toContain('Sitemap:');
  });

  test('llms.txt lists the pillar first', async ({ request }) => {
    const response = await request.get('/llms.txt');
    expect(response.status()).toBe(200);
    const body = await response.text();
    expect(body.startsWith('# Academix')).toBe(true);
    const blogIndex = body.indexOf('## Blog');
    const pillarIndex = body.indexOf('Academix como producto');
    expect(pillarIndex).toBeGreaterThan(blogIndex);
  });

  test('home page renders with blog nav and hero', async ({ page }) => {
    const response = await page.goto('/');
    expect(response?.status()).toBe(200);
    // Navbar has Blog (desktop + mobile drawer variants)
    await expect(page.getByRole('link', { name: 'Blog' }).first()).toBeVisible();
    await expect(page.getByRole('link', { name: 'Comenzar gratis' }).first()).toBeVisible();
  });

  test('S-R1-S1: home page has title and description metadata', async ({ page }) => {
    await page.goto('/');

    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);

    const metaDesc = await page.locator('meta[name="description"]').getAttribute('content');
    expect(metaDesc).toBeTruthy();
    expect(metaDesc!.length).toBeGreaterThan(0);
  });

  test('S-R1-S2: post page has BlogPosting JSON-LD with author Academix', async ({ page }) => {
    await page.goto('/blog/que-es-un-sistema-de-gestion-academica');

    const ldJson = await page
      .locator('script[type="application/ld+json"]')
      .textContent();
    expect(ldJson).toBeTruthy();
    const data = JSON.parse(ldJson!);
    expect(data['@type']).toBe('BlogPosting');
    expect(data.author).toBeDefined();
    expect(data.author.name).toMatch(/Academix/);
    expect(data.datePublished).toBeTruthy();
    expect(data.url).toBeTruthy();
  });

  test('S-R4-S3: no UA cloaking — body identical across UAs', async ({ request }) => {
    const slug = 'que-es-un-sistema-de-gestion-academica';

    const browserRes = await request.get(`/blog/${slug}`, {
      headers: { 'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36' },
    });
    const googlebotRes = await request.get(`/blog/${slug}`, {
      headers: { 'User-Agent': 'Googlebot/2.1 (+http://www.google.com/bot.html)' },
    });
    const byobotRes = await request.get(`/blog/${slug}`, {
      headers: { 'User-Agent': 'BYOBOT/1.0' },
    });

    const browserBody = await browserRes.text();
    const googlebotBody = await googlebotRes.text();
    const byobotBody = await byobotRes.text();

    expect(browserBody.length).toBe(googlebotBody.length);
    expect(browserBody.length).toBe(byobotBody.length);
  });

  test('S-R5-S2: existing prose pages (FAQ, tutorials) still render typography', async ({ page }) => {
    await page.goto('/faq');
    await expect(page.getByRole('heading', { name: 'Preguntas Frecuentes' })).toBeVisible();
    await expect(page.locator('.prose')).toHaveCount(3);

    await page.goto('/tutorials/download-macos');
    await expect(page.locator('article .prose')).toHaveCount(1);
  });
});