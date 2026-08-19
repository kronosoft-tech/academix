# Design: Blog + SEO + Indexing + Performance (blog-web)

## Context

`web/` (Astro 7, `output: 'server'` + Vercel adapter, Tailwind v4 via `@tailwindcss/vite`, React/MUI islands) has no blog, no SEO metadata, no `site` config, no robots/sitemap/llms.txt, no `web/public/` (both layouts reference `/favicon.svg` → 404), dead `prose` classes, and a single `<LandingPage client:load />` MUI island on home. Verified patterns to replicate: `content.config.ts` glob+Zod collections (`tutorials`, `faq`); prerendered list/detail (`tutorials/index.astro`, `tutorials/[...slug].astro` with `getStaticPaths` + `render()`); `PUBLIC_ROUTES` middleware (`web/src/middleware.ts`); `SITE_URL` env with `localhost:4321` fallback (already used in 4 server modules). No `openspec/config.yaml` exists → no `rules.design` to apply.

## Goals

1. Ship a Spanish, conversion-oriented blog (pillar + 10 posts) as pure prerendered Astro — zero client JS.
2. Add full-site SEO metadata (canonical, robots, OG, Twitter, JSON-LD) via one `Seo.astro`.
3. Ship indexing/discovery artifacts: `site`, sitemap, robots.txt, llms.txt, `.md` routes.
4. Reduce home-page hydration JS measurably, no visual redesign.

## Technical Approach

Mirror the tutorials pattern exactly: `blog` collection (glob + Zod) → prerendered `blog/index.astro` + `blog/[...slug].astro`; markdown is the single source of truth, also served as `.md` routes, listed in llms.txt, and auto-included in the sitemap (no drift). `Seo.astro` consumed by `Base.astro` and `index.astro`. robots.txt/llms.txt as **prerendered `.ts` endpoints** (build-time generated from env + collection — a static `public/` file would drift or hardcode the domain). Home: split the one island into per-section islands with lazy directives.

## Architecture Decisions

| # | Decision | Options considered | Choice & rationale |
|---|----------|-------------------|--------------------|
| D1 | Blog via content collections, repo-authored | CMS/DB blog, live collections | Glob+Zod matches tutorials; type-safe, build-validated (spec R1), prerendered (fastest, sitemap auto-covers), zero-JS. CMS rejected: new auth/CRUD surface, SSR cost, unnecessary for starter blog. |
| D2 | `site` + sitemap from `SITE_URL` | Hardcoded domain; static sitemap | `site: process.env.SITE_URL \|\| 'http://localhost:4321'` in `astro.config.mjs` + `@astrojs/sitemap`. Env-driven per spec R7; build fails if sitemap runs with no `site` (fallback satisfies dev R7 scenario). |
| D3 | robots.txt + llms.txt as prerendered endpoints | Static `public/` files | `pages/robots.txt.ts` + `pages/llms.txt.ts`, `prerender = true`, return `Response` with `text/plain`. robots needs env (`Sitemap:` URL); llms.txt needs the collection (post list) → static files would drift. Zero extra deps. |
| D4 | `.md` routes via `pages/blog/[slug].md.ts` endpoint | Content negotiation; UA-sniffing | Prerendered endpoint: `getStaticPaths()` from collection, `GET` returns `entry.body` with `Content-Type: text/markdown`. Same source → no drift. No UA sniffing (cloaking, spec R4). Multi-dot filename → route `/blog/:slug.md` (Astro strips only the last extension, e.g. `contact.md.astro` → `/contact.md`). |
| D5 | `.md` Content-Type guaranteed via `vercel.json` headers | Trust adapter default | Vercel may serve `.md` static files as `application/octet-stream`. Add `headers` rule `/blog/(.*)\.md` → `Content-Type: text/markdown; charset=utf-8`. Deterministic regardless of adapter behavior. |
| D6 | Typography via CSS-first plugin | `tailwind.config.js` (v3 way) | Tailwind v4: add `@plugin "@tailwindcss/typography";` to `web/src/styles/global.css` (only line today is `@import "tailwindcss";`). Fixes dead `prose` on FAQ/tutorials/blog. |
| D7 | TOC: rehype-slug heading ids + regex extractor | Curated frontmatter TOC; manual anchors | `markdown.rehypePlugins: [rehypeSlug]` (via `@astrojs/markdown-remark`, bundled with Astro) + `src/lib/headings.ts` (regex over `entry.body`, GitHub slugger) → deterministic jump links. Pillar TOC is curated section list linking to cluster posts (HubSpot anatomy). |
| D8 | Cluster→pillar backlink derived from collection | `pillar` frontmatter field | Find the single `isPillar: true` entry at build; render "Volver al pilar" block on posts. No schema addition; multi-pillar later = add optional field then. |
| D9 | Lead magnet = downloadable `.md` in `web/public/guias/` | PDF asset | No PDF tooling in repo; `.md` is a valid downloadable asset (spec R6), consistent with LLM direction. Content task. |
| D10 | Home perf: per-section islands, lazy directives | Keep single island; rewrite in pure Astro | Split `LandingPage.tsx` into its exported sections rendered directly in `index.astro`; each section self-contained with its own `ThemeProvider` (emotion context per root). Navbar/Hero `client:idle` (above fold, hydration after load); SocialProof/Features/Stats/Testimonials/CTA/Footer `client:visible`. Vite code-splits per component; recharts chunk defers. No redesign (spec R6). |
| D11 | Measurable JS baseline: build-time script | Ad-hoc DevTools numbers | `web/scripts/measure-home-js.mjs` parses `dist/index.html`, sums bytes of referenced `_astro/*.js` chunks → committed `web/perf-baseline.json`. Acceptance: post-change bytes < baseline AND ≤ 0.7× baseline (≥30% cut). Blog pages: zero module scripts. |

