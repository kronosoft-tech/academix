# Tasks: payments-webhook-and-errors

Retrospective breakdown for commit `42c2d86` — implementation is ALREADY committed and review-approved. **Apply is NOT needed**; the change closes with verification + archive. Each task maps to a spec requirement and the committed code. `[x]` = implemented in `42c2d86`; `[ ]` = follow-up, NOT implemented.

## Review Workload Forecast (retrospective)

| Field | Value |
|-------|-------|
| Estimated changed lines | ~715 implementation+docs (844 incl. SDD artifacts) |
| 400-line budget risk | High (715 > 400) |
| 800-line budget | Within (715 < 800) |
| Chained PRs recommended | Yes |
| Suggested split | 8 work units → 3 stacked PRs |
| Delivery strategy | auto (retrospective — apply skipped) |

```text
Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High
```

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Wompi checksum | PR 1 | `bun run build`; unit pending FU1 | Replay real event+tunnel: valid→200, tampered→401 | Revert `wompi.ts` checksum fn + webhook guard |
| 2 | Wompi ownership | PR 1 | `bun run build`; unit pending FU1 | verify-wompi: foreign ref→400, own→200 | Revert verify-wompi + dashboard banner + widget call |
| 3 | MP preference | PR 2 | `bun run build` | checkout MP: TEST-→200 url; bad token→502+detail | Revert `createPreference` + checkout handler |
| 4 | MP HMAC + retry | PR 2 | `bun run build`; unit pending FU1 | forged x-signature→401; failing activation→500 | Revert HMAC fn + webhook POST guard |
| 5 | Replay guard | PR 2 | `bun run build`; unit pending FU1 | replay payment_id twice→1 row | Revert activate/recordPayment guards |
| 6 | Lazy trial row | PR 3 | `bun run build` | desktop-first user pays→trial row created | Revert `getOrCreateTrialSubscription` + callers |
| 7 | Dashboard SSR | PR 3 | `bun run build` | `/dashboard?id=` + `?payment_id=` redirects | Revert dashboard redirect block |
| 8 | Schema alignment | PR 3 | `bun test src/test/payments.test.ts src/test/payments/subscription-lifecycle.test.ts` | N/A — SQL-string contract proven by mock-db assertions | Revert lifecycle.ts SQL + tests + AGENTS.md |

## Work Unit 1 — Wompi webhook checksum (DONE) [spec: Wompi Webhook Signature Verification]

- [x] 1.1 `web/src/lib/payments/wompi.ts`: `verifyWebhookSignature()` = SHA-256 over values of `signature.properties` resolved in order (`resolveEventProperty()`) + `event.timestamp` + `WOMPI_EVENTS_SECRET`; fallback `['transaction.id','transaction.status','transaction.amount_in_cents']` when properties absent.
- [x] 1.2 `web/src/pages/api/webhooks/wompi.ts`: read `x-event-checksum`; missing or non-matching → 401 with no processing; valid → first-payment/renewal/grace branches.

## Work Unit 2 — Wompi ownership guard (DONE) [spec: Wompi Transaction Ownership Verification]

- [x] 2.1 `web/src/pages/api/payments/verify-wompi.ts`: fetch `/transactions/{id}` with `Authorization: Bearer ${WOMPI_PUBLIC_KEY}`; non-ok → 502 + `console.error`; non-APPROVED → 400; `parts.slice(0,5).join('-') !== payload.sub` → 400.
- [x] 2.2 `web/src/pages/dashboard/index.astro` Wompi branch (`?id=`): Bearer fetch, ownership check → "Este pago no corresponde a tu cuenta." banner; PENDING/DECLINED copy; activate + idempotent payment only when owned + APPROVED.
- [x] 2.3 `web/src/components/CheckoutPlans.tsx` widget callback: on APPROVED, POST `/api/payments/verify-wompi` before redirect to `/dashboard?id=` (dashboard stays as fallback).

## Work Unit 3 — MP Checkout Pro preference creation (DONE) [spec: Checkout Pro Preference Creation]

