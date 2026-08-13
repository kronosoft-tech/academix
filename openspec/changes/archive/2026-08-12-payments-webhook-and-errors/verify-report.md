# Verify Report: payments-webhook-and-errors

Retrospective verification of commit `42c2d86` (`fix(web): secure payment activation and webhook error handling`). Scope: `web/` (Astro 7). Independent requirements/runtime verification against `spec.md` and `tasks.md`. Working tree at `42c2d86` during verification.

**Verification date**: 2026-08-12
**Mode**: hybrid (OpenSpec file + Engram) — BOTH backends written
**Revision**: 2 — corrected per SDD gatekeeper review (tsc defect reclassified CRITICAL; verdict `PASS WITH CRITICAL FINDING`; FU5 added; `next_recommended: archive-with-blocker`)

## Completeness

| Metric | Value |
|--------|-------|
| Tasks total (in scope) | 18 |
| Tasks complete | 18 |
| Tasks incomplete (in scope) | 0 |
| Follow-up tasks (FU1–FU5, out of scope) | 5 — correctly `[ ]` |

All 18 in-scope tasks (work units 1–8) are `[x]` in tasks.md and each maps to committed code verified by source inspection. No core or cleanup task is pending. FU5 is newly recorded by this verification (see CRITICAL).

## Runtime Evidence

| Check | Command | Exit | Result |
|-------|---------|------|--------|
| Unit tests | `bun run test` (web/) | 1 | 99 passed / 7 failed (106 total, 8 files) |
| Typecheck | `bunx tsc --noEmit` (web/) | 2 | 11 errors: **1 CRITICAL introduced by `42c2d86`**, 10 pre-existing |
| Build | `bun run build` (web/, astro build) | 0 | PASS (build does not run `tsc`) |

- `test_output_hash`: `sha256:8a55b62fa4ef551a35f3c5d783342845a0a0e4b0dfe3e8030fcfa58f8572bab7`
- `typecheck_output_hash`: `sha256:d8109f22567452418e8d5cb1ded94158017685c84e6d5eaa81ed378a84be0858`
- `build_output_hash`: `sha256:137890b7adb9cb7694c205ea89d0baca13b7b5cae500aabf835249eddf9a5fb0`

### Failure attribution

**NEW defect from `42c2d86`** (CRITICAL):
- `web/src/lib/payments/mercadopago.ts:88` — `error TS2322: Type '{}' is not assignable to type 'string'`. Confirmed introduced by this commit: the parent (`e904ebf`) threw a plain `Error` with no `detail`/`status` assignment; the whole error-handling block (lines 64–90) is new.

**Pre-existing failures (files untouched by `42c2d86`)**:
1. `src/test/payments.test.ts` — 5 Stripe tests fail with `Cannot find module '../lib/payments/stripe'`. Module absent at parent `e904ebf` too; documented in AGENTS.md. The commit's 6-line diff to this file only updates lifecycle SQL assertions (`trial_starts_at`, `grace_expires_at`) — those tests **pass**.
2. `src/test/payments/checkout-integration.test.ts` — 2 tests assert `geoToGateway('US')` / `geoToGateway(null)` return `'stripe'` (removed gateway). File and `gateway.ts` untouched by the commit.
3. TSC errors pre-existing (files not in the commit): `src/actions/register.ts:38`, `src/lib/db.ts:25`, 5× missing `stripe` module imports in `payments.test.ts`, `checkout-integration.test.ts:61`.

Changed-code tests all pass: `subscription-lifecycle.test.ts` 13/13, `payments.test.ts` lifecycle section 7/7, `webhook-flows.test.ts` 15/15, plus ai-rotator/pricing/download/auth suites.

## Compliance Matrix (spec scenarios)

Source-verified against `42c2d86`. `IMPLEMENTED` = code present and matches spec (static evidence); test column reflects runtime coverage.

