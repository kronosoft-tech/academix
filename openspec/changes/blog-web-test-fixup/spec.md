# Delta Test Plan: blog-web-test-fixup

**Nature**: test-only; no product/spec behavior changes. Removes 7 stale Stripe tests + adds 10 runtime tests closing blog/SEO evidence gaps. B-R3-S2 → strategy (ii) ⚠️ partial. Mode: hybrid.

**Facts**: `Gateway='wompi'|'mercadopago'`; `geoToGateway(null)→'wompi'`, `('CO')→'wompi'`, else→`'mercadopago'` (`gateway.ts`). `astro.config.mjs`: `site=SITE_URL||'http://localhost:4321'`+`sitemap()`. No UA branching (`middleware.ts`). `prose` classes on posts/FAQ/tutorials.

## MODIFIED (`web/src/test/payments/checkout-integration.test.ts`)

The suite MUST assert real `geoToGateway()` routing; drop all `'stripe'`.

| Location | New value |
|----------|-----------|
| L237 `geoToGateway('US')` | `'mercadopago'` |
| L242 `geoToGateway(null)` | `'wompi'` |
| L245–253 gateway block | `'mercadopago'` + `/api/checkout/mercadopago` |
| L263 `checkout.stripe.com` URL | `mercadopago.com.co/subscriptions/checkout` |
| L61 `'nonexistent'` | `'nonexistent' as string` (tsc gate; runtime unchanged) |
| L279 `STRIPE: ✅` status line | removed; WOMPI/MP kept |

- GIVEN `geoToGateway('US')` WHEN called THEN → `'mercadopago'`
- GIVEN `geoToGateway(null)` WHEN called THEN → `'wompi'`

## REMOVED Test Requirements

### R-1 Stripe Webhook Signature (`payments.test.ts` L10–113)
(Reason: 5 tests import non-existent `../lib/payments/stripe`; AGENTS.md §7: Stripe not implemented. Migration: none — `Subscription Lifecycle` L115–252 retained, 6 tests kept.)

### R-2 Stripe Checkout + status (`checkout-integration.test.ts` L72–120, L279)
(Reason: asserts non-existent `/api/checkout/stripe` + `'stripe'` gateway. Migration: none; WOMPI/MP blocks + `geoToGateway('CO'/'AR')` kept.)

## ADDED Test Requirements (10 scenarios)

| # | Scenario | File(s) created/modified | Harness | Exact assertion |
|---|----------|--------------------------|---------|-----------------|
| 1 | B-R2-S1 draft excluded | `+content/blog/draft-test-fixture.md`(draft:true) +`tests/e2e/blog.spec.ts` | e2e | GET `/blog/draft-test-fixture`→404; `/blog` body lacks fixture title |
| 2 | B-R6-S2 guide download | `tests/e2e/blog.spec.ts` | e2e | GET `/guias/guia-gestion-academica.md`→200; ct⊇`text/markdown`; body⊇`## Por qué esta guía` |
| 3 | B-R6-S3 prose before CTA | `tests/e2e/blog.spec.ts` | e2e | `article .prose` box.y < CtaBlock `section` box.y |
| 4 | S-R1-S1 home SEO | `tests/e2e/blog.spec.ts` | e2e | GET `/`; canonical href⊇domain; og:title; twitter:card=`summary_large_image`; ld+json⊇`WebSite` |
| 5 | S-R1-S2 BlogPosting LD | `tests/e2e/blog.spec.ts` | e2e | GET `/blog/que-es-un-sistema…`; parse ld+json; `@type`=`BlogPosting`; datePublished,author,url defined |
| 6 | S-R4-S3 no UA cloaking | `tests/e2e/blog.spec.ts` | e2e | GET `/blog/{slug}` browser-UA vs `GPTBot`; body `.text()` identical |
| 7 | S-R5-S1 prose class | `tests/e2e/blog.spec.ts` | e2e | GET `/blog/{slug}`; `article .prose` count ≥1 |
| 8 | S-R5-S2 prose no regress | `tests/e2e/blog.spec.ts` | e2e | GET `/faq` + `/tutorials/download-macos`; HTML⊇`prose` |
| 9 | S-R2-S2 sitemap contract | `scripts/assert-build-output.mjs` | build-gate | config has `sitemap()` + `site←SITE_URL`; built `sitemap-0.xml` lists absolute URLs |
| 10 | S-R7-S1 prod SITE_URL | `src/test/site.test.ts` | vitest | stub `SITE_URL='https://academix.example.com'`; `absoluteUrl('/blog/x')`=`https://academix.example.com/blog/x` |

**Fixture note (B-R2-S1)**: `draft-test-fixture.md` requires full valid frontmatter (title, description, pubDate, author, tags) + `draft: true` so the B-R1 schema gate passes while the route is excluded everywhere.

## RENAMED Requirements
None.

## B-R3-S2 — Strategy (ii) ⚠️ PARTIAL
`index.astro:55` `{published.length === 0 && …}` unreachable in e2e (11-post seed, all `draft:false`). NO refactor (no `lib/blog-listing.ts`), NO unit test. Gated by static-source evidence only (branch present, render-path confirmed in build output). Stays ⚠️ → 26/27.

## Acceptance Criteria (Exit)
- `bun run test` → exit 0 (7→0 stale failures; 17 lifecycle/Wompi/MP + 21 blog/seo unit preserved)
- `bunx tsc --noEmit` → only `web/src/lib/db.ts:25` cast remains (baseline)
- `bun run build` → exit 0
- `bun run test:e2e tests/e2e/blog.spec.ts` → all green (+8 e2e)
- Re-verify: `test_exit_code:0`, `scenarios:26/27` (B-R3-S2 ⚠️)

## Out of Scope
`Footer.tsx` Blog→`/tutorials`; `.env.example` STRIPE keys; no Stripe code; `gateway.ts` untouched; no product/UX/DB migration.
