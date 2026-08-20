# Exploration: blog-web-test-fixup

Read-only investigation for the `blog-web-test-fixup` change: clean stale payments
tests that cause the `bun run test` non-zero exit, and add runtime tests for the 11
blog/SEO spec scenarios that currently have only source-build evidence. No files
modified. Skills loaded: `sdd-explore`, `astro-7`, `typescript`, `playwright`.

## Current State

`blog-web` is apply-complete (committed, all 31 tasks done, build passes, e2e 8/8
pass). But `sdd-verify` returned **FAIL** on two gates:

1. **`test_exit_code: 1`** — 7 pre-existing failures in 2 payments test files (0
   blog-web files touched). All 7 are stale Stripe assertions referencing a gateway
   that was removed/never existed (`Gateway = 'wompi' | 'mercadopago'` only).
2. **`scenarios: 16/27`** — 11 scenarios have source/build evidence but no dedicated
   runtime test asserting them (10 ⚠️ PARTIAL + 1 ❌ UNTESTED).

Both are documented in `openspec/changes/blog-web/verify-report.md` (the verdict
paragraph and WARNING #1/#5) and confirmed by prior engram discovery #1199
(`web/src/lib/payments/stripe.ts` does not exist; `gateway.ts` has no Stripe branch).

## (a) Stale Stripe Test Files — Root Cause & Exact Fix

**Two files cause all 7 runtime failures:**

### File 1: `web/src/test/payments.test.ts` — 5 failures

Every test in the `Stripe Webhook Signature` describe block imports the non-existent
`../lib/payments/stripe` module. `lib/payments/stripe.ts` does not exist (AGENTS.md
§7: "Stripe is not implemented"). The `Gateway` union in `gateway.ts:1` is
`'wompi' | 'mercadopago'` only.

| Line | Code | Result |
|------|------|--------|
| 16 | `await import('../lib/payments/stripe')` | FAIL — module not found |
| 27 | `await import('../lib/payments/stripe')` | FAIL |
| 50 | `await import('../lib/payments/stripe')` | FAIL |
| 75 | `await import('../lib/payments/stripe')` | FAIL |
| 96 | `await import('../lib/payments/stripe')` | FAIL |

All 5 failures are in the `Stripe Webhook Signature` describe block (lines 10–113).
The `Subscription Lifecycle` describe block (lines 115–252) tests the real
`lifecycle.ts` module and **passes** — it must be preserved.

**Fix: Delete the `Stripe Webhook Signature` describe block (lines 10–113) only.**
Keep the `Subscription Lifecycle` block. This removes all 5 failures and preserves
6 passing tests that cover real subscription lifecycle code.

> Note: `payments.test.ts:127` passes `'sub_stripe_1'` as a string argument to
`createTrialSubscription` — this does NOT import the stripe module; it just uses a
string literal for the `stripe_subscription_id` DB column (which exists in the
schema). It passes and should be kept.

### File 2: `web/src/test/payments/checkout-integration.test.ts` — 2 failures

| Line | Code | Actual | Expected | Fix |
|------|------|--------|----------|-----|
| 237 | `expect(geoToGateway('US')).toBe('stripe')` | `'mercadopago'` | `'stripe'` | Rewrite → `'mercadopago'` |
| 242 | `expect(geoToGateway(null)).toBe('stripe')` | `'wompi'` | `'stripe'` | Rewrite → `'wompi'` |

Root cause: `geoToGateway()` in `gateway.ts:26-31` routes `CO → 'wompi'` and
everything else (including `null`) → `'mercadopago'`, EXCEPT `null` returns
`'wompi'` (line 27: `if (!countryCode) return 'wompi'`). The test expected the old
removed Stripe branch.

**Additional stale references in the same file (currently PASS but misleading):**

- Lines 72–120: `describe('Stripe Checkout (/api/checkout/stripe)', …)` — tests a
  non-existent endpoint. All 4 tests use local variables only (no stripe import).
  **Recommended: delete this entire block.**
- Line 246: `const gateway = 'stripe'` + line 251: `expect(expectedUrl).toBe('/api/checkout/stripe')`
  — hardcoded Stripe. **Rewrite** to use `'mercadopago'` or `'wompi'`.
- Lines 262–265: `mockResponse.url` contains `checkout.stripe.com` — **rewrite** to
  a MercadoPago URL or remove.
- Lines 278–299: `describe('Estado actual de cada pasarela')` includes
  `it('STRIPE: ✅ Funcional…')` (line 279) — **rewrite** or mark as not-applicable.

**Fix: Rewrite the 2 geoToGateway assertions** (lines 235–243) to match the real
gateway. **Delete the Stripe Checkout describe block** (lines 72–120). **Rewrite**
the hardcoded `gateway = 'stripe'` block (lines 245–266) to use `mercadopago`.
**Remove the Stripe status line** from the final describe block (line 279–284).

> Note: `checkout-integration.test.ts:61` (`const plan = PLANS.find((p) => p.id ===
  'nonexistent')`) is a **typecheck error** (TS exit 2), not a runtime test failure.
  `PLANS[0].id` is typed `'basico' | 'pro' | 'premium'`, so `'nonexistent'` has no
  overlap. This surfaces in `tsc --noEmit` but does NOT fail `bun run test` (vitest
  strips types via esbuild). If the task scope includes typecheck cleanup, change
  `'nonexistent'` to `'nonexistent' as string` or use a variable.

### Files that do NOT need changes:
- `web/src/test/payments/subscription-lifecycle.test.ts` — passes, tests real lifecycle
- `web/src/test/payments/webhook-flows.test.ts` — passes, tests Wompi/MP flow mapping
- `web/src/test/payments/mercadopago.test.ts` — passes
- `web/src/test/payments/mercadopago-preference.test.ts` — passes
- `web/src/test/payments/verify-mercadopago.test.ts` — passes
- `web/src/test/payments/cron-billing.test.ts` — passes
- `web/src/lib/payments/gateway.ts` — source is correct (`'wompi' | 'mercadopago'`)
- `web/src/lib/payments/lifecycle.ts` — source is correct
- `web/src/lib/payments/wompi.ts`, `mercadopago.ts` — real modules, tests pass

## (b) 11 Uncovered Blog/SEO Scenarios → Source File Map

Derived from `openspec/changes/blog-web/verify-report.md` compliance matrix
(WARNING #1: 11 scenarios with only source/build evidence = 10 ⚠️ PARTIAL + 1 ❌ UNTESTED).
The 4 "hidden" partials are scenarios marked ✅ in the matrix whose evidence is
build-output/source-grep only (no vitest or e2e test), per the summary line
"11/27 scenarios have only source/build evidence."

| # | Scenario ID | Spec requirement | Source file(s) | Evidence gap |
|---|-------------|-----------------|----------------|--------------|
| 1 | B-R2-S1 | Draft exclusion negative path | `pages/blog/index.astro:9`, `[...slug].astro:11`, `[slug].md.ts:9` (all filter `!post.data.draft`) | No `draft:true` post in seed (11 posts, all `draft:false`); negative path untested |
| 2 | B-R3-S2 | Empty collection renders empty state | `pages/blog/index.astro:55` (`{published.length === 0 && …}`) | Seed always has 11 posts; branch never exercised at runtime |
| 3 | B-R6-S2 | Guide CTA → downloadable asset | `components/blog/CtaBlock.astro:25` (`href="/guias/guia-gestion-academica.md"`) | File exists in `web/public/guias/`; download trigger never asserted at runtime |
| 4 | B-R6-S3 | CTAs do not gate content (prose before CTA) | `[...slug].astro:125-139` (`<Content />` at line 126 precedes `<CtaBlock />` at line 139) | Verified by build render order only; no runtime DOM-order assertion |
| 5 | S-R1-S1 | Home page metadata (canonical/OG/Twitter/JSON-LD) | `components/Seo.astro:54-69`, `pages/index.astro:20-23` | Evidence: source + build grep only; no e2e or vitest asserting meta tags on `/` |
| 6 | S-R1-S2 | Post structured data (BlogPosting JSON-LD) | `components/Seo.astro:29-44` | Evidence: source + build grep only; no runtime assertion of JSON-LD on post pages |
| 7 | S-R2-S2 | Missing site fails the build | `astro.config.mjs:12` (`site: SITE_URL`) | Astro contract enforces this; not asserted by any in-repo test |
| 8 | S-R4-S3 | No cloaking (UA-insensitive identical HTML) | `middleware.ts` (no UA branching), all `pages/` | Source audit only; no runtime UA-comparison test |
| 9 | S-R5-S1 | Styled article content (typography) | `styles/global.css:3` (`@plugin "@tailwindcss/typography"`), `[...slug].astro:125` (`prose prose-invert prose-emerald`) | Evidence: build output only; no runtime test asserting prose classes |
| 10 | S-R5-S2 | No regression on existing prose pages | `pages/faq.astro`, `pages/tutorials/[...slug].astro` | Evidence: build output only; no runtime visual/textual regression check |
| 11 | S-R7-S1 | Production URLs use SITE_URL | `astro.config.mjs:9-12`, `lib/site.ts:8-9` | Tests run without env; no runtime/build test with `SITE_URL` set to a real domain |

### Test approach per scenario (recommended):

1. **B-R2-S1** → Vitest: mock `getCollection('blog')` to include a `draft:true` post, assert it's excluded from the filtered list. Or e2e: add a draft fixture, assert 404 on its slug + absent from `/blog`.
2. **B-R3-S2** → Vitest: mock `getCollection('blog')` to return `[]`, import-call the listing filter logic, assert empty-state branch is reachable. Or e2e: temp branch with zero posts.
3. **B-R6-S2** → E2e: `request.get('/guias/guia-gestion-academica.md')` → 200, `Content-Type` contains `text/markdown`, body contains expected heading.
4. **B-R6-S3** → E2e: on a blog post page, locate the `article .prose` element and the `CtaBlock section` element; assert prose's `boundingBox().y < CtaBlock's .y` (DOM order).
5. **S-R1-S1** → E2e: `page.goto('/')`, assert `link[rel=canonical]`, `meta[property=og:title]`, `meta[name=twitter:card]`, `script[type=application/ld+json]` all present with correct absolute URLs.
6. **S-R1-S2** → E2e: `page.goto('/blog/{slug}')`, parse the JSON-LD `<script>`, assert `@type === 'BlogPosting'`, `datePublished`, `author`, `url` present.
7. **S-R2-S2** → Build-level: `SITE_URL="" bun run build` must exit non-zero. Can be a shell test or a dedicated CI assertion script (not vitest).
8. **S-R4-S3** → E2e: `request.get('/blog/{slug}')` with browser UA vs `GPTBot` UA, assert HTML body is identical (or at least same content length + same headings).
9. **S-R5-S1** → E2e: `page.goto('/blog/{slug}')`, assert `article` has class containing `prose`.
10. **S-R5-S2** → E2e: `page.goto('/faq')`, assert `prose` classes present; `page.goto('/tutorials/{slug}')`, same.
11. **S-R7-S1** → Build-level: `SITE_URL="https://academix.example.com" bun run build`, then grep `sitemap-0.xml` / `robots.txt` / `llms.txt` for `https://academix.example.com`.

## (c) Web Test Conventions

| Aspect | Detail |
|--------|--------|
| Test runner | `vitest` (config: `web/vitest.config.ts`) |
| Environment | `environment: 'node'` (NOT jsdom) |
| Globals | `globals: true` — no import of `describe`/`it`/`expect` needed |
| Setup file | `web/src/test/setup.ts` (currently empty; "additional global mocks") |
| Include pattern | `src/**/*.{test,spec}.{ts,tsx}` |
| Test location | `web/src/test/` — root-level tests (e.g. `blog-schema.test.ts`, `site.test.ts`, `headings.test.ts`); `web/src/test/payments/` — subdirectory for payment-specific |
| Run command | `bun run test` (= `vitest run`, all tests) ; `bunx vitest run <file>` for single |
| E2E runner | Playwright (`@playwright/test`); config at `web/playwright.config.ts`; runs against Astro dev (`:4321`) via `webServer` |
| E2E location | `web/tests/e2e/` (e.g. `blog.spec.ts`) |
| Build-gate scripts | `web/scripts/assert-build-output.mjs` (runs `node scripts/assert-build-output.mjs`; checks sitemap, .md endpoints, robots, llms.txt, zero JS on blog, localhost fallback) — this is a runtime test (node script) |
| Build command | `bun run build` (= `astro build`) |
| Typecheck | `bunx tsc --noEmit` (separate from `bun run test`) |

### Existing passing blog/SEO test examples (patterns to follow):
- `web/src/test/blog-schema.test.ts` — vitest, imports `blogEntrySchema` from `../lib/blog-schema`, uses `safeParse`/`parse`, 8 tests
- `web/src/test/headings.test.ts` — vitest, imports `slugify`/`extractHeadings` from `../lib/headings`, 7 tests
- `web/src/test/site.test.ts` — vitest, uses `vi.resetModules()` + `vi.stubEnv()` pattern to control `process.env.SITE_URL`, re-imports `../lib/site`, 6 tests
- `web/tests/e2e/blog.spec.ts` — Playwright, tests blog listing/post/SEO endpoints, 8 tests (public prerendered routes, no env gating)

### Can `assert-build-output.mjs` be unit-tested directly?
**Yes, but with caveats.** It's a plain Node script (`node scripts/assert-build-output.mjs`) that reads `.vercel/output/static`. It can be imported as a module in vitest, but it calls `process.exit(1)` on failure (line 95), which would kill the vitest process. To unit-test it, wrap it in a child process or mock `process.exit`. Simpler: treat it as a build-gate script (run via shell in CI) rather than a vitest. The existing pattern is `bun run assert:build` as a separate script.

## (d) Approach Comparison

### Stale test cleanup

| Approach | Pros | Cons | Effort |
|----------|------|------|--------|
| **A. Delete both files entirely** | Simplest; zero stale Stripe references remain | Loses 6 passing lifecycle tests + ~11 passing Wompi/MP smoke tests that cover real code | Low |
| **B. Surgically remove Stripe blocks, rewrite geoToGateway** | Preserves all valid coverage; removes only genuinely stale code | More edits; checkout-integration.test.ts needs multiple targeted changes | Medium |
| **C. Rewrite geoToGateway assertions only (minimal)** | Fixes the 2 actual failures; lowest-effort fix | Leaves misleading Stripe Checkout describe block + hardcoded `gateway='stripe'` in place (still confusing for future devs) | Low |

### Scenario test additions

| Approach | Pros | Cons | Effort |
|----------|------|------|--------|
| **A. 11 new spec files** (one per scenario) | Crystal-clear ownership; easy gap tracking | Proliferation of tiny files; some scenarios (S-R2-S2, S-R7-S1) are build-level, don't fit vitest; doesn't follow existing file grouping conventions | High |
| **B. Extend existing files only** | Fewest new files; follows existing patterns | Some scenarios (build-level, e2e-only) don't fit neatly into existing vitest files; mixing concerns | Medium |
| **C. Hybrid** — extend `blog.spec.ts` (e2e) for B-R3-S2/B-R6-S2/B-R6-S3/B-R2-S1/S-R4-S3/S-R5-S1/S-R5-S2; extend `site.test.ts` (vitest) for S-R7-S1; extend `blog.spec.ts` for S-R1-S1/S-R1-S2; add build-gate script for S-R2-S2 | Logical grouping; addresses all 11 with appropriate test type (unit/e2e/build); follows the verify report's own suggestion (SUGGESTION #2) | Need to decide which scenarios go where; 4 scenarios need build-level or env-var tests that don't fit vitest | Medium |

## Recommendation

### Stale tests: **Approach B (surgical removal + rewrite)**
- `payments.test.ts`: Delete the `Stripe Webhook Signature` describe block (lines 10–113). Keep `Subscription Lifecycle`.
- `checkout-integration.test.ts`: Rewrite 2 `geoToGateway` assertions (`US → 'mercadopago'`, `null → 'wompi'`); delete the `Stripe Checkout` describe block (lines 72–120); rewrite the hardcoded `gateway = 'stripe'` block (lines 245–266) to use `'mercadopago'`; remove the Stripe status line (line 279–284).
- This preserves 17 passing tests covering real `lifecycle.ts` + Wompi/MP gateway logic while fixing all 7 failures. **Do NOT add Stripe.**

### Scenario tests: **Approach C (hybrid)**
- **Vitest** (`site.test.ts`): S-R7-S1 — add test with `vi.stubEnv('SITE_URL', 'https://academix.example.com')` + `vi.resetModules()`, assert `absoluteUrl` produces the production domain.
- **E2e** (`blog.spec.ts`): B-R2-S1 (draft fixture → 404/absent), B-R3-S2 (mock-empty via build or a `draft:true`-only branch — see note below), B-R6-S2 (download guide PDF/MD), B-R6-S3 (DOM order: prose before CTA), S-R1-S1 (homepage meta tags), S-R1-S2 (post JSON-LD BlogPosting), S-R4-S3 (UA-parity identical HTML), S-R5-S1 (prose class on article), S-R5-S2 (prose class on FAQ + tutorials).
- **Build-level** (assert-build-output.mjs or a new `scripts/assert-site-build.mjs`): S-R2-S2 — `SITE_URL="" bun run build` must exit non-zero (Astro's sitemap integration throws without `site`).
- **Note on B-R3-S2 (empty state)**: With 11 seed posts, the empty-state branch (`index.astro:55`) is unreachable in normal e2e. Options: (1) extract the filter to a `lib/blog-listing.ts` unit-testable function (clean architectural improvement), or (2) a one-off build test with a temporary empty collection. Recommend option 1 if the team wants clean separation; otherwise mark as "build-evidence only" and accept ⚠️.

### Additional cleanup (not test, but mentioned in verify report):
- `.env.example` line 22: `SITE_URL=https://tu-dominio.vercel.app` — refresh to a proper placeholder or remove (task says S-R7 task was "marked complete" but the file wasn't edited).
- `.env.example` lines 4–6: stale `STRIPE_*` keys — remove since Stripe is not implemented.

## Risks
- **B-R3-S2 empty state**: Cannot be tested via e2e with the current seed without either a temp-empty build or extracting listing logic to a unit-testable module. If neither is acceptable, this scenario stays ⚠️.
- **S-R2-S2 (build failure)**: Astro's `@astrojs/sitemap` throws at build time when `site` is undefined. Testing this requires running `bun run build` with `SITE_URL=""` — a build-level test, not a vitest. Must run in CI, not in `vitest run`.
- **S-R7-S1 (production SITE_URL)**: Tests run in an env without `SITE_URL` set (vitest config doesn't inject it). A vitest with `vi.stubEnv` + `resetModules` can test `absoluteUrl` logic, but cannot test that the full Astro build emits correct sitemap URLs — that requires a build-level test with `SITE_URL` set.
- **checkout-integration.test.ts**: The file has many smoke tests that pass by testing local variables (not real endpoints). Rewriting them to test real gateway behavior would require mocking `wompi`/`mercadopago` modules properly — significant effort. Recommend keeping the smoke tests but fixing the 2 geoToGateway assertions + removing the Stripe block.
- **S-R5 tests**: Asserting `prose` classes via e2e is fragile if Tailwind purges CSS in production builds. Better: assert the class string is in the SSR HTML (it will be, since Astro outputs static HTML with classes).
- **Stale .env.example Stripe keys**: If any test or build reads `.env.example`, removing STRIPE lines is safe (they're never loaded). Confirmed no code reads `.env.example`.

## Affected Areas
- `web/src/test/payments.test.ts` — delete Stripe webhook block (lines 10–113)
- `web/src/test/payments/checkout-integration.test.ts` — rewrite 2 geoToGateway assertions + delete Stripe Checkout block + fix hardcoded gateway
- `web/tests/e2e/blog.spec.ts` — add e2e tests for B-R2-S1, B-R3-S2, B-R6-S2, B-R6-S3, S-R1-S1, S-R1-S2, S-R4-S3, S-R5-S1, S-R5-S2
- `web/src/test/site.test.ts` — add S-R7-S1 test (production SITE_URL via stubEnv)
- `web/scripts/assert-build-output.mjs` — add S-R2-S2 build-failure assertion (or new script)
- `web/src/.env.example` — remove stale STRIPE_* lines + refresh SITE_URL placeholder (optional, low-risk)
- `web/src/lib/payments/gateway.ts` — no changes (correct as-is); reference for rewriting tests

## Ready for Proposal
Yes. The fixes are low-risk and well-scoped: (1) remove/rewrite clearly stale Stripe test code that references a non-existent module, and (2) add 11 runtime tests following the established vitest (node env) + Playwright e2e + build-gate script patterns. The orchestrator should tell the user: the parent blog-web change is sound (0 defects); the remaining work is test cleanup (remove pre-existing Stripe-stale assertions) + closing 11 evidence gaps, and ask whether to extract the empty-state branch into a unit-testable module (affects B-R3-S2 test strategy).