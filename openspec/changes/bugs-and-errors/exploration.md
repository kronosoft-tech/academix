# Exploration: bugs-and-errors — full codebase review

Read-only audit of the desktop app (root: Tauri 2 + React 19) and the web app (`web/`: Astro 7) for bugs, error-handling gaps, and security vulnerabilities. Skills loaded: `security-checklist`, `form-security`, `turso-security`, `mercadopago`, `wompi`, `astro-7`. No files modified. Findings are pre-existing (not introduced by this review).

## Current State

- **Web billing loop**: checkout (Wompi/MP) → webhook → `lifecycle.activateSubscription` on the shared control-plane DB; two crons drive renewals (`charge-wompi` 08:00 UTC) and reminders (`send-reminders` 07:00 UTC) daily in prod.
- **Desktop data loop**: `MemoryBuffer` (in-memory writes) → `flush_timer` (immediate async flush + exponential backoff) → per-user Turso DB; reads cached per table. Auth resolves sessions via buffer first, then Turso; subscription status resolved from control plane and cached on disk (24 h grace).
- **Per-user provisioning**: web `actions/register.ts` → `provisioning.ts` (create DB + full-access auth token) → shared `users`/`user_databases` rows; desktop does the same via `use_cases/register.rs` + `control_plane.rs`.

## Affected Areas

### Web — CRITICAL

| # | Location | Issue | Evidence |
|---|----------|-------|----------|
| W1 | `web/src/pages/api/cron/charge-wompi.ts:43` | Selects `s.plan_id` — column exists in `web/migrations/001_subscriptions.sql:4`; no `s.plan` reference anywhere. Query runs clean → Wompi renewals are charged normally. **No bug found.** | SELECT is `s.id, s.user_id, s.plan_id, s.payment_source_token, u.email`; schema defines `plan_id` |
| W2 | `web/src/pages/api/cron/charge-wompi.ts:65,79` | No `getPlanPriceCOP` exists; renewal amount is `getPlanById(sub.plan_id).priceCOP * 100`, reading current `web/src/data/plans.ts` (basico 89900, pro 149900, premium 259900). Renewals charge current prices. **No bug found.** | `getPlanPriceCOP` absent from tree (grep); `plans.ts` prices are 89900/149900/259900 |

### Web — HIGH

| # | Location | Issue |
|---|----------|-------|
| W3 | `web/src/pages/api/cron/send-reminders.ts:78,82,87` | Selects `s.grace_expires_at` — column exists in `002_multi_gateway.sql:10`; no `s.grace_end` reference. Query runs clean → grace warnings are sent normally. **No bug found.** |
| W4 | `web/src/pages/api/webhooks/{wompi,mercadopago}.ts` | No amount vs plan-price verification; `planId` taken from client-generated `reference` (spoofable); `activateSubscription` extends +30 days from **now**, not period end. |
| W5 | `web/src/pages/api/webhooks/mercadopago.ts` `classifyMpError` | DB-down (no `.status`) → HTTP 500; MP **abandons retries on 500** → payment silently lost. Must return 503. |
| W6 | `web/src/pages/api/payments/verify-wompi.ts:47` | `Authorization: Bearer ${WOMPI_PUBLIC_KEY}` to GET transaction — Wompi requires the **private** key → fallback verify likely always 401 (unverified, no runtime). |
| W7 | `web/src/lib/provisioning.ts` `createAuthToken` | Full-access token, **no expiration**, embedded in 7-day JWT and stored plaintext in control-plane `user_databases`; never rotated/revoked. |
| W8 | `web/src/actions/register.ts` | Email-exists check non-atomic (race); on failure per-user DB deleted but shared `users`/`user_databases` rows orphaned → email permanently blocked from re-registering. |
| W9 | `web/src/pages/api/cron/charge-wompi.ts` | No charge idempotency → webhook failure → next day charges again (double charge). |
| W10 | `web/src/lib/payments/lifecycle.ts` | `activateSubscription` check-then-act on `provider_payment_id` with **no unique constraint**; duplicate MP events (created+updated) can double-record/double-extend. |

### Web — MEDIUM / LOW

