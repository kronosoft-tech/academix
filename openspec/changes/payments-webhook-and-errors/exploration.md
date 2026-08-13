# Exploration: payments-webhook-and-errors

> Scope: `web/` (Astro 7 SSR on Vercel). Read-only exploration — no code was modified.
> Related prior change: `openspec/changes/payments-and-subscriptions/` (specs: wompi-payments, mercadopago-payments, subscription-lifecycle).

## Current State

Payment flow (two gateways, geo-routed): `web/src/components/CheckoutPlans.tsx` → `POST /api/checkout/{gateway}` → gateway API → (Wompi: widget popup / MP: redirect) → DB write happens either on dashboard return (`web/src/pages/dashboard/index.astro`) or via webhook (`web/src/pages/api/webhooks/{gateway}.ts`). `vercel.json` schedules 3 cron jobs under `/api/cron/*` authenticated with `Authorization: Bearer ${CRON_SECRET}` (Vercel cron convention — matches handlers).

## Issue A — Wompi: widget works, nothing saved (root causes, ranked)

### A1 (PRIMARY) Dashboard verification calls the Wompi API WITHOUT the required auth header → 401 → silent no-op
- Widget success path: `web/src/components/CheckoutPlans.tsx:99-107` → redirects to `/dashboard?id={transaction.id}`.
- `web/src/pages/dashboard/index.astro:26-28` fetches `${WOMPI_API_URL}/transactions/${id}` with **no `Authorization` header**.
- Wompi docs (docs.wompi.co/en/docs/colombia/transacciones): `GET /v1/transactions/{transaction_id}` requires `Authorization: Bearer <public_key>` ("You can verify the status of a transaction at any time using your public key"). Without it → 401 → `txRes.ok` false → block skipped; the `catch` at `dashboard/index.astro:101-103` swallows everything ("Silently fail — webhook will handle it later").
- Same bug in `web/src/pages/api/payments/verify-wompi.ts:43-45` (also no auth header). That endpoint is **never called by any frontend code** (grep: only self-references) — dead path.
- Net effect: the intended primary DB-save path (on-return dashboard verification) is broken, and the fallback (webhook) is also broken (A2) and may not even be configured (A3).

### A2 (PRIMARY) Webhook signature computation is wrong → every real webhook returns 401
- `web/src/lib/payments/wompi.ts:104-123` hardcodes `SHA256(transaction.id + transaction.status + transaction.reference + timestamp + WOMPI_EVENTS_SECRET)`.
- Official spec (docs.wompi.co/en/docs/colombia/eventos): checksum = SHA256 of the concatenated values of the fields listed in `signature.properties` **in the order they appear**, then `timestamp`, then the events secret. For `transaction.updated`, the documented properties are `["transaction.id", "transaction.status", "transaction.amount_in_cents"]` → `id + status + amount_in_cents + timestamp + secret`.
- The code therefore: (a) includes `reference` (not part of properties), (b) omits `amount_in_cents` (part of properties), (c) ignores the `event.signature.properties` array entirely.
- Result: `web/src/pages/api/webhooks/wompi.ts:32-38` returns 401 "Invalid signature" for every legitimate event. Even with a correctly configured webhook URL, nothing is saved.

### A3 (HIGH) Webhook URL is a manual dashboard setting — nothing in code registers it
- No code sets the Wompi Events URL; it must be configured in the Wompi dashboard (comercios.wompi.co → Configuración → Eventos/Webhooks).
- Local dev: needs a public HTTPS tunnel (ngrok/cloudflared) → `https://<tunnel>.ngrok.app/api/webhooks/wompi`. Wompi **cannot** reach `localhost`.
- Production: `https://<vercel-project-domain>/api/webhooks/wompi`.
- Endpoint must answer HTTP 200 fast; Wompi retries at 30min / 3h / 24h (`web/src/pages/api/webhooks/wompi.ts:108-111` returns 200).
- `web/.env.example` **does not exist** (AGENTS.md:66 claims env keys are documented there — stale). Only `web/.env` (gitignored) exists. Keys involved: `WOMPI_API_URL` (sandbox vs production base URL), `WOMPI_PUBLIC_KEY`, `WOMPI_PRIVATE_KEY`, `WOMPI_INTEGRITY_SECRET`, `WOMPI_EVENTS_SECRET`, `SITE_URL` (drives checkout `redirectUrl`, default `http://localhost:4321` at `checkout/wompi.ts:66-67`). Local `.env` is consistently sandbox (`pub_test_…` + `sandbox.wompi.co/v1`).

