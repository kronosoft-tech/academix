# Exploration: MP Webhook 404 & Resilience Fix

> Scope: `web/` (Astro 7 SSR on Vercel). Read-only exploration — no code modified.
> Related prior change: `openspec/changes/archive/2026-08-13-mp-webhook-500-diagnosis` (env-var 500 at the secret gate) and `openspec/changes/archive/2026-08-12-payments-webhook-and-errors` (signature verification + dashboard flow).

## Current State

The MP webhook handler (`web/src/pages/api/webhooks/mercadopago.ts`) flow after signature verification passes (L54-69, implemented post-9340e65):

1. **Secret gate** (L39-48): `getWebhookSecret()` — if falsy, returns 500 before anything else. *Already addressed* per the archived diagnosis; `.env.example` L17 now documents `MP_WEBHOOK_SECRET`.
2. **Signature verification** (L54-69): `verifyWebhookSignature(...)` — returns 401 on invalid, never 500 (proven by code read: missing fields → `return false` → 401).
3. **`processPayment(paymentId)`** (L124-146) called at L127 (POST) / L108 (GET) inside a try/catch:
   - L127: `getPayment(paymentId)` — fetches `/v1/payments/{paymentId}` from MP API.
   - L128-134: `getPayment` catch — logs and **rethrows** (double-logs, see Risks).
   - L136: early-return if `payment.status !== 'approved'`.
   - L141: `activateApprovedPayment(...)` — no local try/catch; DB errors propagate.
4. **Webhook handler catch** (POST: L84-93, GET: L109-115): ANY error from `processPayment` → **HTTP 500 "Failed to process payment"** (L89 / L111).

The 500 collapses **every** failure mode into one response — 404, 401, 403, 429, MP 5xx, and DB errors — with no distinction.

### The critical asymmetry (confirmed in code)

`getPayment` throws a **plain `Error`** with no structured status property:

```ts
// web/src/lib/payments/mercadopago.ts L185-188
if (!response.ok) {
  const error = await response.text();
  throw new Error(`MercadoPago getPayment failed: ${response.status} - ${error}`);
}
```

The HTTP status code is embedded in the message *string* only — not as a property on the error. Contrast with `createPreference` (L154-159) which **does** attach `err.status = response.status`:

```ts
// web/src/lib/payments/mercadopago.ts L154-159
const err = new Error(...) as Error & { status?: number; detail?: string };
err.status = response.status;
```

This means the webhook handler currently **cannot programmatically distinguish** a 404 "payment not found" from a 401 "token expired" — both are plain `Error` with status text buried in the message. This is the root obstacle to the resilience fix.

## Affected Areas

- `web/src/lib/payments/mercadopago.ts` — **L185-188** (`getPayment` throw: needs `.status` enrichment, matching `createPreference` L156-158); L256-330 (`activateApprovedPayment` DB calls — errors here are critical, should stay 500); L208-237 (`verifyWebhookSignature` — already 401, not in scope).
- `web/src/pages/api/webhooks/mercadopago.ts` — **POST L81-93** (catch → 500, needs per-status classification); **GET L107-115** (IPN legacy, same 500 issue, no signature verification); L124-146 (`processPayment` — inner try/catch at L126-134 double-logs and rethrows).
- `web/src/lib/db.ts` — L10-12 (`throw new Error('TURSO_URL not set')`) — no `.status` property → naturally falls through to 500 (correct, critical).
- `web/src/pages/api/payments/verify-mercadopago.ts` — L58-66 (dashboard verify path also catches `getPayment` error → 502; has the same `.status`-missing issue but different semantics — user-facing, not MP-driven).

## Key Question Answers

### 1. Exact lines where `getPayment` is called and its throw becomes a 500

| Step | File:Line | What happens |
|------|-----------|-------------|
| Call | `mercadopago.ts:127` | `processPayment` calls `await getPayment(paymentId)` |
| Throw | `mercadopago.ts:187` | `getPayment` throws plain `Error` on `!response.ok` |
| Rethrow | `mercadopago.ts:128-134` | `processPayment` catch logs, then `throw err` (rethrow) |
| 500 (POST) | `mercadopago.ts:89` | Webhook POST catch returns `status: 500` |
| 500 (GET) | `mercadopago.ts:111` | Webhook GET catch returns `status: 500` |

### 2. Does `getPayment` throw for ALL non-2xx (404/401/429/5xx)?

**YES.** Line 185: `if (!response.ok)` — `response.ok` is `false` for every status outside 200–299. A 404 "Payment not found", a 401 expired token, a 429 rate-limit, and a 5xx MP server error ALL throw the same plain `Error`. The thrown error does **NOT** carry a `.status` property (unlike `createPreference` which does at L157). The status code exists only in the message string: `"MercadoPago getPayment failed: 404 - ..."`.

### 3. For a 404 "Payment not found", MP's correct webhook response is **200**