| # | Location | Issue |
|---|----------|-------|
| W11 | `web/src/lib/payments/gateway.ts` | `geoToGateway(null)` returns `'wompi'` — contradicts AGENTS.md (null → mercadopago). Code/docs mismatch. |
| W12 | `web/src/pages/api/chat.ts` | Accepts client `role: 'system'` → prompt injection; no rate limit/quota, no subscription gate → cost abuse. |
| W13 | `web/src/actions/contact.ts` | User input interpolated into HTML email unescaped → email HTML injection; no honeypot/rate limit → spam/email bombing. |
| W14 | `web/src/actions/password-reset.ts` | Reset token stored **plaintext** (not hashed); token never consumed/invalidated after reset; no rate limit → email bombing. |
| W15 | `web/src/actions/{login,admin-login}.ts` | No login rate limiting / account lockout. |
| W16 | `web/src/lib/payments/wompi.ts` `verifyWebhookSignature` | String comparison, not timing-safe (MP uses `timingSafeEqual`). |
| W17 | Migrations/deployment | Control-plane tables `web_admins`, `password_resets`, `downloads`, `ai_conversations`, `pqrs_tickets` exist only in `020_web_schema.sql` — NOT wired for the web shared DB (desktop 020 not registered in `lib.rs`; web has no shared-DB auto-runner). 020 also defines **shadow** `subscriptions`/`subscription_payments` with conflicting schema (`plan`, `grace_end`, CHECK constraints) → drift risk. |
| W18 | `web/src/lib/email.ts` | Silently returns `true` when Gmail creds missing → password reset/contact fail **open**. |
| W19 | `web/src/pages/api/download.ts` | Trusts `x-forwarded-for` (client-controlled IP); no rate limit. |
| W20 | Migrations 001/002 | No unique constraint on `subscriptions.user_id`; no unique on `subscription_payments.provider_payment_id`. |

### Desktop — HIGH

| # | Location | Issue |
|---|----------|-------|
| D1 | `src-tauri/src/lib.rs` `run_local_migrations` | `is_legacy = applied.is_empty()` conflates **fresh install** with legacy → fresh installs seed 001–018 as applied and return, **skipping real schema migrations 001 and 019**. |
| D2 | `src-tauri/src/commands/auth.rs` | Subscription status **fail-open**: no subscription row OR control-plane error → treated `active` + cached `"active"` (error cache-poisoning) → free access (24 h offline grace via `subscription_cache.rs`). |
| D3 | `src-tauri/src/commands/auth.rs` | `change_password` / `update_profile` update **only** the per-user DB → shared control-plane `users` row desyncs; email change breaks web login (`user_databases.email` mapping stale). |
| D4 | `src-tauri/src/commands/auth.rs` | Session expiry broken: `expires_at` stored RFC3339 (`T`, `+00:00`) compared lexicographically vs `datetime('now')` (space) → comparison **always true** → sessions never expire (leaked token = valid forever). |
| D5 | `src-tauri/src/commands/payments.rs` | `sync_payments_to_accounting` + `list_accounting_entries_by_reference` match `description.contains(payment_id[..8])` but synced entries have `description = student_id` → filter never matches → **duplicate accounting entries every run**; `delete_payment` leaves orphan entries. |
| D6 | `src-tauri/src/commands/payments.rs` | `create_payment(paid=true)` then `update_payment(status=paid)` → **double accounting income** for the same payment (first entry keyed by student_id description, second by payment id). |

### Desktop — MEDIUM / LOW

| # | Location | Issue |
|---|----------|-------|
| D7 | `src-tauri/src/commands/register.rs` / `use_cases/register.rs` | `cp.save_user_db` / `cp.save_user` marked **non-fatal** → registration succeeds with no mapping/row → account unusable on web or desktop; no cleanup of orphaned DB on migration failure. |
| D8 | `src-tauri/src/infrastructure/turso/memory_buffer.rs` | Writes lost on app crash (in-memory only, no WAL/journal); flush retry can **duplicate INSERTs** (no idempotency). `find_session_by_token` has no expiry check. Read cache **never expires** → stale data across devices until local write. |
| D9 | `src-tauri/src/infrastructure/turso/flush_timer.rs` | Ops executed without transaction; partial failure leaves partial state; `flush_on_close` 5 s timeout → data loss on close. |
| D10 | `src-tauri/src/lib.rs` | Dormant default-admin seeding with **hardcoded bcrypt hash** (`ADMIN_EMAIL`/`ADMIN_PASSWORD_HASH` defaults); `seed_control_plane_admin` passes `libsql://{hostname}` URL instead of bare DB name to `create_auth_token` (known bug, disabled path). |
| D11 | `src-tauri/src/infrastructure/turso/provisioning.rs` | `&name[..25]` byte-slice could panic on multi-byte UTF-8 (slugs are ASCII → low). |
| D12 | Money as `f64` | `amount: f64` throughout payments/accounting (DTO + domain) → rounding drift on sums. |
| D13 | `src/app/components/ErrorBoundary.tsx` | **Dead code** — defined, never imported → render crash = white screen, no recovery UI. |
| D14 | `src-tauri/capabilities/default.json` | Grants `sql:default` + `process:default` while `tauri-plugin-sql` is a stale dependency (frontend only uses Rust commands) → unnecessary attack surface. |
| D15 | `src-tauri/src/infrastructure/subscription_cache.rs` | Plaintext JSON cache, no tamper protection → trivial local subscription bypass (single-user threat; low). |

