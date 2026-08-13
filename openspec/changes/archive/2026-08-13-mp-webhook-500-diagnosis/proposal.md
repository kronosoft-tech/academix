# Proposal: MP Webhook 500 Diagnosis & Fix

## Intent

`POST /api/webhooks/mercadopago` returns 500 for all payloads. Root cause: `MP_WEBHOOK_SECRET` is unset in Vercel — `webhooks/mercadopago.ts:40` returns 500 "Mercado Pago webhook secret not configured" as the **first** guard, before signature verification, before `getPayment`, before any DB call. `.env.example` (lines 15–16) documents `MP_ACCESS_TOKEN`/`MP_API_URL` but omits `MP_WEBHOOK_SECRET`. After the env fix, a manual test payload (no `x-signature`) correctly returns 401. Code hardening prevents edge-case 500s post-fix.

**User action:** Add `MP_WEBHOOK_SECRET` to Vercel production from MP Dashboard "Webhooks" secret key (NOT the access token), then redeploy.

## Scope

### In Scope
- Add `MP_WEBHOOK_SECRET` to `web/src/.env.example`
- Add env-add command to `web/docs/vercel-rollout.md`
- Harden `verifyWebhookSignature` (`mercadopago.ts:208`) — explicit `try/catch`, no throw
- Guard call site `webhooks/mercadopago.ts:54` → 401 on throw
- Surface stale `checkout-integration.test.ts` (references nonexistent `createPreapproval`) as discovered issue

### Out of Scope
- `getPayment`/DB throw handling (downstream of signature gate — only reachable with valid `x-signature`)
- MP signature algorithm change (unchanged)

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `mercadopago-payments`: `verifyWebhookSignature` must be explicitly no-throw (defensive `try/catch` returning `false`); the handler must return 401 (not 500) for absent/malformed `x-signature`. (Spec already mandates missing-secret→500, invalid-signature→401 — this hardens the implementation to match.)

## Approach

**Phase 1 (user):** Add `MP_WEBHOOK_SECRET` via `bunx vercel env add MP_WEBHOOK_SECRET production`; redeploy; confirm via Vercel logs querying "Mercado Pago webhook secret not configured".

**Phase 2 (code):** Patch `.env.example`; update rollout doc; wrap `verifyWebhookSignature` in `try/catch` returning `false`; wrap call site in `try/catch` returning 401.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `web/src/.env.example` | Modified | Add `MP_WEBHOOK_SECRET` line after `MP_API_URL` |
| `web/docs/vercel-rollout.md` | Modified | Add env-add command to checklist |
| `web/src/lib/payments/mercadopago.ts` | Modified | `try/catch` in `verifyWebhookSignature` (L208–232) |
| `web/src/pages/api/webhooks/mercadopago.ts` | Modified | Guard signature call (L54) → 401 on throw |
| `openspec/specs/mercadopago-payments/spec.md` | Reference | Behavior already spec'd; code hardened to match |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Manual test → 401 post-fix mistaken as "still broken" | High | Document expected post-fix behavior |
| Wrong credential (access token as secret) → 401 | Med | Docs: "distinct from access token" |
| Stale test references `createPreapproval` | Low | Surface as discovered issue, defer |

## Rollback Plan

Git revert the 2 code + 2 doc file patches. Env var removal is via Vercel dashboard only (handler already returns clear 500 on absence — no code rollback needed).

## Dependencies

- User must add `MP_WEBHOOK_SECRET` to Vercel before code changes are meaningful
- MP Dashboard webhook HMAC secret (not `MP_ACCESS_TOKEN`)

## Success Criteria

- [ ] `MP_WEBHOOK_SECRET` documented in `.env.example`
- [ ] `vercel-rollout.md` lists `vercel env add MP_WEBHOOK_SECRET production`
- [ ] `verifyWebhookSignature` wrapped in `try/catch` — no throw can cause 500
- [ ] Malformed/missing `x-signature` → 401, never 500
- [ ] Manual test payload → 401 post-fix; real MP webhook → 200 + payment processed

## Proposal Question Round

> This is a bug fix with a clear root cause. These assumptions need user review:

1. **Post-fix behavior:** After adding the env var, a manual payload *without* `x-signature` should return 401 (not 200). Confirm this is desired, not a "still broken" signal.
2. **Spec alignment:** The `mercadopago-payments` spec already mandates missing-secret→500, invalid-signature→401. We propose no spec change — only code hardening. Agree?
3. **Test debt:** `checkout-integration.test.ts` references `createPreapproval` (nonexistent in CO). Defer to a follow-up?