| # | Scenario | Implementation | Test |
|---|----------|----------------|------|
| S1 | Wompi valid checksum → processed | ✅ `wompi.ts verifyWebhookSignature` (properties→values + timestamp + secret, SHA-256; fallback `[id,status,amount_in_cents]`) + webhook guard | ❌ UNTESTED (FU1) |
| S2 | Wompi missing/non-matching checksum → 401, no processing | ✅ missing header → 401; non-match → 401 before any branch | ❌ UNTESTED (FU1) |
| S3 | Approved transaction owned by caller → 200, active, 1 payment | ✅ `verify-wompi.ts` Bearer fetch, status + ownership checks, activate + idempotent insert | ❌ UNTESTED (FU1) |
| S4 | Ownership mismatch → 400 / dashboard banner | ✅ endpoint 400; dashboard `"Este pago no corresponde a tu cuenta."` | ❌ UNTESTED (FU1) |
| S5 | Wompi webhook desktop-first first payment | ✅ first-payment branch: `extractPlanIdFromReference`, `getOrCreateTrialSubscription`, UPDATE provider/provider_subscription_id/payment_source_token/plan_id, activate, idempotent payment | ❌ UNTESTED (lifecycle SQL indirectly covered) |
| S6 | MP preference created → 200 URL, ref stored | ✅ `createPreference` (auto_return, external_reference, notification_url) + checkout handler stores `provider_subscription_id` | ⚠️ PARTIAL (legacy integration mock passes; new fields unasserted) |
| S7 | MP rejects preference → 502 + detail, UI shows | ✅ error-body parse, rethrow with status/detail; handler 502; `CheckoutPlans` surfaces `data.detail || data.error` | ⚠️ PARTIAL (502 asserted; detail surfacing unasserted) |
| S8 | MP valid signature, approved → 200 + activated | ✅ HMAC-SHA256 manifest `id:<id>;request-id:<rid>;ts:<ts>;`, timing-safe; `processPayment` activates when approved + external_reference | ❌ UNTESTED (FU1) |
| S9 | MP processing failure → 500, MP retries | ✅ POST wrapper try/catch → 500 + `console.error` | ❌ UNTESTED (FU1) |
| S10 | Replayed payment → no duplicate | ✅ `activateApprovedPayment` early-return on `provider_payment_id`; same guard in Wompi `recordPayment`/verify/dashboard | ❌ UNTESTED (FU1; a Stripe-flow idempotency test exists but not this path) |
| S11 | MP desktop-first user → lazy trial row | ✅ `getOrCreateTrialSubscription` inside `activateApprovedPayment` | ❌ UNTESTED (FU1) |
| S12 | Dashboard approved redirect (payment_id/collection_id) | ✅ SSR verifies via `/v1/payments/{id}`, `activateApprovedPayment` when approved+ref; pending/failure copy; no `preapproval_id` | ❌ UNTESTED (SSR not covered) |
| S13 | Lazy trial creation (all paths), web-schema columns | ✅ `getOrCreateTrialSubscription` (trial → any row → create with `plan_id`/`trial_starts_at`); wired into all 5 paths (verify-wompi:89, webhooks/wompi:65, checkout/mp:78, activateApprovedPayment:208, dashboard:52) | ⚠️ PARTIAL (`createTrialSubscription` SQL asserted; `getOrCreateTrialSubscription` itself untested) |

Per the strict sdd-verify gate, scenarios without a passing covering test are `CRITICAL UNTESTED`. Per the graceful-handling clause, this project explicitly accepts review-only verification for the security-critical gap — design.md Testing Strategy documents "None in 42c2d86; review-only" and tasks.md FU1 records "gap confirmed" — so severity is WARNING (see Issues), not an archive blocker.

## Correctness (requirements)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Wompi Webhook Signature Verification (MODIFIED) | ✅ Implemented | `webhooks/wompi.ts:24-38`, `wompi.ts:116-151` |
| Wompi Transaction Ownership Verification (ADDED) | ✅ Implemented | `verify-wompi.ts:44-86`, `dashboard/index.astro:42-108`, `CheckoutPlans.tsx:99-112` |
| Wompi Webhook First-Payment Activation (ADDED) | ✅ Implemented | `webhooks/wompi.ts:58-87` |
| Preapproval Creation (REMOVED) | ✅ Removed | no preapproval calls in web; AGENTS.md documents Checkout Pro only |
| IPN Webhook Processing (REMOVED) | ⚠️ Removed for POST; GET IPN still present unverified | `webhooks/mercadopago.ts:92-112` — FU2, accepted by design |
| Checkout Pro Preference Creation (ADDED) | ✅ Implemented | `mercadopago.ts:28-93`, `checkout/mercadopago.ts:47-95` — **note: contains the CRITICAL tsc defect at line 88** |
| Webhook Signature Verification and Payment Processing (ADDED) | ✅ Implemented | `mercadopago.ts:138-162`, `webhooks/mercadopago.ts:28-90,114-137` |
| Replay-Safe Payment Activation (ADDED) | ✅ Implemented | `mercadopago.ts:180-246`; `webhooks/wompi.ts:148-177`; `verify-wompi.ts:114-134` |
| Dashboard Checkout Pro Redirect Handling (ADDED) | ✅ Implemented | `dashboard/index.astro:119-166` |
| Lazy Trial Subscription Creation (ADDED) | ✅ Implemented | `lifecycle.ts:42-64` + 5 call sites |