### Positives noted

- `tauri.conf.json` CSP is solid: `default-src 'self'; script-src 'self'` (no unsafe-inline scripts); updater `pubkey` set → signature-verified updates; updater endpoints https-only.
- Wompi webhook signature verification (SHA256 integrity) is implemented; MP uses `timingSafeEqual`.
- `middleware.ts` correctly splits customer/admin JWT roles and enforces route access.
- Desktop command layer consistently returns `Result` and buffers writes with retry/backoff; session tokens are UUIDv7 (not guessable).

## Approaches

1. **Fix-first (hotfixes, then hardening)** — ship targeted fixes for W1/W2/W3 (crons) and D1/D4 (migrations, session expiry) immediately, then address W4–W10 and D2/D3/D5/D6 in a structured change.
   - Pros: stops revenue bleed and free-access holes fast; small diffs, easy review.
   - Cons: still leaves the token/idempotency/registration-orphan debt; two waves of work.
   - Effort: Medium

2. **Full remediation change** — one change covering all 26 findings with specs + tasks (this exploration → proposal → spec → design → tasks → apply).
   - Pros: single coherent SDD cycle; complete security posture; tests codify the fixes.
   - Cons: large surface; higher review burden; slow to ship the two CRITICAL cron bugs.
   - Effort: High

3. **Split into themed changes** — (a) `billing-cron-fixes` (W1–W3), (b) `payment-integrity` (W4–W6, W9, W10, W20), (c) `auth-and-provisioning-hardening` (W7, W8, W14, W15, D2, D3, D4), (d) `desktop-data-integrity` (D1, D5, D6, D8, D9).
   - Pros: reviewable units per domain; each ships independently; keeps SDD per change.
   - Cons: more proposal/spec overhead; coordination across changes.
   - Effort: Medium-High

## Recommendation

**Approach 3 (split into themed changes), starting with `billing-cron-fixes`** as the first proposal: W1 and W2 are active revenue bugs running daily in prod, and W3 is a silent customer-experience failure — they are small, isolated, and testable (cron handler + plan price source + regression tests). Auth/session (D2/D3/D4) and payment integrity (W4–W10) follow as separate changes with their own specs. The remaining items (W11–W20, D7–D15) fold into the relevant change as low-severity hardening tasks. Hotfix-first (approach 1) is a valid alternative if the user wants W1–W3 merged immediately outside the SDD cycle.

## Risks

- **Cron failures are silent in prod**: no alerting on 500s; the W1/W2 bugs may have gone unnoticed for weeks (data loss of renewal revenue is unrecoverable retroactively).
- **Column-name bugs (W1, W3) are a class of bug**: tests mock the wrong column names (`s.plan`, `s.grace_end`) — stale test fixtures mask schema drift; any migration change needs a schema-level test against real migrations.
- **Schema drift between desktop and web**: 020 shadow tables + per-user mirror sync must be verified before any migration work.
- **Token exposure**: full-access non-expiring tokens in JWTs and control-plane storage — rotation requires a migration + JWT claim change (breaks existing sessions).
- **Payment idempotency**: fixing W9/W10 needs unique constraints on production data (dedupe first).
- **No runtime available**: findings verified by static analysis + greps; W6 (verify-wompi auth) and cron 500s are inferred, not observed.

## Ready for Proposal

**Yes** — exploration is complete. Recommend the orchestrator launch `sdd-propose` for the first change (`billing-cron-fixes`: W1, W2, W3) and optionally split the remaining findings into `payment-integrity`, `auth-and-provisioning-hardening`, and `desktop-data-integrity` proposals.