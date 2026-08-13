## Exploration: MP Webhook 500 Diagnosis

### Current State
`POST https://academix-three-gilt.vercel.app/api/webhooks/mercadopago` returns HTTP 500 for all payloads — including a manual MP sandbox test payload (`{"action":"payment.updated","data":{"id":"123456"}}`).

The handler (`web/src/pages/api/webhooks/mercadopago.ts:28-90`, POST) runs this exact order:

1. Parse JSON body (lines 30-37) — `try/catch` → 400 on parse error.
2. **Read webhook secret** — `getWebhookSecret()` (line 39). If falsy, **returns 500 "Mercado Pago webhook secret not configured"** (lines 40-48). This is a HARD STOP — nothing after it runs.
3. Verify `x-signature` (line 54) — `verifyWebhookSignature(...)` returns `false` → 401, or throws → uncaught → 500.
4. Extract `paymentId` (lines 61-69).
5. `processPayment(paymentId)` in a `try/catch` (lines 72-83) — throws → 500 "Failed to process payment".

`getWebhookSecret()` returns `import.meta.env.MP_WEBHOOK_SECRET` (read at `mercadopago.ts:16` as `import.meta.env.MP_WEBHOOK_SECRET || ''`). The env var is the ONLY input to the first 500 gate and is consumed *before* signature verification, *before* any MP API call, and *before* any DB call.

### Affected Areas
- `web/src/pages/api/webhooks/mercadopago.ts` — the 500-gate at lines 39-48 (secret check) and the uncaught `processPayment`/`getPayment` path at lines 54/73.
- `web/src/lib/payments/mercadopago.ts` — env reads at lines 13-16; `getWebhookSecret` (line 194); `verifyWebhookSignature` (lines 208-232); `getPayment` (lines 169-191); `activateApprovedPayment` (lines 251-324).
- `web/src/lib/db.ts` — `TURSO_URL`/`TURSO_AUTH_TOKEN` read at lines 7-8; throws if `TURSO_URL` unset.
- `web/src/.env.example` — documents `MP_ACCESS_TOKEN` + `MP_API_URL` (lines 15-16) but **omits `MP_WEBHOOK_SECRET`**.
- `web/docs/vercel-rollout.md:19` — explicitly documents: "Absent → the webhook responds 500."
- `web/migrations/001_subscriptions.sql` / `002_multi_gateway.sql` — `subscription_payments` schema consumed by `activateApprovedPayment` INSERT (line 312).

### Env Var Names Expected by the Handler (verified via grep + code)
| Var | Read at | Consumed by |
|-----|---------|-------------|
| `MP_WEBHOOK_SECRET` | `mercadopago.ts:16` | `getWebhookSecret()` (`mercadopago.ts:194`) → checked `webhooks/mercadopago.ts:40` |
| `MP_ACCESS_TOKEN` | `mercadopago.ts:15` | `getPayment` (line 181), `createPreference` (line 129), checkout `mercadopago.ts` (line 38) |
| `MP_API_URL` | `mercadopago.ts:14` | `getPayment` (line 178), `createPreference` (line 125) |
| `TURSO_URL` | `db.ts:7` | `getDb()` — throws if unset |
| `TURSO_AUTH_TOKEN` | `db.ts:8` | `getDb()` |

### Approach Comparison