## Data Flow

```
content/blog/*.md ──> content.config.ts (glob+Zod, build fails on bad frontmatter)
      │
      ├──> blog/index.astro ────────> /blog            (prerendered, newest-first, pillar card)
      ├──> blog/[...slug].astro ────> /blog/{slug}     (prerendered: Content, TOC, related, CTA,
      │                                link rel=alternate text/markdown)
      ├──> blog/[slug].md.ts ───────> /blog/{slug}.md  (prerendered text/markdown endpoint)
      ├──> llms.txt.ts ─────────────> /llms.txt        (H1 + blockquote + post links)
      └──> @astrojs/sitemap ────────> sitemap-index.xml (only prerendered routes)
Seo.astro ── SITE_URL + Astro.url.pathname ──> canonical/OG/Twitter/JSON-LD in every <head>
robots.txt.ts ── SITE_URL ──> robots.txt (allow all incl. AI bots, disallow /api /dashboard /admin)
```

## Routing Table

| Route | File | Prerender | Notes |
|-------|------|-----------|-------|
| `/blog` | `web/src/pages/blog/index.astro` | true | Listing, grouped, pillar linked at top |
| `/blog/{slug}` | `web/src/pages/blog/[...slug].astro` | true | Post or pillar; 404 for unknown/draft |
| `/blog/{slug}.md` | `web/src/pages/blog/[slug].md.ts` | true | Endpoint, `text/markdown`, raw `entry.body` |
| `/robots.txt` | `web/src/pages/robots.txt.ts` | true | Endpoint, env-aware `Sitemap:` |
| `/llms.txt` | `web/src/pages/llms.txt.ts` | true | Endpoint, generated from published posts |
| `/` | `web/src/pages/index.astro` (modified) | true | Seo head + island split |
| All Base pages | existing | — | `Base.astro` gains Seo + fonts + blog nav |

Middleware (`web/src/middleware.ts`): add `'/blog'` to `PUBLIC_ROUTES` and `pathname.startsWith('/blog/')` to `isPublicRoute`. Prerendered routes already bypass middleware (`context.isPrerendered`), so this is hygiene per spec R7.

## Content Collection Schema (spec R1/R2)

```ts
const blog = defineCollection({
  loader: glob({ pattern: '**/*.md', base: './src/content/blog' }),
  schema: z.object({
    title: z.string().min(1),
    description: z.string(),
    pubDate: z.coerce.date(),
    author: z.string(),
    tags: z.array(z.string()),
    isPillar: z.boolean().default(false),
    updatedDate: z.coerce.date().optional(),
    draft: z.boolean().default(false),
    coverImage: z.string().optional(),
  }),
});
// export const collections = { tutorials, faq, blog };
```

Slug = `entry.id` (filename). Drafts filtered in listing, `getStaticPaths`, and llms.txt → no route → auto-404 and auto-absent from sitemap.

## SEO Metadata Mapping (spec seo R1)

