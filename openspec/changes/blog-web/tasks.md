# Tasks: Blog + SEO + Indexing + Performance (blog-web)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~2,500–3,500 |
| 400-line budget risk | High |
| Session budget (800) | Exceeded — `size:exception` required |
| Chained PRs recommended | No (single-pr-default) |
| Suggested split | Single PR, 6 work units |
| Delivery strategy | single-pr-default |

```text
Decision needed before apply: Yes
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: High
```

### Work Units (commit order, single PR)

| # | Commit | Focused test | Harness | Rollback |
|---|--------|--------------|---------|----------|
| 1 | `chore(web): add blog/SEO foundation deps and config` | `bunx tsc --noEmit` | N/A — config only | Revert config files |
| 2 | `feat(web): seed Spanish blog content (pillar + 10 posts)` | `bun run build` (schema gate) | N/A — content only | Delete `content/blog/`, guide |
| 3 | `feat(web): add prerendered blog listing and post pages` | headings unit test | `bun run dev`; visit `/blog` | Delete `pages/blog/` |
| 4 | `feat(web): add SEO metadata, robots, llms.txt and sitemap` | `bun run build` + assert `dist/` | `bun run dev`; inspect `/blog` head | Revert SEO/nav edits |
| 5 | `perf(web): split home island and cut JS payload` | `node scripts/measure-home-js.mjs` (≥30% cut) | `bun run build`, then measure | Restore `LandingPage.tsx` |
| 6 | `test(web): add blog unit and E2E coverage` | `bun run test` + `bun run test:e2e tests/e2e/blog.spec.ts` | N/A — test only | Remove test files |

## Phase 1: Foundation

- [x] 1.1 Install `@astrojs/sitemap` + `@tailwindcss/typography` in `web/` — seo R2/R5
- [x] 1.2 `web/astro.config.mjs`: `site` from `SITE_URL` (localhost fallback), `sitemap()`, `rehypeSlug` — seo R2, blog R4
- [x] 1.3 Create `web/src/lib/site.ts`: `SITE_URL` + `absoluteUrl()` — seo R1/R7
- [x] 1.4 Create `web/src/lib/headings.ts`: regex extractor + GitHub slugger — blog R4/R5
- [x] 1.5 `web/src/content.config.ts`: add `blog` glob+Zod collection (build fails on violation) — blog R1
- [x] 1.6 `web/src/styles/global.css`: add `@plugin "@tailwindcss/typography";` — seo R5
- [x] 1.7 `web/src/.env.example`: refresh `SITE_URL` + fallback note — seo R7

## Phase 2: Seed Content (Spanish)

- [x] 2.1 Create pillar `web/src/content/blog/academix-como-producto.md` (`isPillar: true`, intro, jump TOC, subtopic link-outs, CTAs) — blog R5/R6
- [x] 2.2 Posts 1–5: `que-es-un-sistema-de-gestion-academica`, `matriculacion-digital`, `gestion-de-pagos-y-cobros-recurrentes`, `asistencia-y-control-diario`, `reportes-e-indicadores-academicos` — author, `draft: false`, staggered `pubDate`, pillar backlink — blog R1/R5
- [x] 2.3 Posts 6–10: `cursos-horarios-y-grupos`, `roles-y-permisos-para-equipos`, `retencion-de-estudiantes`, `migrar-de-excel-a-un-sga`, `automatizacion-de-recordatorios` (same conventions) — blog R1/R5
- [x] 2.4 Create lead magnet `web/public/guias/guia-gestion-academica.md` — blog R6

## Phase 3: Pages, SEO Layer, Routing