| # | Approach (Root Cause) | Evidence | Pros | Cons | Effort |
|---|------------------------|----------|------|------|--------|
| A | **`MP_WEBHOOK_SECRET` not set in Vercel** — secret resolves to `''`, so `mercadopago.ts:16` (`import.meta.env.MP_WEBHOOK_SECRET \|\| ''`) → `getWebhookSecret()` returns `''` → `webhooks/mercadopago.ts:40` `if (!secret)` is true → **500 "Mercado Pago webhook secret not configured"** (line 42), returned BEFORE the signature check (line 54), BEFORE `getPayment`, BEFORE any DB call. The `.env.example` (lines 15-16) documents only `MP_ACCESS_TOKEN` + `MP_API_URL`, omitting `MP_WEBHOOK_SECRET` entirely, so the operator likely never provisioned it. | `web/src/.env.example:15-16` (no `MP_WEBHOOK_SECRET`); `web/docs/vercel-rollout.md:19` ("Absent → the webhook responds 500"); `mercadopago.ts:16,194`; `webhooks/mercadopago.ts:39-48`. | Single-var fix; no code change; documented fallback path unaffected. | Does not explain a genuine downstream failure if the secret *was* set. | Low (add 1 Vercel env var + redeploy) |
| B | **`verifyWebhookSignature` throws on the sandbox test payload** — malformed/absent `x-signature` causes an exception that propagates uncaught (no `try/catch` wraps line 54) → 500. | `mercadopago.ts:208-232`; `webhooks/mercadopago.ts:54` (unguarded). | Matches "test payload" framing. | Code analysis shows it does NOT throw: empty/missing `x-signature` → `parts` lacks `ts`/`v1` → `return false` (→ 401, not 500). Non-hex `v1` → `Buffer.from(v1,'hex')` silently truncates in Node (no throw) → length mismatch → `return false` (→ 401). So signature issues produce **401**, never 500. | Low as a 500 cause — disproven by code read. | Low |
| C | **`getPayment("123456")` with a fake/test id throws** — `getPayment` (`mercadopago.ts:169-191`) calls `fetch(/v1/payments/123456)`; a non-existent id → MP returns 404 → `getPayment` throws → caught at `webhooks/mercadopago.ts:74` → 500 "Failed to process payment". | `mercadopago.ts:178-188`; `webhooks/mercadopago.ts:72-83`. | Would only fire AFTER signature passes (line 54). A manual curl/Postman test payload has no valid `x-signature` → fails at line 54 → 401, never reaching `getPayment`. Only a *real* MP sandbox notification (valid x-signature) for a non-existent payment would hit this — a secondary concern, not the immediate cause. | Medium | Low |
| D | **`activateApprovedPayment` / DB throws** — `db.ts:7-8` throws "TURSO_URL environment variable is not set" if control-plane Turso vars are missing; or a schema mismatch on `subscription_payments` INSERT (`mercadopago.ts:312`). | `db.ts:7-12`; `mercadopago.ts:299-324`; `migrations/001+002`. | Requires passing signature verification first (line 54). Schema actually checks out: migration 001 creates `subscription_payments` with `id,user_id,subscription_id,amount,currency,method,status,provider,paid_at,created_at`; migration 002 adds `provider_payment_id`. The INSERT at line 312 uses exactly `id,user_id,subscription_id,amount,currency,status,provider,provider_payment_id,created_at` — all present. So schema mismatch is NOT the issue; only missing `TURSO_URL`/`TURSO_AUTH_TOKEN` remains, and that is downstream of the signature gate. | Low (only reachable past signature check); schema verified compatible. | Low |

### Recommendation
**Root cause = Candidate A: `MP_WEBHOOK_SECRET` is not set in the Vercel production environment.**

The chain is airtight and confirmed by two independent sources:
1. **Code** (`mercadopago.ts:16` → `getWebhookSecret()` → `webhooks/mercadopago.ts:40-48`): an empty secret returns HTTP 500 at the *very first* guard, before signature verification, before `getPayment`, before any DB call. The 500 body is `{"error":"Mercado Pago webhook secret not configured"}`.
2. **Docs** (`web/docs/vercel-rollout.md:19`): states verbatim — "Absent → the webhook responds 500."
3. **Config gap** (`web/src/.env.example:15-16`): the `.env.example` lists `MP_ACCESS_TOKEN` and `MP_API_URL` but never `MP_WEBHOOK_SECRET`, so an operator following the file (or the checklist in the rollout doc, which only enumerates `MP_ACCESS_TOKEN`/`MP_WEBHOOK_SECRET` in prose but whose `.env.example` omits the secret) plausibly never added it to Vercel.