- [x] 3.1 `web/src/lib/payments/mercadopago.ts` `createPreference()`: POST `/checkout/preferences` with items, payer, `external_reference`, `auto_return:'approved'`, `notification_url`; parse MP error body, rethrow with `status` + `detail`.
- [x] 3.2 `web/src/pages/api/checkout/mercadopago.ts`: `external_reference = {sub}-{planId}-{uuid}`; 401 unauth / 400 bad JSON+plan / 500 MP not configured / 502 + `detail` on failure; `sandbox_init_point` for `TEST-` tokens, else `init_point`.
- [x] 3.3 `CheckoutPlans.tsx`: non-ok checkout response surfaces `data.detail || data.error` in the error banner.

## Work Unit 4 — MP webhook HMAC + retry semantics (DONE) [spec: Webhook Signature Verification and Payment Processing]

- [x] 4.1 `web/src/lib/payments/mercadopago.ts` `verifyWebhookSignature()`: parse `x-signature` `ts`/`v1`; HMAC-SHA256 over `id:<data.id>;request-id:<x-request-id>;ts:<ts>;` with `MP_WEBHOOK_SECRET`; `timingSafeEqual` comparison.
- [x] 4.2 `web/src/pages/api/webhooks/mercadopago.ts` POST: invalid JSON → 400; secret unset → 500; invalid signature → 401; verified `payment.created/updated` or `topic=payment` → `processPayment()`; thrown processing → 500 + log (MP retries); non-approved / no `external_reference` → ack 200 without activating.

## Work Unit 5 — Replay-safe activation (DONE) [spec: Replay-Safe Payment Activation]

- [x] 5.1 `mercadopago.ts` `activateApprovedPayment()`: early-return when `provider_payment_id` exists; malformed reference (<6 parts) → no-op; resolve sub via `provider_subscription_id = external_reference` else lazy trial row; activate + idempotent payment insert.
- [x] 5.2 `webhooks/wompi.ts` `recordPayment()` and `verify-wompi.ts` payment insert: skip insert when `provider_payment_id` already recorded (webhook renewal, webhook first-payment, verify, dashboard paths).

## Work Unit 6 — Lazy trial row (DONE) [spec: Lazy Trial Subscription Creation]

- [x] 6.1 `web/src/lib/payments/lifecycle.ts`: `getOrCreateTrialSubscription()` — existing trial row, else any row, else `createTrialSubscription` (web-schema columns `plan_id`, `trial_starts_at`, `grace_expires_at`).
- [x] 6.2 Wire into all 5 activation paths: verify-wompi.ts, webhooks/wompi.ts first-payment, checkout/mercadopago.ts, `activateApprovedPayment`, dashboard SSR Wompi branch.

## Work Unit 7 — Dashboard SSR redirect handling (DONE) [spec: Dashboard Checkout Pro Redirect Handling]

- [x] 7.1 `web/src/pages/dashboard/index.astro`: handle Wompi `?id=` (Bearer + ownership + activate) and MP `payment_id`/`collection_id` (fetch `/v1/payments/{id}`, `activateApprovedPayment` when approved + `external_reference`); pending informational copy; `preapproval_id` block removed.

## Work Unit 8 — lifecycle.ts schema alignment + tests + docs

- [x] 8.1 `lifecycle.ts`: align SELECT lists + `activateSubscription`/`startGracePeriod`/`getExpiredGraceSubscriptions` SQL to web-schema columns (`plan_id`, `trial_starts_at`, `grace_expires_at` — was `plan`/`grace_start`/`grace_end`).
- [x] 8.2 Update `web/src/test/payments.test.ts` and `web/src/test/payments/subscription-lifecycle.test.ts` to assert new column names and shifted arg positions.
- [x] 8.3 `AGENTS.md`: document gateway scope (Wompi + MercadoPago only; Stripe not implemented), `.env.example` reality, `wompi`/`mercadopago` skills.

## Follow-up — NOT implemented (future work, outside `42c2d86`)

- [ ] FU1 RED unit tests for security-critical functions — Wompi checksum (valid/missing/non-matching), MP HMAC (valid/forged/tampered ts), replay guard (double insert), ownership check (foreign reference). Design open question; gap confirmed.
- [ ] FU2 Disable the MP legacy GET IPN path (`webhooks/mercadopago.ts` GET) or add signature verification — currently unverified by design.
- [ ] FU3 Fix `charge-wompi` cron: `s.plan` select (no `plan` column) + stale `getPlanPriceCOP` prices.
- [ ] FU4 (optional) Extract duplicated `WOMPI_API_URL`/`WOMPI_PUBLIC_KEY` consts from `wompi.ts` / `verify-wompi.ts` / dashboard SSR into one module.
