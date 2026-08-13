# Design: Secure Payment Activation and Webhook Error Handling

## Technical Approach

Retrospective design for `42c2d86` (`web/`, Astro 7 SSR): hardening of both gateways — no re-architecture, no DDL. (1) authenticate before writing (Wompi checksum+ownership, MP HMAC); (2) idempotent writes (replay guard, lazy trial row); (3) fail loudly (status + log).

## Architecture Decisions

### Ownership proof via reference embedding

| Option | Tradeoff | Decision |
|---|---|---|
| Reference = `{userId}-{planId}-{uuid}`; userId = first 5 UUID parts, planId = parts[5] | Needs tamper-proof reference (checksum + Bearer) | Adopted in checkout, verify-wompi, dashboard SSR |
| Server-side user lookup | Extra round-trip; id not in transaction | Rejected |

`verify-wompi` rejects with 400 when `parts.slice(0, 5).join('-') !== payload.sub`; the dashboard shows an error banner (no HTTP status).

### Wompi checksum per `event.signature.properties`

| Option | Tradeoff | Decision |
|---|---|---|
| Hardcoded `id+status+reference` hash | Rejected every real event (401) | Rejected |
| Resolve `signature.properties` in order + `timestamp` + secret, SHA-256 | Per spec | Adopted; fallback `[id, status, amount_in_cents]` |

### Bearer-auth fetches with 502 surfacing

Fetches send `Authorization: Bearer ${WOMPI_PUBLIC_KEY}`; non-ok → 502 + `console.error` (was unauthenticated → silent no-op).

### MP `x-signature` HMAC

| Option | Tradeoff | Decision |
|---|---|---|
| Trust any POST | Forgeable/replayable | Rejected |
| HMAC-SHA256 over `id:<data.id>;request-id:<x-request-id>;ts:<ts>;` with `MP_WEBHOOK_SECRET`, `timingSafeEqual` | Matches MP docs; new env secret | Adopted |

### 500-on-failure webhook retry semantics

| Option | Tradeoff | Decision |
|---|---|---|
| 200 always (old) | MP never retries; lost activations | Rejected |
| 500 when `processPayment` throws | MP retries; activation idempotent | Adopted |

### Replay guard on `provider_payment_id`

`activateApprovedPayment` early-returns when `provider_payment_id` exists; same guard in Wompi `recordPayment`, dashboard, verify.

### Lazy trial row (`getOrCreateTrialSubscription`)

| Option | Tradeoff | Decision |
|---|---|---|
| Require existing trial row | Desktop-first users silent no-op | Rejected |
| trial → any row → create trial row | Small first-payment insert | Adopted in all 5 paths |

### Dashboard SSR redirect handling

Wompi `?id=` (Bearer + ownership + PENDING/DECLINED copy) and MP `payment_id`/`collection_id` — `preapproval_id` block removed. Approved → `activateApprovedPayment`.

### Checkout Pro with `external_reference`

`createPreference`: `auto_return=approved`, `external_reference = {sub}-{plan}-{uuid}`, `notification_url`; reference saved as `provider_subscription_id` pre-redirect. `sandbox_init_point` for `TEST-` tokens; rejection → 502 with `detail` (shown via `data.detail || data.error`).

### Error taxonomy

| Endpoint | 400 | 401 | 500 | 502 |
|---|---|---|---|---|
| verify-wompi | bad JSON / missing id / not approved / ownership mismatch | unauthenticated | — | Wompi fetch failed |
| webhook wompi | bad JSON | missing / invalid `x-event-checksum` | — | — |
| checkout mercadopago | bad JSON / invalid plan | unauthenticated | MP not configured | MP rejected (+ detail) |
| webhook mercadopago | bad JSON | invalid signature | secret unset / processing failed | — |

## Data Flow

```
CheckoutPlans.tsx ─► POST /api/checkout/{gateway} ─► gateway API
  ├─ Wompi widget ─► verify-wompi ─► activate+payment (fallback /dashboard?id=)
  └─ MP redirect ─► /dashboard?payment_id= ◄─► activateApprovedPayment
MP webhook (HMAC) ─► processPayment ─► activateApprovedPayment
Wompi webhook (checksum) ─► activate / record / grace (renewals)
```

## Data Model / Schema Alignment

No DDL. Columns (`001`/`002`): `plan_id`, `trial_starts_at`, `grace_expires_at`, `provider_payment_id`. `lifecycle.ts` queries aligned (was `plan`/`grace_end`); Wompi webhook sets `plan_id` from parts[5] (default `basico`).

## Module / Endpoint Map

| Entry | Handler | Service | Notes |
|---|---|---|---|
| `POST /api/payments/verify-wompi` | `pages/api/payments/verify-wompi.ts` | inline fetch (`lib/auth`, `lib/payments/lifecycle`, `lib/db`) | ownership + Bearer; widget calls it |
| `POST /api/webhooks/wompi` | `pages/api/webhooks/wompi.ts` | `wompi.ts` verify; lifecycle | checksum, `plan_id`, renewal/grace |
| `POST /api/checkout/mercadopago` | `pages/api/checkout/mercadopago.ts` | `mercadopago.ts` createPreference | lazy row, 502+detail |
| `POST / GET /api/webhooks/mercadopago` | `pages/api/webhooks/mercadopago.ts` | verify / getPayment / activateApprovedPayment | 500-on-failure retry |
| `/dashboard` SSR | `pages/dashboard/index.astro` | both gateways inline | Wompi `id`, MP `payment_id`/`collection_id` |

## Testing Strategy

| Layer | What | How |
|---|---|---|
| Unit (lifecycle SQL contract) | `plan_id` / `trial_starts_at` / `grace_expires_at` | Mock-db assertions in `test/payments*.test.ts` |
| Gap — security-critical | checksum / HMAC / replay / ownership | None in `42c2d86`; review-only |

## Threat Matrix

N/A — no routing/shell/subprocess/VCS/PR/executable-file/process-integration boundary; webhook ingress covered above.

## Migration / Rollout

No migration. New env `MP_WEBHOOK_SECRET`. Rollback: revert `42c2d86` (queries/handlers only).

## Open Questions

- [ ] MP legacy GET IPN (`?topic=payment&id=`) unverified by design — consider disabling.
- [ ] Wompi checksum compare is non-timing-safe string equality (MP uses `timingSafeEqual`).
- [ ] `planId` extraction is positional (`parts[5]`) — breaks if a plan id contains `-`.
- [ ] Concurrent checkouts race on the single trial row (reference overwrite).
- [ ] Wompi webhook URL manual (dashboard + tunnel); `charge-wompi` cron (`s.plan`, stale prices) — follow-up.
- [ ] Add RED unit tests for checksum / HMAC / replay guard / ownership.