### A4 (MEDIUM) Desktop-registered users have NO trial subscription row → webhooks/dashboard silently no-op
- Only the web register action creates a subscription: `web/src/actions/register.ts:48` → `createTrialSubscription` (`web/src/lib/payments/lifecycle.ts:22-37`).
- Desktop register saves the user to the control plane `users` table but never inserts a `subscriptions` row: `src-tauri/src/application/use_cases/register.rs:207-227`.
- Every activation path requires an existing `status='trial'` row: dashboard `dashboard/index.astro:39-42`, Wompi webhook `findTrialSubscriptionByUserId` (`webhooks/wompi.ts:139-148`), MP webhook (`webhooks/mercadopago.ts:68-73`). For desktop-first users: silent no-op everywhere.

### A5 (MINOR) Wompi webhook first-payment branch never updates `plan_id`
- `web/src/pages/api/webhooks/wompi.ts:66-78` sets `provider`, `provider_subscription_id`, `payment_source_token` but not `plan_id` (dashboard path does: `dashboard/index.astro:56`). Webhook-only activation leaves the subscription on plan `'trial'`.

## Issue B — Mercado Pago: 502 on POST /api/checkout/mercadopago

### B1 (HIGH) MP API rejected the preference — exact error is in the response `detail`, hidden by the UI
- Endpoint: `web/src/pages/api/checkout/mercadopago.ts:48-69` — the `catch` returns **502** with `detail: err.message` (line 65-66). The reported 502 (237 B body) is this response, so `createPreference` threw — i.e. MP returned a non-2xx (or fetch failed).
- `web/src/lib/payments/mercadopago.ts:50-64` throws with the MP response text: `MercadoPago createPreference failed: <status> - <json>`.
- Frontend `web/src/components/CheckoutPlans.tsx:72-75` shows only `data.error` ("Failed to create payment preference"); `data.detail` (the actual MP error) is discarded — the user must open the 502 response body in DevTools to see it.
- Local env is structurally fine: `MP_API_URL` is 27 chars, no trailing slash, no "sandbox" (consistent with `https://api.mercadopago.com`); `MP_ACCESS_TOKEN` exists, `TEST-` prefixed, 72 chars (valid TEST-token shape). The endpoint itself is correct per the skill: `/checkout/preferences` (not `/v1/preferences`) — a prior commit (eb95e27) already reverted `/v1/preferences`.
- Most likely causes, in order: (1) expired/revoked/invalid TEST access token → MP 401; (2) MP payload validation 400 — e.g. `auto_return: "approved"` combined with an HTTP `success` back_url (`http://localhost:4321/dashboard?payment=success&provider=mercadopago`, from `SITE_URL` fallback at `checkout/mercadopago.ts:45-60`) can be rejected by MP in some configurations; (3) test app without Checkout Pro / seller setup for COP. The `detail` field identifies which.

### B2 (MEDIUM) Missing env var returns 500, not 502 — so Vercel vs local differ
- If `MP_ACCESS_TOKEN` is absent: endpoint returns 500 "Mercado Pago not configured" (`checkout/mercadopago.ts:37-43`), not 502. On Vercel this is what happens when the env vars aren't set in the project settings; locally the token exists (so the 502 is B1, an MP rejection).
- Env keys: `MP_ACCESS_TOKEN` (required), `MP_API_URL` (optional, defaults `https://api.mercadopago.com`, `mercadopago.ts:9-10`), `SITE_URL` (optional, defaults localhost).

### B3 (LOW) Same trial-subscription dependency affects MP success path
- `checkout/mercadopago.ts:73-78` does `UPDATE subscriptions SET provider='mercadopago', provider_subscription_id=? WHERE user_id=? AND status='trial'` — affects 0 rows for desktop-first users, so the MP webhook (`webhooks/mercadopago.ts:68-73` matching `provider_subscription_id`) later finds nothing. Fixing the 502 alone won't complete the flow for those users.