| Page | canonical / OG url | OG type | JSON-LD | robots |
|------|--------------------|---------|---------|--------|
| `/` | `${SITE_URL}/` | website | WebSite + Organization | `index,follow,max-image-preview:large` |
| `/blog` + all Base pages | `${SITE_URL}${pathname}` | website | WebSite | same |
| `/blog/{slug}` | `${SITE_URL}/blog/{slug}` | article | BlogPosting (title, description, pubDate, author, url, tags) | same |

`Seo.astro` props: `title, description?, type?: 'website'|'article', image? (default /og-default.png), publishedTime?, updatedTime?, author?, tags?`. `og:locale = es_ES`. Uses `src/lib/site.ts` (`SITE_URL` + `absoluteUrl(path)`, trailing-slash-safe, same fallback as existing modules).

## Indexing Artifacts — Content Sketch

**robots.txt** (generated): `User-agent: *` → `Allow: /`; explicit `User-agent: GPTBot / ClaudeBot / PerplexityBot` → `Allow: /`; `Disallow: /api/`, `Disallow: /dashboard`, `Disallow: /admin`; `Sitemap: {SITE_URL}/sitemap-index.xml`.

**llms.txt** (generated, llmstxt.org format): `# Academix` / blockquote product summary / `## Blog` / `- [Title]({SITE_URL}/blog/{slug})` per published post (pillar first).

**Post head** additionally: `<link rel="alternate" type="text/markdown" href="/blog/{slug}.md">`.

## File Changes

| File | Action | Responsibility |
|------|--------|----------------|
| `web/src/content.config.ts` | Modify | Add `blog` collection (schema above) |
| `web/src/content/blog/*.md` | Create | Pillar + 10 posts (Appendix) |
| `web/src/pages/blog/index.astro` | Create | Prerendered listing, newest-first, pillar card |
| `web/src/pages/blog/[...slug].astro` | Create | Article: `Content` in `prose`, TOC, related (shared tag, ≤3), CTA block, md alternate link; pillar variant: jump TOC + subtopic link-outs |
| `web/src/pages/blog/[slug].md.ts` | Create | `text/markdown` endpoint from `entry.body` |
| `web/src/components/Seo.astro` | Create | Full head metadata + JSON-LD |
| `web/src/lib/site.ts` | Create | `SITE_URL`, `absoluteUrl()` |
| `web/src/lib/headings.ts` | Create | Regex heading extractor + GitHub slugger |
| `web/src/components/blog/CtaBlock.astro` | Create | Trial CTA (`/auth/register`) + guide CTA (`/guias/guia-gestion-academica.md`) |
| `web/src/pages/robots.txt.ts`, `llms.txt.ts` | Create | Prerendered text endpoints |
| `web/public/favicon.svg` | Create | Fixes existing 404 on both layouts |
| `web/public/og-default.png` | Create | 1200×630 branded social card |
| `web/public/guias/guia-gestion-academica.md` | Create | Lead magnet (Spanish) |
| `web/scripts/measure-home-js.mjs` + `web/perf-baseline.json` | Create | JS-size baseline & acceptance gate |
| `web/src/middleware.ts` | Modify | `PUBLIC_ROUTES` + `/blog/` prefix |
| `web/src/layouts/Base.astro` | Modify | `<Seo/>`, Inter fonts (move from index), Blog nav + footer links |
| `web/src/pages/index.astro` | Modify | `<Seo/>`, replace `<LandingPage client:load/>` with per-section islands |
| `web/src/components/landing/{8 sections}.tsx` | Modify | Each wraps root in `ThemeProvider` (self-contained island) |
| `web/src/components/landing/LandingPage.tsx` | Delete | Composition moved to `index.astro` |
| `web/src/styles/global.css` | Modify | `@plugin "@tailwindcss/typography";` |
| `web/astro.config.mjs` | Modify | `site`, `sitemap()`, `rehypeSlug` |
| `web/package.json` | Modify | +`@astrojs/sitemap`, +`@tailwindcss/typography` |
| `web/src/.env.example` | Modify | Refresh `SITE_URL` line (document real domain; keep placeholder value but note fallback) |
| `web/vercel.json` | Modify | `headers` rule: `/blog/(.*)\.md` → `text/markdown` |

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit (vitest) | `headings.ts` extractor/slugger; `site.ts` URL join; blog Zod schema rejects missing `pubDate`/accepts `isPillar` default | Pure fn tests, node env |
| Build gate | Sitemap contains `/blog` + all slugs; `.md` files emitted; robots/llms content; blog HTML has zero `_astro` script refs | Extend `bun run build` CI step: assert `dist/` artifacts |
| Perf gate | Home JS bytes | `measure-home-js.mjs`: post < baseline, ≤0.7× baseline |
| E2E (Playwright, `web/tests/e2e/blog.spec.ts`) | `/blog` 200 + posts newest-first; post 200 + TOC + CTA + `link alternate`; unknown slug 404; `{slug}.md` 200 `text/markdown`; robots disallows `/api/`; llms.txt lists pillar; anonymous `/blog` passes; home hero visible (no redesign regression) | `page.request` for headers; `page.goto` for pages |
| Visual regression (manual) | FAQ + tutorials prose now styled | Screenshot compare pre/post; no automation infra exists |

