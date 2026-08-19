# Exploration: blog-web — blog section + SEO + indexing + performance

Read-only exploration of the `web/` Astro 7 app (marketing + subscriptions) to scope a **blog section with a pillar page**, plus a full **SEO / social-metadata / robots.txt / sitemap / LLM-discoverability / performance** upgrade. Skills loaded: `sdd-explore`, `astro-7`, `firecrawl`. No files modified. Web research performed with Firecrawl on HubSpot resources (pillar pages / topic clusters) and Evil Martians' LLM-visibility study (Apr 2026).

## Current State

### Blog feasibility (`web/`)
- **No blog exists.** No `blog` content collection, no `/blog` routes, no blog nav entries.
- **Established content pattern to replicate** (tutorials/FAQ): `web/src/content.config.ts` defines `tutorials` + `faq` with `glob()` loaders and Zod schemas (`content.config.ts:5-24`); list pages use `getCollection()` (`pages/tutorials/index.astro:7`); detail pages use `getStaticPaths()` + `render()` with `export const prerender = true` (`pages/tutorials/[...slug].astro:5-16`). This is exactly the shape a `blog` collection should take.
- **Routing/middleware**: `web/src/middleware.ts:4-15` has `PUBLIC_ROUTES` (including `/tutorials` and `/tutorials/` prefix at `:19`). Prerendered pages skip middleware entirely via `context.isPrerendered` (`middleware.ts:36`), so prerendered blog pages bypass auth automatically — but `/blog` + `/blog/` should still be added to `PUBLIC_ROUTES` for hygiene.
- **Reusable layout**: `web/src/layouts/Base.astro` (nav + footer + slot, `<html lang="es">`). Blog listing/post pages should use it.
- **Reusable content components**: landing sections are MUI React islands (`components/landing/*.tsx`) — **NOT** appropriate for blog pages (see Performance).

### SEO / metadata audit — GAPS
- **No canonical, no Open Graph, no Twitter cards, no JSON-LD anywhere.** `grep` for `og:|twitter:|canonical|json-ld|schema.org` across `web/` matched only an unrelated comment (`lib/payments/mercadopago.ts:222`).
- `Base.astro:19-23` emits only charset, viewport, description, favicon link, title. `index.astro:9-17` has its own inline head (fonts + description) and does **not** use `Base`.
- **No `site` configured** in `astro.config.mjs:6-15` — required for canonical URLs and for `@astrojs/sitemap` to generate absolute URLs.
- **No robots.txt, no sitemap, no llms.txt anywhere in the repo** (glob across repo: zero matches).
- **No `web/public/` directory exists** — yet layouts reference `/favicon.svg` (`Base.astro:22`, `index.astro:13`). No favicon exists in the repo (glob `**/favicon.*`: zero matches) → 404. A `web/public/` dir must be created for robots.txt, llms.txt, favicon, og-image.
- **No `@astrojs/sitemap`, no `@astrojs/rss`, no SEO integrations** in `web/package.json:14-43`.
- `SITE_URL` env exists (`web/src/.env.example:19`, placeholder `https://tu-dominio.vercel.app`), used in emails/checkout with `localhost:4321` fallback (`actions/password-reset.ts:6`, `pages/api/checkout/*.ts:46,66`) — reusable for canonical/sitemap URLs, but the **real production domain must be confirmed** (risk: wrong canonical host).

### Performance — findings
- `index.astro:24` renders the whole landing page as `<LandingPage client:load />` — the ENTIRE home page is one React/MUI/emotion island. Heavy hydration cost; MUI + recharts are bundled for marketing. **Blog pages must be pure `.astro` with zero client JS.**
- **`@tailwindcss/typography` is NOT installed** (`web/package.json`), yet FAQ/tutorials use `prose prose-invert` classes (`faq.astro:37`, `tutorials/[...slug].astro:29`) → dead classes, unstyled article content. The blog needs typography styling for Markdown rendering.
- Fonts are loaded inconsistently: only `index.astro` loads Inter via Google Fonts; `Base.astro` loads none.

### LLM / AI discoverability — current state
- Zero AI-indexing artifacts (no `llms.txt`, no `.md` routes, no `Content-Signal:`, no `Link` headers). Since blog content will be authored in Markdown via content collections, serving `.md` routes costs almost nothing (single source of truth, no drift).

## Research Summary (Firecrawl)

### HubSpot: conversion-oriented blog + pillar pages / topic clusters
Sources: `blog.hubspot.es/website/como-crear-un-blog-orientado-a-la-conversion` (user-provided), `knowledge.hubspot.com/content-strategy/pillar-pages-topics-and-subtopics`, topic-cluster explainers.

1. **Pillar page** = comprehensive resource covering a broad topic in depth; lives at a top-level, high-traffic URL; topic in title, URL, and H1; **nothing locked behind forms/paywalls** (crawlability); content adapted to convert.
2. **Topic cluster model**: one pillar + subtopic posts; cluster posts link **back to the pillar** and the pillar links out to each subtopic; descriptive anchor text; ~2–5 internal links per page (avoid link bloat).
3. **Conversion elements**: HubSpot's own article demonstrates the pattern — top-of-page lead magnet (free kit download CTA), CTAs inside and at the end of the article, and all supporting content funneling to the pillar/conversion point.
4. Caveat: creating topic clusters "doesn't affect SEO directly" — value comes from the content quality + internal linking itself.

