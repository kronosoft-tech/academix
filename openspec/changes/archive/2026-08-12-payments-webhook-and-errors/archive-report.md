# Archive Report: payments-webhook-and-errors

**Archive date**: 2026-08-12
**Mode**: hybrid (OpenSpec file + Engram) — BOTH backends written
**Archived to**: `openspec/changes/archive/2026-08-12-payments-webhook-and-errors/`
**Engram**: `sdd/payments-webhook-and-errors/archive-report`

## Change Summary

Retrospective change hardening payment activation and webhook error handling in `web/` (Astro 7 SSR on Vercel). Production-blocking bugs fixed: Wompi "paid but nothing saved" (unauthenticated fetches + wrong checksum → silent no-op), MP checkout 502 hiding `detail`, unverified MP webhook (no `x-signature`), MP replayable activation, and desktop-first users silently no-opping for lack of a trial subscription row.

## Implementation

| Commit | Message | Scope |
|--------|---------|-------|
| `42c2d86` | `fix(web): secure payment activation and webhook error handling` | Main implementation (all 18 in-scope tasks) |
| `af9b316` | `fix(web): narrow MP webhook detail type (TS2322)` | FU5 fix — resolves the CRITICAL verify finding |

No migrations changed; only queries/handlers. Rollback boundary: revert `42c2d86`.

## Gates Passed

- **Task completion gate**: `tasks.md` shows 18/18 in-scope tasks `[x]`. Follow-ups FU1–FU4 are explicitly `[ ]` and verified out-of-scope by the verify report (not implementation tasks). No stale unchecked implementation tasks in the archived audit trail.
- **Verification gate**: `verify-report.md` revision 2 verdict `PASS WITH CRITICAL FINDING` (archive-with-blocker). The sole CRITICAL — `web/src/lib/payments/mercadopago.ts:88` TS2322 (`Type '{}' is not assignable to type 'string'`) — was **FIXED in `af9b316`** using the exact `in`-narrowing prescribed by the verify report (drop the type-assertion cast; `typeof errorBody.detail === 'string'` on the reference). Re-verified: `bunx tsc --noEmit` no longer reports `mercadopago.ts` errors. Blocker cleared; archive proceeds.

## Specs Synced (main specs = source of truth)

Merge base: `openspec/changes/payments-and-subscriptions/specs/` (that change was never archived; its delta specs are the latest base content for these capabilities). `openspec/specs/` previously had no payment capabilities — they were created from base + this change's deltas.

| Domain | Action | Details |
|--------|--------|---------|
| `wompi-payments` | Created (from base + delta) | 1 MODIFIED (`Wompi Webhook Signature Verification` per `signature.properties`/fallback), 2 ADDED (`Wompi Transaction Ownership Verification`, `Wompi Webhook First-Payment Activation`); preserved `Wompi Widget Checkout`, `Recurring Charge via Cron` |
| `mercadopago-payments` | Created (from base + delta) | 2 REMOVED (`Preapproval Creation`, `IPN Webhook Processing` — Reason/Migration recorded in delta), 4 ADDED (`Checkout Pro Preference Creation`, `Webhook Signature Verification and Payment Processing`, `Replay-Safe Payment Activation`, `Dashboard Checkout Pro Redirect Handling`); preserved `IPN Reconciliation`; Purpose updated preapproval → Checkout Pro |
| `subscription-lifecycle` | Created (from base + delta) | 1 ADDED (`Lazy Trial Subscription Creation`); preserved `Trial Period`, `Grace Period`, `Login Enforcement`, `Successful Payment Reactivation` |

**Merge note (preserved with caution)**: `mercadopago-payments` `IPN Reconciliation` was not mentioned in this change's delta, so it was preserved per archive rules. Its scenario still references the removed preapproval API — it is likely stale and should be reviewed in a follow-up cleanup change.

## Verification Outcome

| Metric | Result |
|--------|--------|
| In-scope tasks | 18/18 complete |
| Spec requirements | 8/8 implemented (source-verified) |
| CRITICAL findings | 1 — RESOLVED by `af9b316` (FU5) |
| Unit tests (web/) | 99 passed / 7 failed (failures pre-existing Stripe-era legacy: 5× missing `stripe` module, 2× `geoToGateway` expecting removed `'stripe'`; changed-code tests all pass) |
| Typecheck (web/) | `mercadopago.ts` errors gone post-fix; remaining errors pre-existing (untouched legacy files: `src/actions/register.ts:38`, `src/lib/db.ts:25`, missing `stripe` imports) |
| Build (web/) | PASS (`astro build`) |

## Resolved Follow-up

- **FU5 — `mercadopago.ts:88` TS2322**: RESOLVED in `af9b316` (verified above). The archive audit trail reflects the final state (implementation = `42c2d86` + `af9b316`).

## Open Follow-ups (documented, NOT blocking — schedule as a new change)

- **FU1** — RED unit tests for security-critical functions: Wompi checksum (valid/missing/non-matching), MP HMAC (valid/forged/tampered `ts`), replay guard (double insert), ownership check (foreign reference). Design gap accepted review-only for this retrospective change; schedule before further payment changes.
- **FU2** — Disable the MP legacy GET IPN path (`webhooks/mercadopago.ts` GET) or add signature verification — currently unverified by design.
- **FU3** — Fix `charge-wompi` cron: `s.plan` select (column no longer exists) + stale `getPlanPriceCOP` prices. Renewal charging via cron currently fails at runtime; the webhook renewal branch is the working path.
- **FU4** — (optional) Extract duplicated `WOMPI_API_URL`/`WOMPI_PUBLIC_KEY` consts from `wompi.ts` / `verify-wompi.ts` / dashboard SSR into one module.

## Deployment Note (required before/with release)

`MP_WEBHOOK_SECRET` is a new mandatory env var — must be set in the Vercel environment. Until configured, every MP webhook returns 500 and MP retries indefinitely. Existing vars: `WOMPI_PUBLIC_KEY`, `WOMPI_EVENTS_SECRET`, `SITE_URL`, `MP_ACCESS_TOKEN`, `MP_API_URL`, `CRON_SECRET`.

## Risks / Caveats Carried Forward

- Security-critical functions (checksum/HMAC/replay/ownership) have zero automated tests until FU1 lands — regressions detectable only by manual review.
- Full `web/` unit suite exits 1 on pre-existing Stripe-era failures; a red suite masks future regressions (clean up per AGENTS.md guidance).
- MP legacy GET IPN bypasses signature verification (FU2).
- `charge-wompi` cron broken at runtime (FU3).
- `mercadopago-payments` `IPN Reconciliation` requirement likely stale (references removed preapproval API) — review in a follow-up.
- Wompi checksum comparison is non-timing-safe string equality (low impact — no secret in the compared value).
- Positional `planId` extraction (`parts[5]`, default `basico`) breaks if a plan id contains `-`; concurrent checkouts race on the single trial row.

## SDD Cycle Complete

Change `payments-webhook-and-errors` planned, implemented (`42c2d86`), verified, defect-fixed (`af9b316`), and archived. Delta specs merged into main specs; change folder moved to `openspec/changes/archive/2026-08-12-payments-webhook-and-errors/`. Ready for the next change.
