# Design: MP Webhook 500 Fix — Signature Hardening

## Technical Approach

Root cause (per proposal + exploration): `MP_WEBHOOK_SECRET` is unset in Vercel, so `webhooks/mercadopago.ts:40` returns HTTP 500 "Mercado Pago webhook secret not configured" as the **first** guard — before signature verification, before `getPayment`, before any DB call. This is an **intentional deployment-time signal**, not a code defect.

Phase 1 (operator): add `MP_WEBHOOK_SECRET` via Vercel, redeploy — already documented in proposal.
Phase 2 (code): harden `verifyWebhookSignature` and its call site so that **post-fix**, signature failures (missing/malformed `x-signature`) return **401**, never 500. The missing-secret→500 gate stays. Spec R1-R5 mandates this; the code is hardened to match.

## Architecture Decisions

| # | Decision | Alternatives | Rationale |
|---|----------|-------------|-----------|
| D1 | Preserve missing-secret→500 gate (handler L39-48, unchanged) | Return 401 on missing secret; proceed with empty secret | 500 is a deployment-time signal — tells the operator the env var is missing. 401 would mask misconfiguration; proceeding with empty secret silently breaks signature verification. |
| D2 | Wrap `verifyWebhookSignature` body in try/catch → `false` | Rely on current code not throwing; throw + catch at call site only | Defense in depth — spec R5 mandates `verifyWebhookSignature` NEVER throws. Guarantees any future edge case (e.g. `split`/`Buffer` runtime quirk) returns `false` (→401), never propagates. |
| D3 | Guard call site (handler L54) with try/catch → 401 | Trust D2 alone | Belt-and-suspenders — if a future refactor reintroduces a throw, the handler still returns 401, not 500. |
| D4 | Minimal-diff: add guards around existing logic, don't rewrite | Full rewrite of verification | Smallest change surface; preserves existing tested behavior of `getWebhookSecret`, `getPayment`, `activateApprovedPayment`. |

### Decision: Harden `verifyWebhookSignature` (D2)

**Current code** (`mercadopago.ts:208-232`): already returns `false` for missing `ts`/`v1` and non-hex `v1` (Node `Buffer.from(...,'hex')` truncates silently). But it is NOT wrapped in try/catch — a runtime exception (e.g., `xSignature.split` on a non-string, or `createHmac` failure) would propagate as 500.

**Change**: wrap the entire function body (lines 214-231) in `try { ... } catch (err) { console.error('[MP WEBHOOK] verifyWebhookSignature error:', err); return false; }`. No signature change — still returns `boolean`. No logic change to the verification algorithm itself.

### Decision: Guard call site (D3)

**Current code** (`webhooks/mercadopago.ts:54`):
```ts
if (!verifyWebhookSignature(xSignature, xRequestId, dataId, secret)) {
```
**Change**: wrap in try/catch:
```ts
let signatureValid: boolean;
try {
  signatureValid = verifyWebhookSignature(xSignature, xRequestId, dataId, secret);
} catch (err) {
  console.error('[MP WEBHOOK] signature verification threw:', err);
  return new Response(JSON.stringify({ error: 'Invalid signature' }), {
    status: 401,
    headers: { 'Content-Type': 'application/json' },
  });
}
if (!signatureValid) { /* existing 401 block */ }
```

## Data Flow

```
POST /api/webhooks/mercadopago
  │
  ├── parse JSON body ──→ 400 on parse error
  ├── getWebhookSecret() ──→ 500 "secret not configured" (intentional gate, D1)
  ├── verifyWebhookSignature(sign, reqId, dataId, secret)
  │     │
  │     ├── try { ... existing HMAC logic ... }
  │     └── catch → console.error + return false  (D2: guaranteed no-throw)
  │
  ├── if !valid ──→ 401 "Invalid signature"  (D3: also try/catch → 401)
  ├── extract paymentId from body.data / body.id / topic
  ├── processPayment(paymentId) ── try/catch → 500 "Failed to process payment" (preserved)
  └── 200 { received: true }
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `web/src/lib/payments/mercadopago.ts` | Modify | Wrap `verifyWebhookSignature` body (L214-231) in try/catch → `false` |
| `web/src/pages/api/webhooks/mercadopago.ts` | Modify | Guard call site (L54) with try/catch → 401 |
| `web/src/.env.example` | Modify | Add `MP_WEBHOOK_SECRET=` after line 16 (`MP_API_URL`) |
| `web/docs/vercel-rollout.md` | Modify | Add `bunx vercel env add MP_WEBHOOK_SECRET production` to checklist + post-fix behavior note |
| `web/src/test/payments/mercadopago.test.ts` | Create | Signature + handler behavior tests (see below) |

## Interfaces / Contracts

**Unchanged.** `verifyWebhookSignature(xSignature: string, xRequestId: string, dataId: string, secret: string): boolean` — same signature, same return type. The try/catch wraps internals only. Handler `POST: APIRoute` — same Astro route, same response shape. No new types or exported APIs.

## Testing Strategy

New file `web/src/test/payments/mercadopago.test.ts` (vitest, node env per `vitest.config.ts`):

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Valid signature | Real secret, correct HMAC | `verifyWebhookSignature` | returns `true` |
| No `x-signature` | `MP_WEBHOOK_SECRET` set | POST with empty `x-signature: ''` | `verifyWebhookSignature` returns `false` → handler 401 (not 500) |
| Malformed non-hex `v1` | `MP_WEBHOOK_SECRET` set | `x-signature: ts=abc,v1=zzzz` | returns `false` → handler 401 (not 500) |
| Missing secret (deployment gate) | `MP_WEBHOOK_SECRET` unset/empty | any POST | handler returns 500 "not configured" (preserved) |

**Run**: from `web/` → `bun run test -- web/src/test/payments/mercadopago.test.ts` then `bunx tsc --noEmit`.

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary. This is HMAC signature verification hardening within an existing API route.

## Migration / Rollout

No DB migration. No data changes.

**User action**: Add `MP_WEBHOOK_SECRET` to Vercel production via `bunx vercel env add MP_WEBHOOK_SECRET production` (copied from MP Dashboard → Webhooks → Secret key, NOT the access token), then redeploy.

**Post-fix manual test**: POST to `/api/webhooks/mercadopago` without `x-signature` → expect **401** (not 500). A real MP notification with valid `x-signature` + approved payment → 200 + payment activated.

## Rollback

Revert 2 code patches + 2 doc patches + delete test file. Code rollback restores original behavior: `verifyWebhookSignature` has no try/catch (could throw → 500), call site is unguarded. Env var removal is via Vercel dashboard only — the handler's missing-secret gate already returns a clear 500.

## Open Questions

- [ ] None blocking. The stale `checkout-integration.test.ts` references nonexistent `createPreapproval` (deferred per proposal Q3) — noted as discovered technical debt, not addressed here.