- [x] 3.1 Create `web/src/components/Seo.astro`: canonical, robots, OG `es_ES`, Twitter, JSON-LD (WebSite/Organization/BlogPosting) — seo R1
- [x] 3.2 Create `web/src/components/blog/CtaBlock.astro`: trial → `/auth/register`, guide → `/guias/guia-gestion-academica.md` — blog R6
- [x] 3.3 Create `web/src/pages/blog/index.astro`: prerendered, newest-first, pillar card, tag groups, zero client JS, empty state — blog R3
- [x] 3.4 Create `web/src/pages/blog/[...slug].astro`: non-draft `getStaticPaths()`, `prerender = true`; `Content` in `prose`, TOC, related (≤3), CtaBlock, `<link rel="alternate" type="text/markdown">`; pillar variant: jump TOC + link-outs; unknown/draft → 404 — blog R2/R4/R5
- [x] 3.5 Create `web/src/pages/blog/[slug].md.ts`: prerendered endpoint returning `entry.body` as `text/markdown` — seo R4
- [x] 3.6 Create `web/src/pages/robots.txt.ts`: allow all incl. GPTBot/ClaudeBot/PerplexityBot; disallow `/api/` `/dashboard` `/admin`; `Sitemap:` from `SITE_URL` — seo R3
- [x] 3.7 Create `web/src/pages/llms.txt.ts`: llmstxt.org format (H1, blockquote, `## Blog` pillar-first links) — seo R4
- [x] 3.8 Create `web/public/favicon.svg` + `web/public/og-default.png` (1200×630) — seo R1
- [x] 3.9 `web/src/layouts/Base.astro`: add `<Seo/>`, Inter fonts, Blog nav + footer links — blog R7, seo R1
- [x] 3.10 `web/src/middleware.ts`: `'/blog'` in `PUBLIC_ROUTES` + `/blog/` prefix in `isPublicRoute` — blog R7
- [x] 3.11 `web/vercel.json`: headers rule `/blog/(.*)\.md` → `text/markdown; charset=utf-8` — seo R4

## Phase 4: Home Performance

- [x] 4.1 `web/src/pages/index.astro`: add `<Seo/>`; Navbar/Hero `client:idle`, rest `client:visible`; drop `<LandingPage client:load/>` — seo R1/R6
- [x] 4.2 Update 8 `web/src/components/landing/*Section.tsx` + `Navbar.tsx` + `Footer.tsx`: self-contained, own `ThemeProvider` — seo R6
- [x] 4.3 Delete `web/src/components/landing/LandingPage.tsx` — seo R6
- [x] 4.4 Create `web/scripts/measure-home-js.mjs` (sum `_astro/*.js` from `dist/index.html`) + commit `web/perf-baseline.json` — seo R6

## Phase 5: Verification

- [x] 5.1 Unit tests: `headings.ts` slugger; `site.ts` URL join; blog Zod rejects missing `pubDate`, accepts `isPillar` default — blog R1, seo R7
- [x] 5.2 Create `web/tests/e2e/blog.spec.ts`: `/blog` 200 newest-first; post 200 + TOC + CTA + alternate; unknown 404; `.md` 200 `text/markdown`; robots disallows `/api/`; llms lists pillar; anonymous `/blog` passes; home hero — blog R2–R7, seo R1–R4
- [x] 5.3 Build gate: `bun run build` (web/) — assert sitemap lists `/blog` + slugs, `.md` emitted, robots/llms content, zero `_astro` scripts on blog, localhost fallback — seo R2/R4, blog R3
- [x] 5.4 Perf gate: `node web/scripts/measure-home-js.mjs` → < baseline and ≤0.7× baseline — seo R6
- [x] 5.5 `bunx tsc --noEmit` (web/); visual regression: FAQ, tutorials (prose), home (no redesign) — seo R5/R6
## Apply Notes (deviations, recorded during implementation)

- **1.2**: `rehypeSlug` NOT added — Astro 7 Sätteri natively slugs h1–h6 via
  github-slugger; `headings.ts` replicates its semantics (incl. `-1` duplicates)
  so TOC anchors match. Native behavior verified against rendered HTML.
- **4.1–4.3**: Deviated from island-per-section to *static SSR + one island*.
  Only the interactive Navbar hydrates (`client:visible`); hero and all other
  sections render as static HTML with zero client JS. The island-everything
  approach (client:visible on all sections) could not meet the ≤0.7× gate
  (~360KB with MUI per island); static sections + Tailwind Navbar reached
  187,534 bytes (-55.5%). `LandingPage.tsx` retained as the static sections
  wrapper (task 4.3's delete superseded).
- **4.4**: `perf-baseline.json` committed at `web/perf-baseline.json` (as
  specified); measure script at `web/scripts/measure-home-js.mjs` sums
  `astro-island component-url` + `renderer-url` JS refs (the actual load
  mechanism), not `dist/` globbing.
- **5.3**: Build gate codified as `web/scripts/assert-build-output.mjs`
  (`bun run assert:build`) — 10 assertions, all passing.
- **playwright.config.ts**: pinned `ASTRO_DEV_BACKGROUND=1` in webServer —
  Astro 7 daemonizes `astro dev` in agent/CI environments (am-i-vibing),
  which made the spawned webServer process exit early for ALL e2e specs.
  This was a pre-existing breakage, fixed to unblock `tests/e2e/blog.spec.ts`.