## Threat Matrix

| Boundary | Applicability | Reason |
|----------|---------------|--------|
| Documentation-like paths | N/A | No shell/executable handling |
| Git repository selection | N/A | No VCS automation |
| Commit state | N/A | No git commands |
| Push state | N/A | No git commands |
| PR commands | N/A | No PR automation |

Web middleware change is additive public-route allow-listing on prerendered content only — no auth-boundary or process-integrity surface. Not in the matrix's boundary set; no manufactured tasks.

## Migration / Rollout

No data migration. Additive only; rollback = revert commit (collection+pages drop routes/sitemap automatically). **Pre-ship checklist**: confirm the real Vercel deployment domain; set `SITE_URL` in Vercel env; run build gate + perf gate; visual-check FAQ/tutorials and home.

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| **HIGH: wrong `SITE_URL`** → broken canonicals/sitemap/robots | High | Env-driven everywhere; localhost fallback for dev; pre-ship confirmation step (open question); no hardcoded domain |
| `.md` Content-Type on Vercel | Med | `vercel.json` headers rule; verify post-deploy |
| rehype-slug not honored under Sätteri (Astro 7 default processor) | Med | Fallback: explicit `id` attributes in markdown headings; TOC ids must match |
| Typography changes existing prose pages | Low | Visual check FAQ/tutorials |
| Island split breaks home styling (emotion context) | Low | Per-island `ThemeProvider`; E2E sanity on home |
| `llms.txt`/`.md` are bets (no LLM commits) | Med | Near-zero cost; positioned as human-initiated AI value, not a hard KPI |

## Open Questions

- [ ] **BLOCKER (pre-ship only, not design-blocking):** confirm the real Vercel `SITE_URL` production domain before deploy.

## Appendix — Seed Content (Spanish)

**Pillar** — `academix-como-producto.md` — "Academix como producto: la guía completa de gestión académica para academias" (`isPillar: true`; intro, jump TOC, section per subtopic linking to each post below, CTA block).

| Slug | Title | Tags |
|------|-------|------|
| `que-es-un-sistema-de-gestion-academica` | Qué es un sistema de gestión académica (SGA) y por qué tu academia lo necesita | fundamentos, productividad |
| `matriculacion-digital` | Matriculación digital: cómo reducir la fricción al inscribir estudiantes | matriculación, estudiantes |
| `gestion-de-pagos-y-cobros-recurrentes` | Gestión de pagos y cobros recurrentes para academias | pagos, finanzas |
| `asistencia-y-control-diario` | Control de asistencia: digitaliza el registro diario de tus estudiantes | asistencia, operación |
| `reportes-e-indicadores-academicos` | Reportes académicos: las métricas que toda academia debería seguir | reportes, datos |
| `cursos-horarios-y-grupos` | Cómo organizar cursos, horarios y grupos sin caos administrativo | cursos, horarios |
| `roles-y-permisos-para-equipos` | Roles y permisos: quién debe ver qué en tu academia | seguridad, equipo |
| `retencion-de-estudiantes` | Retención de estudiantes: reduce la deserción con datos | retención, datos |
| `migrar-de-excel-a-un-sga` | De Excel a un SGA: cómo migrar la administración de tu academia | migración, fundamentos |
| `automatizacion-de-recordatorios` | Automatiza recordatorios y notificaciones para no perder un pago | pagos, automatización |

All posts: Spanish copy, `author: "Equipo Academix"`, `tags` from table, `draft: false`, `pubDate` staggered 2026; each links back to the pillar (D8); pillar contains the lead-magnet + trial CTAs (R6).