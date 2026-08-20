# Apply Progress: blog-web-test-fixup

**Strategy**: (ii) test-only — no app refactor. B-R3-S2 stays ⚠️ partial (no refactor, no unit test).

---

## 22-Task Checklist

### Phase 1: Payments cleanup (DONE — pre-applied, verified intact)

- [x] 1.1 Modify `web/src/test/payments.test.ts`: delete `Stripe Webhook Signature` describe block (L10–113); keep `Subscription Lifecycle`. (R-1)
- [x] 1.2 Modify `web/src/test/payments/checkout-integration.test.ts`: `geoToGateway('US')`→`'mercadopago'` (L187). (R-2)
- [x] 1.3 Continue `checkout-integration.test.ts`: `geoToGateway(null)`→`'wompi'` (L192). (R-2)
- [x] 1.4 Continue `checkout-integration.test.ts`: rewrite gateway POST block `gateway='stripe'`→`'mercadopago'` + URL `/api/checkout/mercadopago` (L196–203). (R-2)
- [x] 1.5 Continue `checkout-integration.test.ts`: drop `checkout.stripe.com` URL assert and STRIPE status line. (R-2)
- [x] 1.6 Typecheck gate: `bunx tsc --noEmit` → only `web/src/lib/db.ts:25` cast (baseline).

**Phase 1 status**: DONE ✓ — verified intact via `git diff`: 170 deletions, 8 insertions across both files. Zero Stripe references remain. `geoToGateway('US')`→`'mercadopago'`, `geoToGateway(null)`→`'wompi'`.

### Phase 2: Site config evidence (DONE)

- [x] 2.1 Add S-R7-S1 to `web/src/test/site.test.ts`: stub `SITE_URL='https://academix.example.com/'`, `resetModules`, assert `absoluteUrl('/blog/x')`=`https://academix.example.com/blog/x`.

**Phase 2 status**: DONE ✓ — added test; `site.test.ts` now 49 lines (was 44).

### Phase 3: Blog/SEO runtime tests (DONE)

- [x] 3.1 Create `web/src/content/blog/draft-test-fixture.md`: full frontmatter + `draft: true`.
- [x] 3.2 Add B-R2-S1 to `web/tests/e2e/blog.spec.ts`: `GET /blog/draft-test-fixture`→404; `/blog` body lacks fixture title; 10 unique titles preserved.
- [x] 3.3 Add B-R6-S2: `GET /guias/guia-gestion-academica.md`→200; ct⊇`text/markdown`; body⊇`## Por qué esta guía`; guide link visible on post page.
- [x] 3.4 Add B-R6-S3: `article .prose` bbox.y < CtaBlock `<section>` bbox.y (DOM order).
- [x] 3.5 Add S-R1-S1: `GET /`; `<title>` + `<meta name="description"` present + non-empty.
- [x] 3.6 Add S-R1-S2: `GET /blog/que-es-un-sistema-de-gestion-academica`; parse ld+json; `@type`=`BlogPosting`; `author.name`⊇`Academix`; `datePublished`/`url` defined.
- [x] 3.7 Add S-R4-S3: `GET /blog/{slug}` browser-UA vs `Googlebot` vs `BYOBOT`; `body.text()` length identical.
- [x] 3.8 Add S-R5-S1 + S-R5-S2: `/blog/{slug}` `article .prose`≥1; `/faq` + `/tutorials/download-macos` HTML⊇`prose`.
- [ ] 3.9 B-R3-S2: NO test — strategy (ii) ⚠️. Empty-state branch at `index.astro:55` not exercised. Stays ⚠️ → 26/27.

**Phase 3 status**: DONE ✓ — fixture created (11 lines); `blog.spec.ts` now 218 lines (+100). 8 new e2e tests all passing.

### Phase 4: Build gate (DONE)

- [x] 4.1 Modify `web/scripts/assert-build-output.mjs`: assert every sitemap `<loc>` URL is absolute (scheme present).
- [x] 4.2 B-R3-S2 stays ⚠️ — strategy (ii): no `lib/blog-listing.ts` refactor, no unit test; static-source evidence only.

**Phase 4 status**: DONE ✓ — `assert-build-output.mjs` now 108 lines (+12). 11/11 build-gate assertions pass.

### Phase 5: Verification (DONE)