### LLM discoverability (Evil Martians, Apr 2026 — 6 techniques that work)
| # | Technique | Priority | Effort |
|---|-----------|----------|--------|
| 0 | `robots.txt` audit — don't block AI crawlers; optional `Content-Signal:` line | Critical | Low |
| 1 | `/llms.txt` (llmstxt.org format: H1 + blockquote + H2 link sections) | Critical | Low (static file) |
| 2 | `.md` routes — clean Markdown at `URL.md`; **Astro content is already Markdown → serve source directly** | Critical | Low |
| 3 | `<link rel="alternate" type="text/markdown">` + HTTP `Link` header | High | Low |
| 4 | Hidden `<div>` hint for paste-URL-into-ChatGPT case | Medium | Low |
| 5 | `/llms-full.txt` | Low–Med | Low |
| 6 | `Accept: text/markdown` content negotiation (q-values, `Vary: Accept`, 406) | High | Med |

Key evidence & caveats:
- No major LLM provider has committed to reading `llms.txt`; 94.9% of its traffic is GoogleBot. Its real value: **human/coding-tool pastes a URL into an LLM** (README for AI-mediated conversations). Still worth shipping — near-zero cost.
- `.md` routes: log studies show crawlers rarely fetch them unprompted; the value is the same human-initiated path, plus 80% token reduction vs HTML soup. HTML with clean semantics is already legible (HtmlRAG).
- **JSON-LD does NOT directly help LLM visibility** (ChatGPT/Claude/Perplexity treat it as text; only Bing/Copilot inherit schema). Keep JSON-LD for classic SEO/rich results, don't expect LLM gains.
- **GEO content signals that DO work** (KDD 2024 study): direct quotations (+43% AI visibility), statistics (+33%), citing authoritative sources (+115%).
- Anti-patterns to avoid: `<meta name="ai-content-url">`, `<meta name="llms">`, `/.well-known/ai.txt`, HTML comments, UA-sniffing to serve Markdown (= cloaking, Google-penalized), dedicated "AI info pages".
- Vercel publishes an implementation guide for agent-friendly pages via content negotiation; scanners: `acceptmarkdown.com`, `isitagentready.com`.

## Affected Areas

| Area | Location | Why |
|------|----------|-----|
| Content collections | `web/src/content.config.ts` | Add `blog` collection (glob loader + Zod schema: title, description, pubDate, updatedDate?, author, tags, pillar flag, draft?, coverImage?, slug override) |
| Blog content | `web/src/content/blog/*.md` (new) | Markdown posts (Spanish) incl. one or more pillar posts flagged in frontmatter |
| Blog listing | `web/src/pages/blog/index.astro` (new) | Prerendered list grouped by topic cluster; links to pillar |
| Blog post | `web/src/pages/blog/[...slug].astro` (new) | Prerendered `getStaticPaths()`; renders `Content`, TOC, related posts, CTA; mirrors tutorials pattern |
| Pillar page | `web/src/pages/blog/` (pillar post or `pages/blog/guia-*.astro`) | HubSpot-style pillar: TOC jump links, section-by-section subtopic link-outs, end-of-page conversion CTA |
| Middleware | `web/src/middleware.ts:4-15` | Add `/blog` and `/blog/` prefix to `PUBLIC_ROUTES` |
| Layout | `web/src/layouts/Base.astro` | Consume new Seo component; add blog nav/footer links; unify fonts |
| SEO component | `web/src/components/Seo.astro` (new) | canonical, OG, Twitter, robots meta, JSON-LD (WebSite/Organization/BlogPosting), uses `SITE_URL` + `Astro.url` |
| Astro config | `web/astro.config.mjs` | Add `site` (REQUIRED); add `@astrojs/sitemap` integration |
| Home page | `web/src/pages/index.astro` | Add same Seo head (OG/Twitter/canonical/JSON-LD Organization) |
| Static assets | `web/public/` (new dir) | `robots.txt`, `llms.txt`, `favicon.svg`, `og-default.png` (social card), maybe `llms-full.txt` |
| Sitemap | via `@astrojs/sitemap` | Auto-generates `sitemap-index.xml` from prerendered pages (blog included); decide on SSR-only `/pricing` coverage |
| Typography | `web/package.json` + `src/styles/global.css` | Add `@tailwindcss/typography` (prose classes currently dead) |
| Images | blog covers | Astro `astro:assets` `<Image>` (built-in optimization, srcset, AVIF/WebP) |
| Env | `web/.env.example`, `web/.env` (Vercel) | Confirm real `SITE_URL` production domain for canonical/sitemap |

## Approaches

