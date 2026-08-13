# Apply Progress: MercadoPago Payment Persistence (verify-mercadopago)

**Date**: 2026-08-13
**Mode**: Standard
**Delivery**: single PR — work-unit commits, **size-exception recorded** (final authored diff 466 changed lines, >400 forecast)

## Status

**6/6 tasks complete.** All apply work for `mercado-pago-persistence` is DONE. Ready for `sdd-verify`.

## Completed Tasks

| # | Task | Status | Evidence |
|---|------|--------|----------|
| T1 | Create `web/src/pages/api/payments/verify-mercadopago.ts` (GET) | DONE | 12/12 focused tests green; endpoint per D5 contract |
| T2 | Extend `activateApprovedPayment` with optional `expectedUserId` guard | DONE | Guard early-returns before SELECT; webhook callers unchanged (webhook-flows 15/15 green) |
| T3 | Replace inline MP verification in `web/src/pages/dashboard/index.astro` | DONE | No `MP_API_URL`/`MP_ACCESS_TOKEN`/`activateApprovedPayment` remain; astro build passes |
| T4 | Create `web/src/test/payments/verify-mercadopago.test.ts` | DONE | 5 suites, 12 tests, all green; idempotency asserts INSERT count === 1 |
| T5 | Document MP env vars in `web/docs/vercel-rollout.md` | DONE | Both `MP_WEBHOOK_SECRET` and `MP_ACCESS_TOKEN` documented |
| T6 | Verify web suite | DONE | See Work Unit Evidence below |

## Work Unit Evidence (Hard Gate — all modes)

| Unit | Focused test command and exact result | Runtime harness command/scenario and exact result | Rollback boundary |
|------|--------------------------------------|---------------------------------------------------|-------------------|
| T1+T2 (endpoint + guard) | `bun run test -- src/test/payments/verify-mercadopago.test.ts` → **12 passed / 0 failed**; `bun run test -- src/test/payments/webhook-flows.test.ts` → **15 passed / 0 failed** (guard absent in webhook callers, unchanged) | `bun run build` (astro, in `web/`) → **Complete!** — dashboard + endpoint compile; endpoint is `prerender = false`, exercised through the mocked suite | Delete `verify-mercadopago.ts`; drop `expectedUserId` from `activateApprovedPayment` (webhook never passes it) |
| T3 (dashboard wiring) | `bunx tsc --noEmit` (from `web/`) → **0 new errors** (8 pre-existing, proven pre-existing via stash test) | `bun run build` (astro) → **Complete!** — modified `index.astro` frontmatter compiles; same-origin `fetch` to verify endpoint sends session cookie | Restore inline MP block in `index.astro` (git revert of the section); endpoint stays as fallback |
| T4+T5 (tests + docs) | `bun run test -- src/test/payments/verify-mercadopago.test.ts` → **12 passed / 0 failed** (also green inside the full suite run) | N/A — Vitest mocks external HTTP; no runtime boundary beyond unit suites | Delete test file; revert `vercel-rollout.md` env rows |

## T6 Full-Suite Verification

Commands (run from `web/`):

1. `bun run test -- src/test/payments/verify-mercadopago.test.ts` → **12 passed / 0 failed**
2. `bunx tsc --noEmit` → **0 errors attributable to this change** (8 pre-existing errors, proven pre-existing by stashing this change and re-running — identical 8)
3. `bun run test` (full web suite) → **168 passed / 7 failed / 0 skipped**

### Pre-existing failures (documented, NOT fixed — out of scope per T6)

| File | Failures | Root cause |
|------|----------|------------|
| `src/test/payments/payments.test.ts` | 5 | Imports non-existent `../../lib/payments/stripe` module — Stripe is deliberately not implemented (AGENTS.md: "only a legacy `stripe_subscription_id` column and tests importing a non-existent `lib/payments/stripe` module — do not add Stripe without updating `gateway.ts`") |
| `src/test/payments/checkout-integration.test.ts` | 2 | Tests expect `geoToGateway('US')` → `'stripe'` and `geoToGateway(null)` → `'stripe'`, but real `gateway.ts` routes non-CO → `'mercadopago'` and null → `'wompi'` — tests written for an older Stripe-inclusive design, never updated |

Neither failing file is touched by this change; the 7 failures are identical to the pre-change baseline (same files, same assertions, no MP dependency).

## Files Changed

| File | Action | What Was Done |
|------|--------|---------------|
| `web/src/pages/api/payments/verify-mercadopago.ts` | Created | Owner-checked GET endpoint: JWT via `getFullTokenPayload`; `payment_id` (alias `collection_id`) + optional `external_reference`; MP API `getPayment`; D5 contract (401/400/403/502/400-status/200) |
| `web/src/lib/payments/mercadopago.ts` | Modified | `ApprovedPaymentInput.expectedUserId?: string`; early-return guard when set and `!externalReference.startsWith(expectedUserId)` — before replay SELECT |
| `web/src/pages/dashboard/index.astro` | Modified | Replaced inline MP fetch + direct `activateApprovedPayment` with same-origin `fetch('/api/payments/verify-mercadopago?...')`; `paymentSuccess` only on `{ success: true }`; 401/403/502/pending/rejected-cancelled messages; catch → webhook fallback |
| `web/src/test/payments/verify-mercadopago.test.ts` | Created | 5 suites (happy path incl. `expectedUserId: payload.sub` assertion, auth 401, ownership 403, idempotency INSERT count === 1, MP API failure 502) — 12 tests |
| `web/docs/vercel-rollout.md` | Modified | Payment env-var section: `MP_WEBHOOK_SECRET` (signature verification; failure mode 500 + MP retries; verify endpoint as fallback) and `MP_ACCESS_TOKEN` (verify dependency) |

## Deviations from Design

None — implementation matches design. The D5 status contract and T1–T6 acceptance criteria were followed exactly. One note: the dashboard no longer consumes the `status` query param directly (removed `mpStatus`) — the endpoint derives status from the MP API, matching the design's redirect handling.

## Issues Found

- **Pre-existing (documented, not fixed)**: 8 tsc errors + 7 failing tests as tabulated above — all unrelated to this change (Stripe module absence, stale `geoToGateway` test expectations).
- No issues in the authored change itself.

## Workload / PR Boundary

- Mode: single PR with work-unit commits (per tasks.md suggested split; `Chained PRs recommended: No`)
- Size: **size-exception recorded** — final authored diff = 437 additions + 29 deletions = **466 changed lines** (forecast was ~425, budget risk Medium; decision pre-resolved in tasks.md)
- Chain strategy: `size-exception` (no chain needed — single PR to main)
- Work units (commits per tasks.md commit plan, each independently revertible):
  1. `feat(web): guard MercadoPago activation with optional expectedUserId` → T2
  2. `feat(web): add owner-checked verify-mercadopago redirect endpoint` → T1
  3. `test(web): cover verify-mercadopago auth, ownership, idempotency, MP failures` → T4
  4. `feat(web): verify MercadoPago redirects via endpoint in dashboard` → T3
  5. `docs(web): document MP_WEBHOOK_SECRET and MP_ACCESS_TOKEN env vars` → T5
  6. T6 — verification only, no commit

## Notes for Verify

- The 8 tsc errors and 7 test failures are the pre-existing baseline — verify should not attribute them to this change (stash-proof: `git stash` of this change reproduces identical 8 tsc errors).
- New code touches only the Mercado Pago persistence path; webhook trust boundary unchanged (guard is caller-supplied, webhook never passes it).