- [x] 5.1 `bun run test` → exit 0 (7→0 stale failures). **211 passed / 211 total**.
- [x] 5.2 `bunx tsc --noEmit` → only `web/src/lib/db.ts:25` cast (baseline). 0 Stripe errors.
- [x] 5.3 `bun run build` → exit 0. Build complete. Draft fixture NOT prerendered (confirms B-R2-S1 at build time).
- [x] 5.4 `bun run test:e2e tests/e2e/blog.spec.ts` → all green (+8 e2e). **16 passed / 16 total** (8 existing + 8 new).
- [x] 5.5 Build gate: `node scripts/assert-build-output.mjs` → 11/11 PASS.

**Phase 5 status**: DONE ✓ — Re-verify: `test_exit_code:0`, `scenarios:26/27` (B-R3-S2 ⚠️).

---

## Gate Table

| Gate | Command | Exit | Result | Output Hash |
|------|---------|------|--------|-------------|
| Typecheck | `bunx tsc --noEmit` | 2 | 1 error: `db.ts:25` cast (baseline) | `edba644bacb372a84727c11ddbd3d4727fd2ec5910055fafa9d7e2efe66ebca6` |
| Unit tests | `bun run test` | 0 | 211 passed / 211 total (17 files) | `1028165cc60d065c76d8b7982e9d7f6adb098c6dc983fac5054b7970dc0adeb0` |
| Build | `bun run build` | 0 | Complete!, 15.16s server build | `f9fec76dae6c15d0aa8ac0250d9d9301e61f7f3df23eacd93b41bd9401ee334a` |
| Build gate | `node scripts/assert-build-output.mjs` | 0 | 11/11 PASS | (inline) |
| E2E (blog) | `bun run test:e2e tests/e2e/blog.spec.ts` | 0 | 16 passed / 16 total (8 existing + 8 new) | `a799c7151e6ac21e61c57fb566b23582b03b58e04276f6515cc85adbb6d75c80` |

## Changed / New Files

| File | Action | Lines | Delta |
|------|--------|-------|-------|
| `web/src/test/payments.test.ts` | Modified (Phase 1) | 148 | −105 (Stripe Webhook Signature block deleted) |
| `web/src/test/payments/checkout-integration.test.ts` | Modified (Phase 1) | 243 | −73 (Stripe Checkout + STRIPE status removed; geoToGateway rewritten) |
| `web/src/test/site.test.ts` | Modified (Phase 2) | 49 | +6 (S-R7-S1 test) |
| `web/tests/e2e/blog.spec.ts` | Modified (Phase 3) | 218 | +100 (8 new e2e tests) |
| `web/scripts/assert-build-output.mjs` | Modified (Phase 4) | 108 | +12 (S-R2-S2 structural assert + limitation doc) |
| `web/src/content/blog/draft-test-fixture.md` | Created (Phase 3 fixture) | 11 | new |

## Commit Work-Unit Plan

| Commit | Files | Description |
|--------|-------|-------------|
| c1 | `src/test/payments.test.ts`, `src/test/payments/checkout-integration.test.ts` | `test(web): remove stale Stripe assertions` |
| c2 | `src/test/site.test.ts`, `tests/e2e/blog.spec.ts`, `src/content/blog/draft-test-fixture.md`, `scripts/assert-build-output.mjs` | `test(web): add blog/SEO runtime evidence tests` |
| c3 | — | gates only (amend into c2) |

**Pre-existing baseline (unaffected)**: `db.ts:25` cast error; 7 Stripe-stale tests removed (5× missing `lib/payments/stripe` import + 2× `geoToGateway` asserting removed `'stripe'` branch).

## Honest Limitations

- **S-R2-S2**: Cannot safely reproduce Astro's build-time throw on a missing `site` (astro.config.mjs:9 `site: SITE_URL` falls back to `http://localhost:4321`). Structural output assert (every `<loc>` has a scheme) used instead. Gate = build-gate, not vitest.
- **B-R3-S2**: Empty-state branch `{published.length === 0 && …}` at `index.astro:55` unreachable in e2e (seed always 11 posts, `draft:false`). Strategy (ii) — no refactor, no unit test. Stays ⚠️ → 26/27.
- **B-R2-S1**: Draft fixture is a content collection entry (`draft: true`), loaded by the glob but filtered in `getStaticPaths`. Verified at both build time (not prerendered) and runtime (404 + absent from listing).