This also explains why the **manual sandbox test payload** returns 500 regardless of its `x-signature`: the secret gate fires first and short-circuits. (Once A is fixed, that same manual payload with no valid `x-signature` would correctly return **401**, matching real MP behavior — MP's live webhooks send `x-signature`, manual tests do not.)

#### Single most likely quick-fix
Add `MP_WEBHOOK_SECRET` to the Vercel production environment — copied from the **Mercado Pago Dashboard "Webhooks" section's "Secret key"** (distinct from the Access Token, which is `MP_ACCESS_TOKEN`). Per `vercel-rollout.md:22`, use:
```bash
bunx vercel env add MP_WEBHOOK_SECRET production
```
Then redeploy. The MP dashboard webhook secret should match the HMAC the handler expects: `x-signature` format `ts=...,v1=...`, HMAC-SHA256 over `id:<data.id>;request-id:<x-request-id>;ts:<ts>;` (see `mercadopago.ts:208-232`).

#### What to inspect next (Vercel function logs)
Query the Vercel Functions/Logs tab for the exact 500 body strings, scoped to path `/api/webhooks/mercadopago`:

- **`"Mercado Pago webhook secret not configured"`** → confirms A. (Expected if A is the cause.)
- **`"Failed to process payment"`** → would indicate C (post-signature `processPayment`/`getPayment` failure). Absent if A is the cause.
- **`"TURSO_URL environment variable is not set"`** → would indicate D. Absent if A is the cause.
- **`"Invalid signature"`** → would indicate B (but as 401, not 500). Absent for the reported 500.

If the first string is present, A is confirmed and no further code diagnosis is needed.

### Risks
- **Misdiagnosing B as the cause:** a manual test payload (no `x-signature`) is *expected* to return 401 after the secret is fixed. If MP's real webhook continues 500-post-fix, the real culprit shifts to C or D and the log queries above will surface it.
- **Confirming the wrong credential:** `MP_WEBHOOK_SECRET` (HMAC signing secret from the MP "Webhooks" settings) is NOT the same as `MP_ACCESS_TOKEN` (Bearer token). Swapping them silently fails signature verification → 401, not 500.
- **Stale `.env.example`:** the example is already missing the secret, so the fix should also patch `web/src/.env.example` to add `MP_WEBHOOK_SECRET=<webhook-secret>` for future operators (deferred to the proposal phase; NOT modified in this exploration).
- **Astro env static analysis:** env vars are read via `import.meta.env` at module load (`mercadopago.ts:13-16`). The Astro config (`astro.config.mjs`) has no custom `publicPrefix`/`env` integration gating server-side vars, so once `MP_WEBHOOK_SECRET` is set in Vercel it will be visible to the API route. No code change needed, only a redeploy.

### Ready for Proposal
Yes. The root cause is a missing env var (`MP_WEBHOOK_SECRET`), confirmed by code + docs. The orchestrator should tell the user: the 500 is almost certainly the undocumented/unset `MP_WEBHOOK_SECRET`; add it to Vercel production (from the MP Dashboard webhooks secret, not the access token) and redeploy, then re-run the manual test — it should become 401 (correct, since a manual payload lacks a valid `x-signature`). Only if it stays 500 afterward should we investigate C/D via the log strings above. If the user wants, the next step is `sdd-propose` to (a) patch `.env.example` to document `MP_WEBHOOK_SECRET` and (b) harden `verifyWebhookSignature` against edge cases and add the missing env var to the rollout checklist.

```
status: complete
executive_summary: |
  The MP webhook 500 is caused by MP_WEBHOOK_SECRET being unset — the .env.example
  (lines 15-16) omits it and vercel-rollout.md:19 documents its absence → 500. The handler
  (webhooks/mercadopago.ts:39-48) returns HTTP 500 "Mercado Pago webhook secret not
  configured" as the FIRST guard, before signature verification, before getPayment, and
  before any DB call. This is consistent with a manual sandbox test payload 500-ing
  regardless of x-signature. Signature verification (mercadopago.ts:208-232) provably
  does NOT throw (returns false → 401), so B cannot cause 500. Candidates C/D are
  downstream of the signature gate and thus unreachable when the secret is missing.
artifacts:
  - { backend: engram, topic_key: "sdd/mp-webhook-500-diagnosis/explore", type: decision }
  - { backend: openspec, path: "openspec/changes/mp-webhook-500-diagnosis/exploration.md", type: architecture }
next_recommended: propose
risks:
  - "After fixing A, a manual test payload with no x-signature returns 401 (correct) — must not be mistaken for a still-broken webhook."
  - "MP_WEBHOOK_SECRET (HMAC signing secret) ≠ MP_ACCESS_TOKEN (Bearer) — wrong credential silently → 401."
  - ".env.example is stale; should be patched in the proposal phase."
skill_resolution:
  loaded: [sdd-explore, mercadopago, wompi]
  notes:
    - "sdd-explore/PLAN.md provides the executor gate and return envelope."
    - "mercadopago skill (skills/mercadopago/SKILL.md): Checkout Pro single-payment model, webhooks send x-signature ts=,v1=, verify via GET /v1/payments/{id}."
    - "wompi skill (skills/wompi/SKILL.md): Colombia-only gateway; not the subject here — MP is the active gateway (gateway.ts: CO→wompi, AR→mercadopago). Loaded only for webhook-signature verification patterns which MP mirrors."
  env_var_names_verified:
    - MP_WEBHOOK_SECRET
    - MP_ACCESS_TOKEN
    - MP_API_URL
    - TURSO_URL
    - TURSO_AUTH_TOKEN
```
