# Proposal: MercadoPago Payment Persistence (verify-mercadopago)

## Intent
Guarantee MercadoPago payment persistence in the control-plane `subscription_payments` table. Today only Wompi persists reliably because the MP webhook 500s in production (missing `MP_WEBHOOK_SECRET` — user action, out of scope) and there is no `verify-mercadopago` endpoint (unlike `verify-wompi.ts`).

## Approach
1. New owner-checked `GET /api/payments/verify-mercadopago` endpoint mirroring `verify-wompi.ts`. MP Checkout Pro appends `?payment_id=...&status=approved&external_reference=...` to the `back_url`; the dashboard calls this endpoint after redirect to persist the payment even when the webhook failed.
2. Extend `activateApprovedPayment` with an optional `expectedUserId` guard: webhook path passes none (signature verification is its trust boundary); verify endpoint passes the JWT `sub` as defense-in-depth.
3. Dashboard wiring: replace inline MP fetch with a same-origin fetch to the verify endpoint; show success UI only on `{ success: true }`; surface 401/403/502/400 failures.
4. Idempotent: SELECT by `provider_payment_id` before INSERT (already present in `activateApprovedPayment`).
5. Tests + env documentation.

## Scope
- IN: new verify endpoint, `expectedUserId` guard, dashboard fetch wiring, 12-test suite, env docs.
- OUT: configuring `MP_WEBHOOK_SECRET` (user action), MP renewal/reconciliation cron, Stripe, subscription plan changes.

## Key Decisions
1. Strict ownership: only the logged-in user can verify their own payment (`external_reference` must start with JWT `sub`). Per security review R1-03/R1-04.
2. GET endpoint with query params matching the verify-wompi pattern (MP redirect appends query params to `back_url`).
3. `expectedUserId` is optional: webhook never passes it; verify endpoint passes `payload.sub`.
4. Response contract: `200 { success: true, plan, status: 'active' }` / `401` / `400 { message, status? }` / `403` / `502`.

## Capabilities Contract
- New capability: `payment-verification-mercadopago`
- Modified capability: `payment-processing-mercadopago` (delta)

## Success Criteria
- MP payments confirmed via dashboard redirect are persisted in `subscription_payments` with `provider='mercadopago'`.
- All 12 tests pass.
- `tsc --noEmit` introduces no new errors.

## Next Step
sdd-spec → sdd-design → sdd-tasks → sdd-apply → sdd-verify → sdd-archive.
