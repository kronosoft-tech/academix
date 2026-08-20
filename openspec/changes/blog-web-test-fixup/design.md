# Design: Clean stale Stripe tests + close 11 blog/SEO scenario gaps

## Technical Approach

Test-only change. Two surgical removal/rewrite passes on stale payments tests (no Stripe code added), then 10 runtime/gate tests closing evidence gaps from `blog-web/verify-report.md` WARNING #1. B-R3-S2 uses strategy (ii): **no refactor, no unit test** — empty-state branch stays ⚠️ partial. S-R2-S2 uses a structural build-gate instead of reproducing a failing build.

Maps to proposal step 3 + spec §1/§2/§3. Follows existing `web/` conventions: vitest `node` env with `vi.resetModules()`+`vi.stubEnv` (`site.test.ts`), Playwright `request`/`page` e2e against the dev server (`blog.spec.ts`), and the shell-runnable `assert-build-output.mjs` build-gate.

## Architecture Decisions

| # | Decision | Alternatives | Choice & rationale |
|---|----------|--------------|--------------------|
| D1 | Delete Stripe blocks, rewrite 2 gateways | Delete whole files (loses 6 lifecycle tests) | Surgical: preserves 17 real-coverage tests; only stale Stripe refs leave |
| D2 | B-R3-S2 strategy (ii) — no `lib/blog-listing.ts` | Extract filter to unit-testable module | Proposal default was (i) but task explicitly pins (ii); avoids touching `index.astro` source |
| D3 | S-R2-S2 via structural URL assertion | Run `SITE_URL="" bun run build` expecting exit≠0 | Structural assert is CI-safe; reproducing the failure needs a deliberately-broken build step |
| D4 | One new fixture file for draft (B-R2-S1) | Mock `getCollection` in vitest | Draft exclusion is a route/prerender concern → e2e + real fixture is the honest assertion |

## Data Flow

```
vitest (node)         request/stubEnv/resetModules
 site.test.ts ──────► lib/site.ts absoluteUrl()  (S-R7-S1)

Playwright dev(:4321)              build output (.vercel/output/static)
  blog.spec.ts ──────► /blog,/blog/{slug}  (B-R2-S1,B-R6-S2/3,S-R1-S1/2,S-R4-S3,S-R5-S1/2)
  ──────► scripts/assert-build-output.mjs  (S-R2-S2)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `web/src/test/payments.test.ts` | Modify | Delete `Stripe Webhook Signature` describe (L10–113); keep `Subscription Lifecycle` (L115–252) |
| `web/src/test/payments/checkout-integration.test.ts` | Modify | Rewrite 2 `geoToGateway` asserts (US→mercadopago, null→wompi); delete `Stripe Checkout` block (L72–120); rewrite `gateway='stripe'`(L245)→`'mercadopago'`; drop `checkout.stripe.com` URL; remove STRIPE status line (L279–284) |
| `web/src/test/site.test.ts` | Modify | Add S-R7-S1 test: `vi.stubEnv('SITE_URL','https://academix.example.com')`, `resetModules`, re-import; assert `absoluteUrl('/blog/x')` |
| `web/tests/e2e/blog.spec.ts` | Modify | Add 7 new e2e tests (see matrix below) |
| `web/src/content/blog/draft-test-fixture.md` | Create | `draft:true` post with full valid frontmatter (title/description/pubDate/author/tags) |
| `web/scripts/assert-build-output.mjs` | Modify | Add structural S-R2-S2 assert (sitemap/robots URLs absolute); see below |

## Test Matrix (10 added tests)

| # | Scenario | File | Fixture/Env | Assertion |
|---|----------|------|-------------|-----------|
| 1 | B-R2-S1 | `blog.spec.ts` | `draft-test-fixture.md` (draft:true) | `/blog/draft-test-fixture`→404; `/blog` body lacks title |
| 2 | B-R6-S2 | `blog.spec.ts` | none (existing `public/guias/`) | `request.get(…/guia…md)`→200; `content-type`⊇`text/markdown`; body⊇`## Por qué` |
| 3 | B-R6-S3 | `blog.spec.ts` | none | `article .prose` bbox.y < `section`(CtaBlock) bbox.y |
| 4 | S-R1-S1 | `blog.spec.ts` | none | `/` canonical href⊇domain; `og:title`; `twitter:card=summary_large_image`; ld+json⊇WebSite |
| 5 | S-R1-S2 | `blog.spec.ts` | none | `/blog/que-es-…`; parse ld+json; `@type=BlogPosting`; datePublished/autho/url defined |
| 6 | S-R4-S3 | `blog.spec.ts` | `request` UA header | browser-UA vs `GPTBot` body `.text()` identical |
| 7 | S-R5-S1 | `blog.spec.ts` | none | `/blog/{slug}`; `article .prose` count ≥1 |
| 8 | S-R5-S2 | `blog.spec.ts` | none | `/faq`+`/tutorials/download-macos` HTML⊇`prose` |
| 9 | S-R2-S2 | `assert-build-output.mjs` | `sitemap` var already parsed | config `site←SITE_URL`; all `<loc>` URLs absolute |
| 10 | S-R7-S1 | `site.test.ts` | `vi.stubEnv`+`resetModules` | `absoluteUrl('/blog/x')`=`https://academix.example.com/blog/x` |

## Interfaces / Contracts

- **Draft fixture** (`content/blog/draft-test-fixture.md`) frontmatter must satisfy `blogEntrySchema`:
  ```text
  title, description, pubDate (coerce.date), author, tags, draft: true
  ```
  Slug = filename `draft-test-fixture` → excluded from prerender (`getStaticPaths` filters `!draft` per design.md D4), so route auto-404s.

- **S-R2-S2 structural assert** (added to `assert-build-output.mjs`): after sitemap parse, `check('sitemap <loc> URLs are absolute', !sitemap.includes('/blog/academix-como-producto"') && sitemap.match(/<loc>https?:\/\//g)?.length === sitemap.match(/<loc>/g)?.length)`. Mirrors robots/Sitemap line absolute-URL check already present.

- **Shared helper**: none new. Reuses existing `vi.resetModules()`+`import` pattern (`site.test.ts`), Playwright `request.get`+`page` (`blog.spec.ts`), `check()` helper in `assert-build-output.mjs`.

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit (vitest, node) | S-R7-S1 | `stubEnv`+`resetModules`, 1 new test in `site.test.ts` |
| E2E (Playwright) | B-R2-S1,B-R6-S2/3,S-R1-S1/2,S-R4-S3,S-R5-S1/2 | 7 new tests in `blog.spec.ts`, dev server |
| Build gate | S-R2-S2 | Extend `assert-build-output.mjs` structural URL assert |

## Migration / Rollout

No migration. Test-only. Rollback = `git revert` of the change branch → restores 7-failure baseline.

## Threat Matrix

N/A — no routing/shell/subprocess/VCS/PR-automation/executable-file-classification/process-integration boundary. Test additions only.

## Open Questions

- None blocking. B-R3-S2 explicitly ⚠️ partial (strategy ii); S-R2-S2 limitation documented in D3.