## Issue C — Webhook configuration: local vs production

| | Wompi | Mercado Pago |
|---|---|---|
| Where URL is set | Wompi dashboard (Developers → Events) — manual | Per-preference `notification_url` in code (`checkout/mercadopago.ts:61`), built from `SITE_URL`; dashboard-level config is an alternative |
| Local dev | HTTPS tunnel → `https://<tunnel>/api/webhooks/wompi` (Wompi blocks localhost) | Same tunnel → `SITE_URL=https://<tunnel>` so `notification_url` is reachable (default `http://localhost:4321` is unreachable by MP) |
| Production | `https://<vercel-domain>/api/webhooks/wompi` | `SITE_URL=https://<vercel-domain>` → `https://<vercel-domain>/api/webhooks/mercadopago` |
| Signature | `X-Event-Checksum` header; SHA256(properties-values + timestamp + `WOMPI_EVENTS_SECRET`) — **implementation wrong (A2)** | MP sends `x-signature` — **not verified at all** (`web/src/pages/api/webhooks/mercadopago.ts` has no signature check); webhook trusts any POST and calls MP API back via `getPayment` (`mercadopago.ts:73-95`) |
| Gating env keys | `WOMPI_API_URL`, `WOMPI_PUBLIC_KEY`, `WOMPI_PRIVATE_KEY`, `WOMPI_INTEGRITY_SECRET`, `WOMPI_EVENTS_SECRET`, `SITE_URL` | `MP_ACCESS_TOKEN`, `MP_API_URL`, `SITE_URL` |

Crons (`web/vercel.json:1-16`): expire-subscriptions 06:00, send-reminders 07:00, charge-wompi 08:00 UTC; handlers require `Authorization: Bearer ${CRON_SECRET}` (Vercel cron invocations attach this automatically). Correct by design; requires `CRON_SECRET` set in Vercel.

## Additional bugs discovered

