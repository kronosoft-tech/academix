# Proposal: Blog + SEO + Indexing + Performance (blog-web)

## Intent

Academix needs a conversion-oriented blog to attract via SEO and convert readers to registration/trial. Today the web app has no blog, no canonical/OG/Twitter/JSON-LD, no `site` config, no robots.txt/sitemap/llms.txt, no favicon, dead `prose` classes, and a heavy MUI island on home. This change ships a Spanish-first blog with a pillar page, plus a full-site SEO/indexing layer, with structure ready for future English.

## Scope

### In Scope
- **Blog core**: `blog` content collection (glob + Zod), prerendered `/blog` + `/blog/[...slug]`, `/blog` added to `PUBLIC_ROUTES`, nav/footer links
- **Pillar page**: "Academix como producto", HubSpot anatomy — TOC jump links, subtopic link-outs, conversion CTA
- **Content seed**: pillar + 8–12 posts (Spanish); lead-magnet guide (downloadable) + trial CTA in pillar and posts
- **SEO layer (whole public site)**: `Seo.astro` (canonical, OG, Twitter, robots, JSON-LD); `site` in `astro.config.mjs`; `@astrojs/sitemap`; `web/public/` with robots.txt, favicon, og-default.png
- **LLM discoverability**: `llms.txt`; `.md` routes + `<link rel="alternate" type="text/markdown">`
- **Typography**: add `@tailwindcss/typography` (prose classes currently dead on FAQ/tutorials)
- **Home performance**: split MUI island, lazy-load, reduce JS weight — no redesign, keep aesthetics
- **Env**: `SITE_URL` env-driven (current Vercel domain, not hardcoded); refresh stale `.env.example`

### Out of Scope
- Full i18n (routes/frontmatter i18n-ready only), `llms-full.txt`, `Accept: text/markdown` content negotiation, CMS/DB-backed blog, Stripe, landing MUI redesign, UA-sniffing/cloaking

## Capabilities

### New Capabilities
- `blog`: content collection, listing/detail/pillar pages, CTAs, middleware routes, nav
- `seo`: Seo.astro metadata, site config, sitemap, robots.txt, llms.txt, `.md` routes

### Modified Capabilities
- None

## Approach

Replicate the tutorials pattern: `content.config.ts` glob+Zod → prerendered list (`blog/index.astro`) + `[...slug].astro` with `getStaticPaths()` and `prerender = true`, pure `.astro` (zero client JS). `Seo.astro` consumed by `Base.astro` and `index.astro`. Sitemap integration + static files in new `web/public/`. Home: split the single `<LandingPage client:load />` island into smaller `client:visible` islands.

## Open Questions

- **SITE_URL (HIGH)**: confirm the current Vercel deployment domain before shipping — `site`/canonical/sitemap must use it; do NOT hardcode a guessed domain.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `web/src/content.config.ts` | Modified | Add `blog` collection |
| `web/src/content/blog/*.md` | New | Seed posts + pillar (Spanish) |
| `web/src/pages/blog/index.astro` | New | Prerendered list |
| `web/src/pages/blog/[...slug].astro` | New | Post + pillar rendering |
| `web/src/components/Seo.astro` | New | Canonical/OG/Twitter/JSON-LD |
| `web/src/middleware.ts` | Modified | `/blog`, `/blog/` in `PUBLIC_ROUTES` |
| `web/src/layouts/Base.astro` | Modified | Seo component, blog nav/footer, fonts |
| `web/src/pages/index.astro` | Modified | Seo head, island split/lazy-load |
| `web/astro.config.mjs` | Modified | `site`, `@astrojs/sitemap` |
| `web/public/` | New | robots.txt, llms.txt, favicon.svg, og-default.png |
| `web/package.json` | Modified | `@astrojs/sitemap`, `@tailwindcss/typography` |
| `web/src/.env.example` | Modified | Refresh `SITE_URL` line |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Wrong `site`/`SITE_URL` → broken canonicals/sitemap | High | Env-driven; confirm Vercel domain pre-ship |
| `llms.txt`/`.md` value is a bet (no LLM commits to them) | Medium | Low cost; position as human-initiated AI value |
| JSON-LD ≠ LLM visibility | Medium | Keep for rich results only, no AI expectations |
| `@tailwindcss/typography` changes existing prose pages | Low | Visual regression check on FAQ/tutorials |

## Rollback Plan

Revert to prior commit. Blog is additive: removing the collection + pages drops routes and sitemap entries automatically. Removing `Seo.astro` usage restores the old head. Delete `web/public/` files and uninstall the two new packages to restore baseline.

## Dependencies

- Confirm real `SITE_URL` (Vercel deployment domain)
- `@astrojs/sitemap`, `@tailwindcss/typography` installs in `web/`

## Success Criteria

- [ ] `/blog`, pillar, and posts render prerendered with zero client JS
- [ ] Canonical/OG/Twitter/JSON-LD present on all public pages; robots.txt + sitemap valid
- [ ] `llms.txt` + `.md` routes served; `<link rel="alternate">` in post heads
- [ ] Home page JS weight reduced without visual redesign
- [ ] Seed content live: pillar + 8–12 posts in Spanish