## Coherence (design decisions)

| Design decision | Followed? | Notes |
|-----------------|-----------|-------|
| Reference = `{userId}-{planId}-{uuid}` ownership proof | ✅ | checkout/mp:47, verify-wompi:81, dashboard:48 |
| Wompi checksum per `signature.properties` + fallback | ✅ | wompi.ts:120-128 |
| Bearer-auth fetches, 502 on failure | ✅ | verify-wompi:44-59, dashboard:29-38 |
| MP `x-signature` HMAC, timing-safe | ✅ | mercadopago.ts:144-161 |
| 500-on-failure webhook retry | ✅ | webhooks/mercadopago.ts:72-84 |
| Replay guard on `provider_payment_id` | ✅ | all 4 write paths |
| Lazy trial row (trial → any → create) | ✅ | lifecycle.ts:42-64 |
| Dashboard: Wompi `?id=` + MP `payment_id`/`collection_id`, no `preapproval_id` | ✅ | dashboard/index.astro:17-166 |
| Error taxonomy 400/401/500/502 | ✅ | matches design table for all 4 endpoints |
| Schema alignment (`plan_id`, `trial_starts_at`, `grace_expires_at`) | ✅ | lifecycle.ts all queries; tests updated to match |
| Doc updates (gateway scope, .env reality, skills) | ✅ | AGENTS.md diff verified |
| No DDL | ✅ | no migration files in commit |

## Issues Found

### CRITICAL

1. **New type error in changed code violates the repo pre-commit contract** (AGENTS.md: `bunx tsc --noEmit` must pass).
   - **Location**: `web/src/lib/payments/mercadopago.ts:88` — `if (detail) err.detail = detail;` → `error TS2322: Type '{}' is not assignable to type 'string'`.
   - **Root cause**: `detail` (lines 73–79) is inferred as `unknown`, not `string`. The guard `typeof (errorBody as Record<string, unknown>).detail === 'string'` applies to a **type-assertion expression**; TypeScript's control-flow analysis does not narrow property accesses through a cast, so the ternary's true branch type stays `unknown`. Assigning `unknown` to `err.detail` (declared `string | undefined` via the `Error & { detail?: string }` cast at line 86) fails with TS2322.
   - **Introduced by this commit**: the parent (`e904ebf`) threw a plain `Error`; the entire error-handling block is new.
   - **Impact**: typecheck fails (exit 2); runtime unaffected (the value IS a string when set) and `astro build` passes, but the repo's pre-commit checklist (`tsc --noEmit` green) is broken by this commit.
   - **Suggested fix** (report-only — NOT applied): drop the cast and let `in`-narrowing do the work on the reference itself, which TS narrows properly:
     ```ts
     const detail =
       errorBody != null &&
       typeof errorBody === 'object' &&
       'detail' in errorBody &&
       typeof errorBody.detail === 'string'
         ? errorBody.detail   // narrowed to string via the in + typeof guards
         : null;
     ```
     Alternative: annotate `const detail: string | null` and cast the true branch (`as string`), or extract a `getMpErrorDetail(body: unknown): string | null` helper. **A one-line fix — but still a NEW code change requiring an explicit follow-up commit (FU5).**

### WARNING