**Confirmed.** Mercado Pago's webhook contract: the endpoint must return HTTP 200 to acknowledge receipt and stop retries. A 404 from `getPayment` means the payment id is unknown to MP — possibly already final, already processed by another account, or a transient/migrated id. Returning 500 makes MP treat the endpoint as **broken and abandon all retries** → the payment never persists. The correct behavior is **200** with a warning field and a log entry:

```
200 { received: true, warning: "payment not found" }
```

### 4. For 401/403 from MP API on `getPayment`: currently 500, should be log + 200

**Confirmed.** A 401/403 from MP's API means the `MP_ACCESS_TOKEN` is expired, revoked, or misconfigured — a **server-side config error** that MP cannot fix by retrying the webhook. Returning 500 causes MP to abandon retries (same broken-endpoint penalty as 404). The correct behavior is:

```
200 { received: true }   +   console.error with clear "MP_API_AUTH_ERROR" tag
```

This acknowledges receipt (MP stops retrying) while surfacing the config issue in logs where operators can act on it.

### 5. For 429 rate-limiting from MP API (additional finding)

Currently 500 (same catch). A 429 means **we** are being rate-limited by MP's API — the webhook notification was received fine, but the follow-up `getPayment` call was throttled. Returning 500/200 both have tradeoffs:

- **500** → MP retries with backoff (gives rate limit time to reset; correct for transient throttling).
- **200** → MP stops retrying; we lose the payment (bad).
- **503 + Retry-After** → most semantically correct (tells MP to back off); but MP's webhook retry contract is 200-received / 5xx-retry, and 503 is a 5xx so MP will retry.

**Recommended:** 503 with `Retry-After` header for 429 (semantically correct — MP retries later).

## Approaches

### Approach A: Enrich `getPayment` error + handler-level classification (RECOMMENDED)

**Change 1** — `mercadopago.ts` getPayment (L185-188): attach `.status` to the thrown error, matching the existing `createPreference` pattern (L156-158). Optionally attach a marker like `.isMpApiError = true` or use a custom error class (Approach B variant).

**Change 2** — `webhooks/mercadopago.ts` handler catch (POST L84-93, GET L107-115): inspect `err.status`:
- `404` → `console.warn` + 200 `{ received: true, warning: "payment not found" }`
- `401` / `403` → `console.error` (tagged) + 200 `{ received: true }`
- `429` → `console.warn` + 503 + `Retry-After` header
- `5xx` (MP server error) → `console.error` + 503 (transient; MP retries)
- **No `.status`** (DB errors, TURSO_URL missing, signature threw) → 500 (critical)

**Change 3** — Remove the inner try/catch in `processPayment` (L126-134) so `getPayment`'s enriched error bubbles directly to the handler (avoids double-logging and preserves `.status`).

| Pros | Cons | Effort |
|------|------|--------|
| Minimal diff; reuses the existing `err.status` pattern from `createPreference` (L156-158) | Adds an error-classification branch in the handler | Low |
| Clear mapping: each MP status → correct MP webhook semantics (200/503) | Double-logging at L129 + L85 must be cleaned up | |
| DB/critical errors naturally fall through (no `.status`) → 500 | | |

### Approach B: Custom `MpApiError` class + handler classification

Define `class MpApiError extends Error { status: number; statusCode: number }` (precedent: `ProvisioningError` at `provisioning.ts:35-43`). `getPayment` throws `MpApiError`; handler does `err instanceof MpApiError` + `err.status`.

| Pros | Cons | Effort |
|------|------|--------|
| Type-safe; follows existing `ProvisioningError` codebase precedent | Slightly more code (new class) | Low-Med |
| Clean `instanceof` check in handler avoids fragile property checks | | |

### Approach C: `processPayment` returns a structured result (no throws)

Change `processPayment` to return `{ ok: true } | { errorType: 'payment_not_found' \| 'auth_error' \| 'rate_limit' \| 'transient' \| 'critical'; retryAfter?: number }`. Handler maps `errorType` → HTTP response.

| Pros | Cons | Effort |
|------|------|--------|
| Explicit, self-documenting result handling; no error parsing | Larger refactor; changes function contract from throw-based to return-based | Med-High |
| Deviates from the throw-based pattern used throughout `mercadopago.ts` (`createPreference`, `activateApprovedPayment`) | | |

## Recommendation

**Approach A** — enrich `getPayment`'s thrown error with `.status` (matching `createPreference` at L156-158), then classify at the webhook handler level using `err.status`. This is the smallest, most consistent change. Optionally use Approach B's `MpApiError` class variant for type safety if the team prefers.

The response mapping should be:

| MP API status | HTTP response | Log level | Rationale |
|---|---|---|---|
| 404 | 200 `{ received: true, warning: "payment not found" }` | warn | Payment unknown/transient — MP should stop retrying |
| 401 / 403 | 200 `{ received: true }` | error (tagged `MP_API_AUTH_ERROR`) | Server-side token config issue; MP can't fix by retrying |
| 429 | 503 + `Retry-After` | warn | We're rate-limited; MP should back off and retry |
| 5xx | 503 | error | MP server error; transient, MP retries |
| (no `.status` — DB/TURSO_URL/signature) | 500 | error | Truly critical — should remain 500 |

