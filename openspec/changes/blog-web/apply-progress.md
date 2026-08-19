# Apply Progress: Blog + SEO + Indexing + Performance (blog-web)

**Status**: COMPLETE — all 6 work units committed on `main`, single PR, `size:exception` approved.

**Date**: 2026-08-19

## Work Units

| # | Commit | Scope | Evidence |
|---|--------|-------|----------|
| 1 | `2c96add` | Foundation: sitemap + typography deps, `site.ts`, `headings.ts`, blog Zod collection, prose plugin, `.env.example` | `bunx tsc --noEmit` (7 pre-existing errors only); build Complete + sitemap-index.xml |
| 2 | `ddfd0a1` | Seed pillar + 10 posts (ES, staggered pubDates 2026-03-02→07-13, `draft: false`) + lead magnet `/guias/guia-gestion-academica.md` | `bun run build` (schema gate passed) |
| 3 | `96c751d` | Prerendered `/blog` listing (pillar pin, tag groups, zero JS) + post pages (TOC, related ≤3, CtaBlock, alternate markdown link) + `/blog/{slug}.md` endpoint | headings+site unit tests 13/13; build prerenders 11 posts + 10 `.md`; blog HTML ships no scripts |
| 4 | `6fb998c` | `Seo.astro` (canonical/robots/OG/Twitter/JSON-LD), robots.txt, llms.txt, favicon + og-default.png, Base.astro (Inter, Seo, Blog nav+footer), middleware `/blog`, vercel.json `.md` headers | build Complete; post head = article + BlogPosting; home = WebSite+Organization graph; robots/llms emitted |
| 5 | `fe96dfa` | Home perf: static SSR sections + single `client:visible` Navbar island (Tailwind), measure script + baseline | `bun run measure:home-js`: 421,214 → 187,506 bytes (**-55.5%**, gate ≤ 294,850 passes) |
| 6 | `0c91d64` | blog-schema unit tests (8), e2e blog.spec.ts (8), assert-build-output script (10 assertions), playwright webServer fix, Navbar Blog link (R7) | tsc 7 baseline; 213 unit tests pass; e2e 8/8; assert:build all pass |

## Gates (final run)

- `bunx tsc --noEmit` (web/): **7 errors, all pre-existing baseline** (db.ts cast, Stripe-stale tests) — zero new
- `bun run test` (web/): **213 passed / 7 failed** — failures are exactly the pre-existing Stripe-stale tests (payments.test.ts + checkout-integration.test.ts), untouched by design
- `bun run build`: Complete; sitemap lists `/blog` + all 11 slugs; 10 `.md` endpoints emitted
- `bun run assert:build`: 10/10 assertions pass (sitemap, .md, robots, llms, zero-JS blog listing, localhost fallback)
- `bun run measure:home-js`: **187,534 bytes** (ratio 0.445, -55.5%) — passes `< baseline` and `≤ 0.7× baseline`
- `bun run test:e2e tests/e2e/blog.spec.ts`: **8/8 pass** (listing order/pillar, post TOC/CTA/alternate, 404, `.md` content-type, robots AI bots, llms pillar-first, anonymous /blog, home nav)

## Deviations from Design (recorded in tasks.md Apply Notes)

1. **rehype-slug not installed** (task 1.2): Astro 7 Sätteri natively slugs headings with github-slugger; `headings.ts` replicates it (dup `-1` suffixes verified in rendered HTML). TOC anchors match rendered ids — covered by e2e.
2. **Home perf approach** (tasks 4.1–4.3): static SSR + single Navbar island instead of island-per-section. Island-per-section with MUI could not meet ≤0.7× (~360KB); static sections + Tailwind Navbar hit 187.5KB (-55.5%), exceeding the gate. `LandingPage.tsx` kept as static wrapper instead of deleted.
3. **measure script** (4.4): sums `astro-island component-url/renderer-url` (the real load mechanism — home JS loads dynamically, no `<script src>` tags).
4. **playwright.config.ts**: `ASTRO_DEV_BACKGROUND=1` pin — pre-existing e2e breakage from Astro 7 daemonizing `astro dev` in agent environments; fixed to run the new e2e suite.

## Notable fixes found by the suite

- Home navbar (Tailwind) was missing the Blog link (blog R7) — added to desktop + mobile drawer.
- `.md` endpoint initially wrapped in `---` frontmatter delimiters (TS endpoint pattern) — endpoints are plain modules; fixed.
- Plain `zod` vs `astro/zod` instance mismatch broke Astro's JSON-schema generation + collection type inference — schema extracted to `lib/blog-schema.ts` importing `astro/zod`.

## Rollback

Each commit is independently revertible (see tasks.md Work Units table). `git revert` of the 6 commits in reverse order restores the pre-change tree.