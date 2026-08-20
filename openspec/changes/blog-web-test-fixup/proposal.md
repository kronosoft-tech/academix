# Proposal: Clean stale Stripe tests + close 11 blog/SEO scenario gaps

## Intent

`blog-web` is functionally sound (0 new defects; build ✅, e2e 8/8 ✅, blog/SEO unit 21/21 ✅) but `sdd-verify` FAILed on two pre-existing, implementation-unrelated gates: (1) `bun run test` exits 1 — 7 stale tests assert a `stripe` gateway never implemented (`Gateway = 'wompi' | 'mercadopago'` only); (2) 11 blog/SEO scenarios have only source/build evidence (10 ⚠️ + B-R3-S2 ❌). Removes stale Stripe assertions, adds 11 missing runtime tests to unblock archive.

## Goals / Non-Goals
- **Goals**: `bun run test` exits 0 (7→0); close 11 evidence gaps; preserve 17 lifecycle/Wompi/MP tests.
- **Non-goals**: no new payments, no Stripe, no `gateway.ts` change, no blog UX, no spec changes.

## Scope Boundaries
- **In**: delete Stripe-stale blocks + rewrite 2 `geoToGateway` + 11 runtime tests.
- **Out**: `Footer.tsx` Blog→`/tutorials`; TS typo at `checkout-integration.test.ts:61`; stale `.env.example` STRIPE keys.

## Capabilities
- **New**: None. **Modified**: None — test-only; asserts already-specified blog/seo behavior.

## Approach
1. `payments.test.ts`: delete `Stripe Webhook Signature` block; keep `Subscription Lifecycle`.
2. `checkout-integration.test.ts`: rewrite 2 `geoToGateway` (US→mercadopago, null→wompi); delete `Stripe Checkout` block (72–120); rewrite `gateway='stripe'` (245–266)→mercadopago; drop Stripe status line.
3. Tests (explore scenario→src map): vitest (B-R2-S1, S-R7-S1); e2e (B-R3-S2, B-R6-S2, B-R6-S3, S-R1-S1, S-R1-S2, S-R4-S3, S-R5-S1/S2); build-gate (S-R2-S2).

## Affected Areas
| Area | Impact |
|------|--------|
| `web/src/test/payments.test.ts` | Modified |
| `web/src/test/payments/checkout-integration.test.ts` | Modified |
| `web/src/test/site.test.ts` | Modified |
| `web/tests/e2e/blog.spec.ts` | Modified |
| `web/scripts/assert-build-output.mjs` | Modified |
| `web/src/lib/blog-listing.ts` *(if decision (i)* | New |

## Risks
| Risk | Likelihood | Mitigation |
|------|------------|------------|
| B-R3-S2 unreachable via e2e (11-post seed) | High | See open decision (i) |
| S-R2-S2 needs empty-SITE_URL build | Med | CI build-gate, not vitest |
| Prose/DOM-order e2e under Tailwind purge | Med | Assert SSR HTML classes + DOM order |

## Open Decision (awaiting maintainer) — B-R3-S2 empty-state coverage

`{published.length === 0 && …}` in `index.astro:55` is unreachable in e2e (seed always 11 posts).
- **(i) Extract** filter into `lib/blog-listing.ts` (refactor `index.astro:5-13`), vitest with mocked empty collection — **preferred**.
- **(ii) Build-output check only**; leave B-R3-S2 ⚠️.

**Default: (i)** — no layout/UX or gateway change; drop `lib/blog-listing.ts` if (ii) preferred.

## Rollback Plan
`git revert` — test files + e2e + build-gate only. No production source under (ii); under (i) removing `lib/blog-listing.ts` restores prior behavior. Returns to 7-failure baseline.

## Dependencies
- Parent `blog-web` applied (main); `gateway.ts` read-only.

## Success Criteria
- [ ] `bun run test` exits 0 (7→0).
- [ ] B-R3-S2 passing runtime test (vitest under (i); build-gate under (ii)).
- [ ] 11/11 scenarios have a dedicated asserting test.
- [ ] `blog-web` re-verify PASS (exit 0, 27/27).
