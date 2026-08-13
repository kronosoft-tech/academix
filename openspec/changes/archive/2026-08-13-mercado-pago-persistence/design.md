# Design: MercadoPago Payment Persistence (verify-mercadopago)

## Technical Approach

Guarantee MercadoPago (MP) payment persistence in control-plane `subscription_payments` despite the webhook 500 (missing `MP_WEBHOOK_SECRET` — user action, out of scope). Mirror the proven `verify-wompi.ts` redirect pattern: a new owner-checked `GET /api/payments/verify-mercadopago` endpoint, called by the dashboard after Checkout Pro redirects, reusing `activateApprovedPayment` extended with an optional ownership guard. All paths stay idempotent (SELECT-by-`provider_payment_id` before INSERT), so webhook retries, repeated verifies, and webhook+verify races are safe. Satisfies the `payment-persistence` delta requirements.

## Architecture Decisions

### D1: New endpoint `web/src/pages/api/payments/verify-mercadopago.ts` (GET)

| Option | Tradeoff | Decision |
|--------|----------|----------|
| POST + JSON body (verify-wompi style) | Needs client-assembled body; MP redirect yields query params | Rejected |
| GET + query params | Params visible in logs; benign (no secrets) | Adopted — MP appends `?payment_id&status&external_reference` to `back_url`; spec fixes GET |

`export const prerender = false;` + `export const GET: APIRoute`. Reads `payment_id` / `external_reference` from `url.searchParams`.

### D2: Optional `expectedUserId` guard on `activateApprovedPayment`

| Caller | Guard | Why |
|--------|-------|-----|
| MP webhook handler | None | MP-signed; signature verification is the trust boundary |
| verify endpoint | `payload.sub` | Endpoint enforces ownership (D3); guard is defense-in-depth for future callers |

`activateApprovedPayment({ paymentId, externalReference, transactionAmount, currencyId, expectedUserId? })` returns early unless `externalReference.startsWith(expectedUserId)` (format `{userId}-{planId}-{uuid}`, userId = 5 UUID parts).

### D3: Strict ownership at the endpoint

| Option | Tradeoff | Decision |
|--------|----------|----------|
| No ownership check | Any authenticated user could activate another's payment | Rejected (security review R1-03/R1-04) |
| 403 when reference does not start with JWT `sub` | Stricter than verify-wompi's 400 | Adopted — spec mandates 403 |

### D4: Idempotency — SELECT before INSERT

`activateApprovedPayment` returns early when `provider_payment_id` exists and re-checks before the INSERT; `activateSubscription` is a plain status UPDATE (no double-charge effects). No schema change needed. Duplicate verify → 200 with no second row.

### D5: Error / response contract

| Case | Status | Body |
|------|--------|------|
| No valid JWT cookie | 401 | `{ success: false, message }` |
| Missing `payment_id` / `external_reference` | 400 | `{ success: false, message }` |
| Ownership mismatch | 403 | `{ success: false, message }` |
| MP API non-2xx / network error | 502 | `{ success: false, message }` + `console.error` |
| MP status ≠ `approved` | 400 | `{ success: false, message, status }` — no persistence |
| Success | 200 | `{ success: true, plan, status: 'active' }` |

`plan` parsed from `external_reference` parts[5] (same parsing as `activateApprovedPayment`).

## Data Flow

```
MP redirect → /dashboard?payment_id&status&external_reference
   → fetch GET /api/payments/verify-mercadopago (same-origin; cookies auto-sent)
     1. getFullTokenPayload(cookies) → 401
     2. parse query params           → 400
     3. reference.startsWith(sub)    → 403
     4. getPayment(payment_id)       → 502 on failure
     5. status === 'approved'?       → 400 otherwise
     6. activateApprovedPayment({..., expectedUserId: sub })
        ├─ SELECT provider_payment_id (replay guard)
        ├─ match/lazy-create subscription by provider_subscription_id
        └─ activateSubscription() + INSERT payment (re-checked)
     7. 200 { success, plan, status: 'active' }
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `web/src/pages/api/payments/verify-mercadopago.ts` | Create | GET endpoint per D1–D5; reuses `getPayment`, `activateApprovedPayment`, `getFullTokenPayload` |
| `web/src/lib/payments/mercadopago.ts` | Modify | `activateApprovedPayment` gains optional `expectedUserId` guard (D2) |
| `web/src/pages/dashboard/index.astro` | Modify | Replace inline MP fetch (lines ~122–166) with endpoint fetch; success UI only on `{ success: true }`; surface 401/403/502; informational message for non-approved |
| `web/src/test/payments/verify-mercadopago.test.ts` | Create | Vitest suites below |
| `web/docs/vercel-rollout.md` | Modify | Env table: `MP_WEBHOOK_SECRET` required for webhook signature verification; absence → webhook 500s, MP retries; verify endpoint is dashboard-side fallback |

## Interfaces / Contracts

```ts
export interface ApprovedPaymentInput {
  paymentId: string;
  externalReference: string;
  transactionAmount: number;
  currencyId: string;
  expectedUserId?: string; // early-return unless externalReference starts with it
}
```

```text
GET /api/payments/verify-mercadopago?payment_id=P&external_reference=U-plan-x
200 { success: true, plan: 'pro', status: 'active' }
400 { success: false, message, status? } | 401 | 403 | 502 { success: false, message }
```

## Testing Strategy

`web/src/test/payments/verify-mercadopago.test.ts` (Vitest; mock `getFullTokenPayload`, `getPayment`, `activateApprovedPayment`, `db` — pattern of `webhook-flows.test.ts`):

| Suite | Key assertions |
|-------|----------------|
| Happy path | 200 `{ success: true, plan, status: 'active' }`; activation called with `expectedUserId: payload.sub` |
| Auth | no payload → 401; no activation/persistence |
| Ownership mismatch | reference not starting with `sub` → 403; no activation/persistence |
| Idempotency | repeated verify → 200; payment INSERT once (mock call count) |
| MP API failure | non-2xx `getPayment` → 502; nothing persisted |

Run from `web/`: `bun run test -- web/src/test/payments/verify-mercadopago.test.ts` and `bunx tsc --noEmit`.

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary. Auth/ownership boundaries are covered by D3/D5 and their RED tests.

## Migration / Rollout

No DB migration — no schema change; idempotency enforced at query level. No feature flag; endpoint is additive, dashboard swap self-contained. `vercel.json` unchanged. Deploy: merge → `bunx vercel --prod` from `web/`. The webhook keeps 500ing until `MP_WEBHOOK_SECRET` is set in Vercel (non-goal); verify closes the persistence gap regardless.

## Rollback

Delete `verify-mercadopago.ts` and its test; restore `mercadopago.ts` (drop `expectedUserId` — webhook never passes it) and `index.astro` inline verification; revert `vercel-rollout.md`. No migration to reverse; straggler payments duplicate-safe by design.

## Open Questions

- None blocking. Follow-up (out of scope): unique index on `subscription_payments(provider_payment_id)` as a hard DB-level replay guard.