1. **Content-collection blog + prerendered pages (recommended)** — mirror the tutorials pattern exactly.
   - Pros: type-safe, validated frontmatter, zero-JS pages (fastest), auto-included in sitemap, `.md` routes served from the same source (no drift), no DB/API dependency, works with Vercel static prerendering.
   - Cons: content lives in the repo (no CMS editor); rebuild required to publish.
   - Effort: Low–Medium for the blog itself.

2. **CMS/DB-backed blog (e.g., Turso or headless CMS)** — blog posts in the control-plane DB or external CMS.
   - Pros: non-developers publish; live updates without deploy.
   - Cons: SSR per request (middleware + sitemap + caching complexity), new auth/CRUD surface, heavier; contradicts the static-content best practice for blogs; unnecessary for a starter blog.
   - Effort: High. Not recommended now.

3. **Astro Content Layer "live" collections** (`live.config.ts`) — per-request freshness.
   - Pros: no rebuild for updates if the source changes at runtime.
   - Cons: network cost per request, no MDX, no image optimization, no prerendering → sitemap/SEO/performance all worse for static blog content.
   - Effort: Medium. Not needed unless posts move to an external source.

## Recommendation

**Scope for `blog-web` (in order):**

1. **Blog core**: `blog` content collection + `web/src/pages/blog/index.astro` + `web/src/pages/blog/[...slug].astro`, all `prerender = true`; add `/blog` + `/blog/` to `PUBLIC_ROUTES`; blog links in `Base.astro` nav/footer. Posts in Spanish (site language). Seed with 2–4 posts + 1 pillar post.
2. **Pillar page**: first pillar = e.g. "Guía completa de gestión académica" — HubSpot-style anatomy: intro, jump-link TOC, section per subtopic linking out to full posts, related-posts block, closing conversion CTA (download/trial). Cluster posts link back to the pillar (2–5 internal links/page).
3. **SEO metadata layer**: new `Seo.astro` component (canonical, robots, OG, Twitter, JSON-LD). Wire into `Base.astro` + `index.astro`. Set `site` in `astro.config.mjs` (prerequisite). Confirm real `SITE_URL`.
4. **Social metadata**: OG/Twitter cards on every public page; add `web/public/og-default.png` social card; `<meta name="robots" content="index,follow,max-image-preview:large">` (HubSpot pattern).
5. **robots.txt** (`web/public/robots.txt`): allow all incl. GPTBot/ClaudeBot/PerplexityBot, disallow `/api/` + `/dashboard` + `/admin`, `Sitemap:` pointer. Optional `Content-Signal:` line (emerging, flagged experimental — harmless warning in Google validator).
6. **Sitemap**: add `@astrojs/sitemap` (+ `site`); covers all prerendered pages (blog included). Decide whether `/pricing` (SSR-only) needs a supplementary sitemap entry (manual endpoint) — default: accept omission or add manual `pages/sitemap.xml` if full coverage wanted.
7. **LLM discoverability**: `llms.txt` (static); `.md` routes for blog posts (same markdown source, `text/markdown`); `<link rel="alternate" type="text/markdown">` in post heads; optional HTTP `Link` header via `vercel.json` `headers` (or middleware for SSR routes); skip `llms-full.txt` (marketing/blog = overkill per research) and skip content negotiation (defer — effort/benefit not yet justified for a marketing site; revisit when blog grows).
8. **Performance**: blog pages pure `.astro` (no islands, zero JS); add `@tailwindcss/typography` for article prose; Astro `<Image>` for covers; keep landing-page MUI hydration OUT of blog scope (note as separate optimization candidate).

## Risks

- **HIGH — Missing `site` config + placeholder `SITE_URL`**: canonical/sitemap URLs wrong until the real production domain is confirmed and set. Verify the deployed domain before shipping.
- **MEDIUM — `.md` routes / `llms.txt` are bets on the future**: no LLM provider commits to reading them; value is human-initiated AI interactions. Low cost, but set expectations (don't over-invest).
- **MEDIUM — JSON-LD ≠ LLM visibility**: include structured data for classic SEO/rich results only; do NOT treat it as the AI-indexing answer.
- **MEDIUM — Content language**: site is Spanish; blog + metadata must be authored in Spanish (SEO targets `es`); this exploration artifact stays English per convention.
- **LOW — `@tailwindcss/typography` addition** changes prose rendering on existing FAQ/tutorials pages (currently unstyled) — verify visual regression.
- **LOW — Stale artifacts**: `web/src/.env.example` is known-stale (AGENTS.md); update `SITE_URL` line as part of this change if touched.
- **NOT in scope (do NOT do)**: Stripe additions, CMS/DB-backed blog, `llms-full.txt`, full `Accept: text/markdown` negotiation, landing-page MUI rewrite, UA-sniffing for AI bots (cloaking).

## Ready for Proposal

**Yes.** Scope is well-defined with a proven in-repo pattern (tutorials) and clear research-backed priorities. The orchestrator should tell the user: blog section + pillar page via Astro content collections (prerendered, Spanish), plus a layered SEO/metadata/robots/sitemap/llms.txt upgrade; and confirm (a) the real production domain for `site`/`SITE_URL`, and (b) whether the first pillar topic is the proposed "gestión académica" guide or another theme.