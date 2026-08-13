# Tasks: MercadoPago Payment Persistence (verify-mercadopago)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~425 (range 400–450) |
| 400-line budget risk | Medium |
| Chained PRs recommended | No |
| Suggested split | Single PR with work-unit commits |
| Delivery strategy | single-pr (auto-forecast preflight) |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | T1+T2 endpoint + guard | PR 1 | `bun run test -- web/src/test/payments/verify-mercadopago.test.ts` | `bun run dev`; MP sandbox redirect to `/dashboard?payment_id=P&status=approved&external_reference={sub}-pro-uuid` | Delete `verify-mercadopago.ts`; drop `expectedUserId` from `activateApprovedPayment` (webhook never passes it) |
| 2 | T3 dashboard wiring | PR 1 | `bunx tsc --noEmit` | `bun run dev`; load `/dashboard?payment_id=P&status=approved` with endpoint mocked to 401/403/502 | Restore inline MP block in `index.astro` (lines 122–166) |
| 3 | T4+T5 tests + docs | PR 1 | `bun run test -- web/src/test/payments/verify-mercadopago.test.ts` | N/A — Vitest mocks external HTTP; unit suites cover runtime behavior | Delete test file; revert `vercel-rollout.md` env rows |

## Phase 1: Foundation (Guard + Endpoint)

### T1 — Create `web/src/pages/api/payments/verify-mercadopago.ts` (GET)

- **Description**: `export const prerender = false;` + `GET: APIRoute` reading `payment_id` / `external_reference` from `url.searchParams` (accept `collection_id` alias for `payment_id`). Reuse `getFullTokenPayload` (`web/src/lib/auth.ts`) and `getPayment` + `activateApprovedPayment` (`web/src/lib/payments/mercadopago.ts`). Mirror `verify-wompi.ts` structure with the D5 contract.
- **Status**: DONE
- **Acceptance criteria**: 401 when no JWT payload; 400 when params missing; 403 when `externalReference` does not start with `payload.sub`; 502 + `console.error` when `getPayment` throws (non-2xx/network); 400 `{ success: false, message, status }` when MP `status !== 'approved'`; 200 `{ success: true, plan, status: 'active' }` after `activateApprovedPayment({ paymentId, externalReference, transactionAmount, currencyId, expectedUserId: payload.sub })`. `plan` = `externalReference.split('-')[5]`.
- **Dependencies**: none (builds on existing libs)
- **Est. changed lines**: +120
- **Files**: `web/src/pages/api/payments/verify-mercadopago.ts`

### T2 — Extend `activateApprovedPayment` with optional `expectedUserId` guard

- **Description**: In `web/src/lib/payments/mercadopago.ts`, add `expectedUserId?: string` to `ApprovedPaymentInput` and the destructure; at the top of the function body (before the replay SELECT) early-return unless `externalReference.startsWith(expectedUserId)`. Webhook callers never pass the guard, so their behavior is unchanged.
- **Acceptance criteria**: With `expectedUserId` set and reference not prefixed → returns with no SELECT, no activation, no INSERT; without the field → identical behavior to today; existing webhook tests pass unchanged.
- **Status**: DONE
- **Dependencies**: none
- **Est. changed lines**: ~10 (add+del)
- **Files**: `web/src/lib/payments/mercadopago.ts`

## Phase 2: Integration (Dashboard Wiring)

### T3 — Replace inline MP verification in `web/src/pages/dashboard/index.astro`

- **Description**: Replace lines 122–166 (inline MP fetch + direct `activateApprovedPayment` call) with `fetch('/api/payments/verify-mercadopago?payment_id=…&external_reference=…')` (same-origin; cookies auto-sent; keep `collection_id` fallback). Set `paymentSuccess = true` ONLY on `response.ok && body.success === true`; surface 401/403/502 failure messages; informational message for non-approved status.
- **Acceptance criteria**: No `MP_API_URL` / `MP_ACCESS_TOKEN` / `activateApprovedPayment` references remain in dashboard; success UI only on `{ success: true }`; failures surfaced; non-approved → informational text.
- **Status**: DONE
- **Dependencies**: T1
- **Est. changed lines**: ~70 (add+del)
- **Files**: `web/src/pages/dashboard/index.astro`

## Phase 3: Tests + Docs

### T4 — Create `web/src/test/payments/verify-mercadopago.test.ts`

- **Description**: 5 Vitest suites using the `webhook-flows.test.ts` mock pattern (`vi.mock` `../../lib/auth`, `../../lib/payments/mercadopago`, `../../lib/db` with `mockExecute`): Happy path (200 body; activation called with `expectedUserId: payload.sub`), Auth (no payload → 401; no activation/persistence), Ownership mismatch (403; none), Idempotency (repeated verify → 200; payment INSERT executed exactly once), MP API failure (non-2xx `getPayment` → 502; nothing persisted).
- **Acceptance criteria**: All 5 suites pass; idempotency suite asserts INSERT call count === 1.
- **Status**: DONE
- **Dependencies**: T1, T2
- **Est. changed lines**: +210
- **Files**: `web/src/test/payments/verify-mercadopago.test.ts`

### T5 — Document MP env vars in `web/docs/vercel-rollout.md`

- **Description**: Add `MP_WEBHOOK_SECRET` (required for webhook `x-signature` verification; absence → webhook 500s, MP retries; verify endpoint is the dashboard-side persistence fallback) and `MP_ACCESS_TOKEN` (dependency of the verify endpoint) to the env table with location + failure mode.
- **Acceptance criteria**: Both vars listed; docs scenario "Docs locate the secret" satisfied.
- **Status**: DONE
- **Dependencies**: none
- **Est. changed lines**: +15
- **Files**: `web/docs/vercel-rollout.md`

## Phase 4: Verification

### T6 — Verify web suite

- **Description**: From `web/`: `bun run test -- web/src/test/payments/verify-mercadopago.test.ts`, `bunx tsc --noEmit`, then full `bun run test`.
- **Acceptance criteria**: New suites green; tsc clean; full suite green or only pre-existing unrelated failures documented.
- **Status**: DONE
- **Dependencies**: T1–T5
- **Est. changed lines**: 0
- **Files**: none

## Commit Plan (work units, conventional commits)

1. `feat(web): guard MercadoPago activation with optional expectedUserId` → T2
2. `feat(web): add owner-checked verify-mercadopago redirect endpoint` → T1
3. `test(web): cover verify-mercadopago auth, ownership, idempotency, MP failures` → T4
4. `feat(web): verify MercadoPago redirects via endpoint in dashboard` → T3
5. `docs(web): document MP_WEBHOOK_SECRET and MP_ACCESS_TOKEN env vars` → T5

T6 is verification only (no commit). Each commit keeps its tests/docs with the behavior they verify; each reverts independently.