Remove the redundant inner try/catch in `processPayment` (L126-134) to avoid double-logging.

## Risks

- **MP webhook contract risk**: Returning 200 for 404 means MP stops retrying forever. If the payment id is genuinely needed but MP can't resolve it (e.g. wrong access token returns 404 rather than 401 for some MP endpoints), we could silently drop a real payment. Mitigation: log 404s at warn level with full context so they're visible in monitoring.
- **Double-logging**: `processPayment` L129-132 logs `getPayment` errors, then rethrows to the handler which logs again at L85-88. The fix must remove the inner log/rethrow (or keep only one).
- **GET handler (IPN legacy)**: Has no signature verification (L102-121) and the same 500 issue. If IPN is still used, the same classification must apply — but without signature verification, a malicious actor could send a fake IPN. This should be noted as a future hardening item.
- **`getPayment` status not on Error**: If the enrichment is forgotten, the handler can't classify and falls back to 500 (current behavior). The fix must ensure `.status` is always set.
- **Prior exploration context**: The archived `mp-webhook-500-diagnosis` concluded the 500 was from a missing `MP_WEBHOOK_SECRET` (secret gate at L39-48). That gate returns 500 *before* `getPayment` is ever called, so it's a **different** 500 path. This exploration addresses the *post-signature* 500s from `getPayment` errors. Both fixes are complementary.

## Ready for Proposal

**Yes.** Three concrete code changes are identified:
1. Enrich `getPayment` error with `.status` (and optionally `.isMpApiError` flag) in `mercadopago.ts`.
2. Classify errors by `err.status` in the webhook handler's catch blocks (POST + GET), returning 200/503/500 per the mapping table above.
3. Remove the redundant inner try/catch in `processPayment` (L126-134) to avoid double-logging.

The recommendation is solid and the changes are mechanical. The orchestrator should tell the user: "I've confirmed the root cause — `getPayment` throws a plain Error with no `.status` property for all non-2xx MP API responses, and the webhook handler returns 500 for everything. The fix enriches the error with `.status` (matching the existing `createPreference` pattern) and classifies 404→200, 401/403→200+log, 429→503, 5xx→503, and DB/critical→500. Ready for `sdd-propose`."

```
status: complete
executive_summary: |
  POST /api/webhooks/mercadopago returns 500 for ALL getPayment failures (404/401/429/5xx/DB)
  because getPayment (mercadopago.ts L185-188) throws a plain Error with no .status property,
  and the webhook handler catch (L84-93) returns 500 "Failed to process payment" for everything.
  Unlike createPreference (L156-158) which attaches err.status, getPayment does not. MP's webhook
  contract requires 200 to acknowledge receipt; 500 makes MP treat the endpoint as broken and
  abandon retries, so payments for 404/401/429 errors never persist. Recommended fix: enrich
  getPayment error with .status (matching createPreference pattern), classify at handler level
  (404→200+warning, 401/403→200+log, 429→503+Retry-After, 5xx→503, DB/critical→500), and remove
  the double-logging inner try/catch in processPayment (L126-134).
artifacts:
  - { backend: engram, topic_key: "sdd/mp-webhook-404-and-logging/explore", type: decision }
  - { backend: openspec, path: "openspec/changes/mp-webhook-404-and-logging/exploration.md", type: architecture }
next_recommended: propose
risks:
  - "404→200 mapping stops MP retries forever; if 404 masks a token issue, real payments could be silently dropped — mitigate with warn-level logging"
  - "Double-logging: processPayment L129-132 + handler L85-88 must be consolidated (remove inner try/catch)"
  - "GET handler (IPN legacy L102-121) lacks signature verification — same 500 fix needed but security gap noted separately"
  - "getPayment must reliably attach .status or classification falls back to 500 (current behavior)"
  - "Prior archived exploration (mp-webhook-500-diagnosis) addressed the secret-gate 500 at L39-48 — a DIFFERENT 500 path; both fixes are complementary"
skill_resolution:
  loaded: [sdd-explore]
  notes:
    - "sdd-explore SKILL.md: read-only investigation; persist exploration.md to openspec + engram"
    - "Codebase precedent: createPreference (mercadopago.ts L156-158) already attaches err.status — follow this pattern"
    - "Codebase precedent: ProvisioningError class (provisioning.ts L35-43) — available if team prefers typed error class over property check"
    - "MP webhook contract: 200 acknowledges receipt and stops retries; 5xx triggers MP backoff retry; 4xx typically not retried"
    - "verify-mercadopago.ts L58-66 (dashboard verify) also calls getPayment but returns 502 (different semantics — user-facing)"
```
