# Tasks: Clean stale Stripe tests + close 11 blog/SEO scenario gaps

## Review Workload Forecast

Decision needed before apply: Yes
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: Medium

Test-only; ~440 lines dominated by deleting 7 stale Stripe assertions. Single PR
+ maintainer `size:exception` keeps the green-suite invariant (each slice must
keep `bun run test` exit 0); splitting would break CI.

### Suggested Work Units

| Unit | Goal | Commit | Focused test command | Runtime harness | Rollback boundary |
|------|------|--------|----------------------|-----------------|-------------------|
| 1 | Remove stale Stripe tests | `test(web): remove stale Stripe assertions` | `cd web && bun run test` → exit 0 | N/A — unit only | `payments.test.ts` + `checkout-integration.test.ts` |
| 2 | Add 10 blog/SEO evidence tests | `test(web): add blog/SEO runtime evidence tests` | `bun run test:e2e tests/e2e/blog.spec.ts` → exit 0 (+8) | Astro :4321 (webServer) | `draft-test-fixture.md` + `site.test.ts` + `blog.spec.ts` + `assert-build-output.mjs` |
| 3 | Verify gates | (gates only) | `test` + `tsc --noEmit` + `build` | Astro :4321 e2e | none

## Phase 1: Payments cleanup (unblock `bun run test` exit 0)

- [ ] 1.1 Modify `web/src/test/payments.test.ts`: delete `Stripe Webhook Signature` describe block (L10–113); keep `Subscription Lifecycle`. (R-1)
- [ ] 1.2 Modify `web/src/test/payments/checkout-integration.test.ts`: rewrite `geoToGateway('US')`→`'mercadopago'` and `geoToGateway(null)`→`'wompi'` (L235–242). (R-2)
- [ ] 1.3 Continue `checkout-integration.test.ts`: delete `Stripe Checkout` block (L72–120). (R-2)
- [ ] 1.4 Continue `checkout-integration.test.ts`: rewrite gateway POST block `gateway='stripe'`→`'mercadopago'` + URL `/api/checkout/mercadopago` (L245–252).
- [ ] 1.5 Continue `checkout-integration.test.ts`: drop `checkout.stripe.com` URL assert (L262–266) and STRIPE status line (L279–284).
- [ ] 1.6 Typecheck gate: `bunx tsc --noEmit` → only `web/src/lib/db.ts:25` cast (baseline).

## Phase 2: Site config evidence (vitest)

- [ ] 2.1 Add S-R7-S1 to `web/src/test/site.test.ts`: stub `SITE_URL='https://academix.example.com'`, `resetModules`, assert `absoluteUrl('/blog/x')`=`https://academix.example.com/blog/x`.

## Phase 3: Blog/SEO runtime tests (e2e, 8 scenarios)

- [ ] 3.1 Create `web/src/content/blog/draft-test-fixture.md`: full frontmatter (title, description, pubDate, author, tags) + `draft: true`.
- [ ] 3.2 Add B-R2-S1 to `web/tests/e2e/blog.spec.ts`: `GET /blog/draft-test-fixture`→404; `/blog` body lacks fixture title.
- [ ] 3.3 Add B-R6-S2: `GET /guias/guia-gestion-academica.md`→200; ct⊇`text/markdown`; body⊇`## Por qué esta guía`.
- [ ] 3.4 Add B-R6-S3: `article .prose` bbox.y < CtaBlock `section` bbox.y (DOM order).
- [ ] 3.5 Add S-R1-S1: `GET /`; canonical⊇domain; `og:title`; `twitter:card=summary_large_image`; ld+json⊇`WebSite`.
- [ ] 3.6 Add S-R1-S2: `GET /blog/{pillar}`; parse ld+json; `@type=BlogPosting`; datePublished/author/url defined.
- [ ] 3.7 Add S-R4-S3: `GET /blog/{slug}` browser-UA vs `GPTBot`; `body.text()` identical.
- [ ] 3.8 Add S-R5-S1+S-R5-S2: `/blog/{slug}` `article .prose`≥1; `/faq`+`/tutorials/download-macos` HTML⊇`prose`.

## Phase 4: Build gate (S-R2-S2 structural assert)

- [ ] 4.1 Modify `web/scripts/assert-build-output.mjs`: assert every sitemap `<loc>` URL is absolute (scheme present).
- [ ] 4.2 B-R3-S2 stays ⚠️ — strategy (ii): no `lib/blog-listing.ts` refactor, no unit test; static-source evidence only.

## Phase 5: Verification

- [ ] 5.1 `bun run test` → exit 0 (7→0 stale failures).
- [ ] 5.2 `bunx tsc --noEmit` → only `web/src/lib/db.ts:25` cast (baseline).
- [ ] 5.3 `bun run build` → exit 0.
- [ ] 5.4 `bun run test:e2e tests/e2e/blog.spec.ts` → all green (+8).
- [ ] 5.5 Re-verify: `test_exit_code:0`, `scenarios:26/27` (B-R3-S2 ⚠️).