1. **Security-critical functions have zero automated tests** (the known gap; Wompi checksum, MP HMAC, replay guard, ownership check). Strict rubric status: `CRITICAL UNTESTED` for scenarios S1–S5, S8–S12. Downgraded to WARNING because design.md/tasks.md explicitly accept review-only for this retrospective change (FU1 "gap confirmed"). **Schedule FU1 before further payment changes** — until it lands, regressions in signature/replay/ownership logic are detectable only by manual review.
2. **Full unit suite exits 1** due to 7 pre-existing legacy failures (5× missing `stripe` module, 2× `geoToGateway` expecting removed `'stripe'`). Not caused by this change, but a red suite masks future regressions; clean up per AGENTS.md guidance.
3. **MP legacy GET IPN remains unverified by design** (`webhooks/mercadopago.ts:92-112`, FU2). Unauthenticated GET activates approved payments; data is server-fetched from MP (not attacker-supplied) so exploitability is limited, but it bypasses signature verification and MP does not require GET IPN with Checkout Pro. Disable or HMAC-verify it.
4. **`MP_WEBHOOK_SECRET` is a new mandatory env var** (spec: unset → 500, implemented correctly). Deployment risk: until configured in the Vercel environment, every MP webhook returns 500 and MP retries indefinitely — check env config before/after deploy.
5. **`charge-wompi` cron is broken** (`web/src/pages/api/cron/charge-wompi.ts:40` selects `s.plan` — column no longer exists; stale `getPlanPriceCOP` prices) — FU3, out of scope, but renewal charging via cron currently fails at runtime; the webhook renewal branch is the working path.

### SUGGESTION

- Wompi checksum comparison is non-timing-safe string equality (design open question). Low practical impact: the checksum travels in the request header (public), so a timing leak does not expose the secret; unlike the MP HMAC case there is no secret in the compared value.
- `planId` extraction is positional (`parts[5]`, default `basico`) — breaks if a plan id ever contains `-`.
- Concurrent checkouts race on the single trial row (`provider_subscription_id` overwrite).
- FU4: `WOMPI_API_URL`/`WOMPI_PUBLIC_KEY` duplicated in `wompi.ts`, `verify-wompi.ts`, `dashboard/index.astro`.
- `checkout/mercadopago.ts` surfaces `err.message` (which embeds MP `detail`); using `err.detail` directly would be cleaner and spec-literal.

## Follow-up tasks (FU1–FU5)

Verified **correctly NOT in the implemented scope** of `42c2d86`:

- **FU1** (RED tests for checksum/HMAC/replay/ownership): no such tests in the commit — confirmed by grep across `web/src/**/*.test.ts` (only Stripe `verifyWebhookSignature` tests exist). tasks.md: `[ ]`, "gap confirmed".
- **FU2** (disable/verify GET IPN): GET handler present and unverified — confirmed `webhooks/mercadopago.ts:92-112`. tasks.md: `[ ]`.
- **FU3** (fix charge-wompi cron): broken select confirmed at `cron/charge-wompi.ts:40`. tasks.md: `[ ]`.
- **FU4** (extract WOMPI consts): duplicated consts confirmed across 3 files. tasks.md: `[ ]`.
- **FU5** (NEW, added by this verification): fix `web/src/lib/payments/mercadopago.ts:88` TS2322 (`Type '{}' is not assignable to type 'string'`) — narrow `detail` to `string` via `in`-narrowing on the reference (see CRITICAL fix) so `bunx tsc --noEmit` passes. Not in tasks.md; must be recorded by the orchestrator.

None of FU1–FU5 correspond to any spec requirement in spec.md — the spec defines only the implemented behavior. Out-of-scope marking is correct.

## Verdict

**PASS WITH CRITICAL FINDING** (corrected from PASS WITH WARNINGS)

All 18 in-scope tasks complete; all 8 spec requirements implemented and source-verified; changed-code tests pass; build passes. **However, `42c2d86` introduces a new type error (`mercadopago.ts:88`, TS2322) that breaks the repo's pre-commit contract (`bunx tsc --noEmit` must pass) — a CRITICAL defect in shipped code.** The security-testing gap (FU1) is a documented, accepted follow-up; the remaining warnings are non-blocking but must be tracked.

**Archive disposition**: `archive-with-blocker`. Archive of this change may proceed **only if the user explicitly accepts the `mercadopago.ts:88` defect as a documented follow-up (FU5)** — i.e. the tsc-red state of the commit is knowingly carried forward. Otherwise STOP and surface the defect for remediation before archiving. FU1–FU3 and FU5 should be scheduled as a follow-up change regardless.