1. **Renewal price mismatch**: `web/src/pages/api/cron/charge-wompi.ts:92-99` uses basico 49900 / pro 89900 / premium 149900, but `web/src/data/plans.ts:16-61` defines basico 89900 / pro 149900 / premium 259900 → renewals charge the wrong (lower) amount.
2. **`plan` vs `plan_id` column mismatch**: migrations define `plan_id` (`web/migrations/001_subscriptions.sql:4`; 002 adds provider columns; commit 114c468 fixed `createTrialSubscription` to use `plan_id`). Yet `web/src/lib/payments/lifecycle.ts:43,58,116,133` SELECTs `plan`, and `charge-wompi.ts:40` SELECTs `s.plan`. If the live table only has `plan_id`, those queries throw → expire-subscriptions cron 500s, charge-wompi cron 500s, `findByProviderSubId` broken. (Desktop's `020_web_schema.sql` defines `plan` but is not wired; the web schema is `plan_id`.)
3. **MP dashboard redirect handler is dead code for Checkout Pro**: `web/src/pages/dashboard/index.astro:106-188` looks for `?preapproval_id=` (preapproval/subscriptions API — not supported in Colombia and not used). MP Checkout Pro redirects append `?payment_id=...&status=approved&external_reference=...` to the success back_url (`checkout/mercadopago.ts:57`) → the block never runs; the MP success path relies solely on the webhook.
4. **`.env.example` missing** although AGENTS.md:66 says it documents env keys; only gitignored `web/.env` exists. No webhook/tunnel documentation anywhere in the repo (README.md:32,129 still mentions Stripe).
5. **Unused import**: `findByProviderSubId` imported in `web/src/pages/api/webhooks/wompi.ts:9` but never used (would trip `noUnusedLocals`).

## Affected Areas

- `web/src/pages/dashboard/index.astro:15-104` — Wompi verification fetch missing auth header (A1); dead MP preapproval block (add.3)
- `web/src/pages/api/payments/verify-wompi.ts:43-45` — same missing auth header; endpoint never called (A1)
- `web/src/lib/payments/wompi.ts:104-123` — wrong checksum computation (A2)
- `web/src/pages/api/webhooks/wompi.ts` — 401 on every event; no plan_id update (A2, A5); unused import
- `web/src/pages/api/checkout/mercadopago.ts:48-69` — 502 wrapping hides detail in UI; trial-subscription dependency (B1, B3)
- `web/src/components/CheckoutPlans.tsx:72-75,99-107` — discards `data.detail`; widget redirect
- `web/src/pages/api/webhooks/mercadopago.ts` — no signature verification (C)
- `web/src/pages/api/cron/charge-wompi.ts:40,63,92-99` — `s.plan` + wrong prices (add.1/2)
- `web/src/lib/payments/lifecycle.ts:43,58,116,133` — `plan` column selects (add.2)
- `web/src/actions/register.ts:48` / `src-tauri/src/application/use_cases/register.rs:207-227` — only web register creates trial subscription (A4)
- `web/.env.example` — missing (add.4); `web/.env` — sandbox keys for both gateways

## Approaches

1. **Fix-in-place (bug fixes, no re-architecture)** — add `Authorization: Bearer ${WOMPI_PUBLIC_KEY}` to the two transaction fetches; rewrite `verifyWebhookSignature` to follow `event.signature.properties` order (fallback to documented order) and include `amount_in_cents`; wire `verify-wompi` call in the widget callback; update `charge-wompi` prices + `plan` selects; surface `detail` in the UI.
   - Pros: smallest diff, unblocks all three user-reported issues immediately.
   - Cons: keeps the duplicated verification logic (dashboard vs webhook vs verify endpoint); security gaps (MP signature) remain unless separately patched.
   - Effort: Medium.

2. **Normalize into a single lifecycle module** — extract one `processWompiTransaction()` / `processMpPayment()` used by webhook, dashboard redirect, and verify endpoint; add MP x-signature verification; add a `syncSubscriptionForUser(userId)` that lazily creates the trial row for desktop-first users.
   - Pros: kills the silent-failure class of bugs; idempotency centralized; MP webhook becomes secure.
   - Cons: larger refactor; touches dashboard SSR and both webhooks; needs tests.
   - Effort: High.

3. **SDD change via propose→spec→design→tasks** — formalize as a change (`payments-webhook-and-errors`) covering the fixes above with acceptance scenarios.
   - Pros: traceability, verification step; fits existing openspec structure.
   - Cons: slower than direct fixes; user-reported issues are blocking.
   - Effort: Medium (planning) + implementation.

## Recommendation

Approach 1 first (unblocks the user: Wompi save path + MP 502 diagnosis), then fold Approach 2's safety items (MP signature verification, webhook checksum per `signature.properties`, lazy trial-row creation) into the same change as follow-up tasks. All three reported issues share one root pattern: **silent failures with no logging** — add `console.error` on every webhook/dashboard verification failure path.

## Risks

- CRITICAL: MP webhook accepts unauthenticated POSTs; an attacker who learns a payment id could activate a subscription without payment (and `getPayment` verifies against MP with the server token, so impact is limited but still requires MP webhook signature verification).
- CRITICAL: Wompi webhook 401s all real events (checksum bug) — if fixed but dashboard path stays broken, nothing changes; fix both.
- WARNING: `plan`/`plan_id` column mismatch means expire-subscriptions and charge-wompi crons likely throw on the live DB; verify actual Turso schema before touching crons.
- WARNING: renewal prices in `charge-wompi.ts` are stale (lower than checkout prices).
- WARNING: desktop-registered users have no subscription row; flows silently no-op for them until a row exists or is lazily created.
- SUGGESTION: create `web/.env.example` (documented env keys) and a webhook configuration section (tunnel for local dev, Vercel URL for prod).
- SUGGESTION: no tests cover the real webhook/verify endpoints (existing tests are mock-descriptions in `web/src/test/payments/*`); add integration tests for checksum + activation.

## Ready for Proposal

Yes — the three issues have confirmed root causes with file:line evidence. Recommend `sdd-propose` for change `payments-webhook-and-errors` (or a direct-fix batch given they are production-blocking).
