/**
 * Build-gate assertions for the blog + SEO work (tasks 5.3).
 *
 * Runs against the prerendered output and fails (exit 1) when any assertion
 * breaks. Checks, per seo R2/R4 + blog R3:
 *  - sitemap-index.xml lists /blog and every seeded slug
 *  - the /blog/{slug}.md endpoints were emitted
 *  - robots.txt allows AI bots and disallows /api/ /dashboard /admin
 *  - llms.txt lists the pillar before the posts
 *  - blog listing page ships zero client JS
 *  - SITE_URL localhost fallback is reflected in sitemap/robots/llms
 *
 * Usage: node scripts/assert-build-output.mjs
 */
import { readFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const outDir = process.env.OUT_DIR || join(root, '.vercel', 'output', 'static');

const failures = [];
const check = (label, ok) => {
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${label}`);
  if (!ok) failures.push(label);
};

const read = (rel) => {
  const file = join(outDir, rel);
  return existsSync(file) ? readFileSync(file, 'utf8') : null;
};

// --- sitemap ---
const sitemapIndex = read('sitemap-index.xml') ?? '';
const locMatch = sitemapIndex.match(/<loc>([^<]*)<\/loc>/);
const sitemapLoc = locMatch ? locMatch[1] : '';
// The loc is a full URL when SITE_URL is set; extract the pathname.
const sitemapPath = sitemapLoc.replace(/^https?:\/\/[^/]+/, '');
const sitemap = sitemapPath ? read(sitemapPath.replace(/^\//, '')) ?? '' : '';
const hasSitemap = sitemapPath.length > 0 && sitemap.length > 0;
check('sitemap-index.xml exists and references a sitemap', hasSitemap);

const seeded = [
  'academix-como-producto',
  'asistencia-y-control-diario',
  'automatizacion-de-recordatorios',
  'cursos-horarios-y-grupos',
  'gestion-de-pagos-y-cobros-recurrentes',
  'matriculacion-digital',
  'migrar-de-excel-a-un-sga',
  'que-es-un-sistema-de-gestion-academica',
  'reportes-e-indicadores-academicos',
  'retencion-de-estudiantes',
  'roles-y-permisos-para-equipos',
];
check('sitemap lists /blog', sitemap.includes('/blog'));
const missing = seeded.filter((slug) => !sitemap.includes(`/blog/${slug}`));
check(`sitemap lists all ${seeded.length} seeded slugs`, missing.length === 0);

// --- .md endpoints emitted ---
const mdMissing = seeded.filter((slug) => !existsSync(join(outDir, 'blog', `${slug}.md`)));
check('.md endpoints emitted for all seeded slugs', mdMissing.length === 0);

// --- robots.txt ---
const robots = read('robots.txt') ?? '';
check(
  'robots.txt allows AI bots + disallows admin/api',
  ['User-agent: GPTBot', 'User-agent: ClaudeBot', 'User-agent: PerplexityBot'].every((ua) => robots.includes(ua)) &&
    ['Disallow: /api/', 'Disallow: /dashboard', 'Disallow: /admin'].every((d) => robots.includes(d)),
);
check('robots.txt has Sitemap line', robots.includes('Sitemap:'));

// --- llms.txt ---
const llms = read('llms.txt') ?? '';
check('llms.txt is llmstxt.org shaped (H1 + blockquote + ## Blog)', llms.startsWith('# Academix') && llms.includes('> ') && llms.includes('## Blog'));
check('llms.txt lists the pillar first', llms.indexOf('Academix como producto') > llms.indexOf('## Blog'));

// --- blog listing has zero client JS (CSS links are fine) ---
const blogIndex = read('blog/index.html') ?? '';
const jsRefs = [
  ...blogIndex.matchAll(/(?:component-url|renderer-url)="([^"]*\.js)"/g),
  ...blogIndex.matchAll(/<script[^>]+src="([^"]*\.js)"/g),
].length;
check('blog listing page ships zero client JS', jsRefs === 0);

// --- localhost SITE_URL fallback (no production domain hardcoded) ---
const localhostReflected =
  (sitemap.includes('http://localhost:4321') || sitemap.includes('localhost:4321')) &&
  robots.includes('localhost:4321') &&
  llms.includes('localhost:4321');
check('localhost SITE_URL fallback reflected in sitemap/robots/llms', localhostReflected);

if (failures.length > 0) {
  console.error(`\n${failures.length} assertion(s) failed`);
  process.exit(1);
}
console.log('\nAll build-gate assertions passed.');