# Tasks: Academix Web

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 3500–5000 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (Phase 1: Landing+Downloads) → PR 2 (Phase 2: Auth+Dashboards) → PR 3 (Phase 3: Payments+Support) |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Static marketing + downloads portal | PR 1 | `bun run build && bun run test` in `web/` | `bun run dev` → visit `/`, `/downloads`, `/pricing` | Remove `web/` directory entirely |
| 2 | Auth system + customer/admin dashboards | PR 2 | `bun run test -- --filter auth && bun run test:e2e` | `bun run dev` → register → login → dashboard | Revert PR 2 branch; drop migration 020 tables |
| 3 | Payments + PQRS + AI chat | PR 3 | `bun run test -- --filter payments && bun run test -- --filter chat` | Stripe CLI `stripe trigger` → verify webhook; SSE chat | Revert PR 3 branch; tables are additive |

---

## Phase 1: Landing + Downloads (Static Pages, Content, Download Endpoint)

- [x] 1.1 Scaffold Astro project in `web/`: `package.json` (bun), `astro.config.mjs` (Vercel adapter, Tailwind v4 vite plugin, React integration), `tsconfig.json` — [Spec §1, §2, §8]
- [x] 1.2 Create `web/src/layouts/Base.astro` with shared head, nav slot, footer — [Spec §1]
- [x] 1.3 Create `web/src/components/UserNavbar.astro` server island (session-aware placeholder for Phase 2) — [Design: Islands]
- [x] 1.4 Build landing page `web/src/pages/index.astro`: hero, features, testimonials, video embed — [Spec §1 scenario: visitor views landing]
- [x] 1.5 Create FAQ page `web/src/pages/faq.astro` using content collection — [Spec §1]
- [x] 1.6 Create contact page with Astro action `web/src/actions/contact.ts` (Zod validation, Resend email) — [Spec §1 scenarios: contact form submission/validation]
- [x] 1.7 Create `web/src/data/pricing.json` and `web/src/components/PricingByCountry.astro` server island (geo header → localized price, USD fallback) — [Spec §8 all scenarios]
- [x] 1.8 Build pricing page `web/src/pages/pricing.astro` with static shell + server island — [Spec §8]
- [x] 1.9 Set up content collections in `web/src/content/`: `tutorials/` and `faq/` schemas — [Design: Content]
- [x] 1.10 Create tutorial listing page `web/src/pages/tutorials/index.astro` and dynamic `[...slug].astro` — [Proposal: tutorial content]
- [x] 1.11 Build `web/src/components/DownloadSelector.tsx` client island (OS detection, arch selector) — [Spec §2 scenarios: auto-detected OS, unknown OS fallback]
- [x] 1.12 Create downloads page `web/src/pages/downloads.astro` wiring the DownloadSelector island — [Spec §2]
- [x] 1.13 Create `web/src/pages/api/download.ts` endpoint: validate against GitHub allow-list, increment counter, redirect — [Spec §2 scenario: download tracking; Threat: open-redirect]
- [x] 1.14 Unit tests (Vitest): PricingByCountry lookup + USD fallback, download endpoint allow-list validation rejects non-GitHub URLs — [Design: Testing]

## Phase 2: Auth + Dashboards (JWT, Middleware, Customer/Admin)

- [x] 2.1 Create migration `src-tauri/migrations/020_web_schema.sql` (web_admins, subscriptions, subscription_payments, invoices, pqrs_tickets, pqrs_responses, password_resets, downloads, ai_conversations) — [Design: Interfaces]
- [x] 2.2 Create `web/src/lib/db.ts`: `@libsql/client` connection using env vars (`TURSO_URL`, `TURSO_AUTH_TOKEN`) — [Design: DB layer]
- [x] 2.3 Create `web/src/lib/auth.ts`: JWT sign/verify with `jose` (CustomerJwtPayload, AdminJwtPayload), bcryptjs hash/compare, cookie set/clear helpers — [Spec §3; Design: Interfaces]
- [x] 2.4 Create `web/src/middleware.ts`: verify JWT cookie, populate `locals.user`/`locals.admin`, protect `/dashboard/*` and `/admin/*`, redirect unauth to login, 403 on cross-type access — [Spec §3 scenario: middleware blocks; Spec §4 scenario: non-admin denied; Threat: privilege crossing]
- [x] 2.5 Create customer auth actions in `web/src/actions/`: `register.ts`, `login.ts`, `logout.ts`, `password-reset.ts` — [Spec §3 scenarios: registration, login, password reset]
- [x] 2.6 Create admin auth action `web/src/actions/admin-login.ts` (validates against web_admins table) — [Spec §4]
- [x] 2.7 Build auth pages: `web/src/pages/auth/login.astro`, `register.astro`, `reset-password.astro`, `web/src/pages/admin/login.astro` — [Spec §3]
- [x] 2.8 Build customer dashboard `web/src/pages/dashboard/index.astro`: subscription status, payment history, expiration alert, no-subscription prompt — [Spec §5 all scenarios]
- [x] 2.9 Build admin dashboard `web/src/pages/admin/index.astro`: client count, revenue, downloads, payments, churn rate — [Spec §4 scenario: admin views dashboard]
- [x] 2.10 Create `web/src/components/DashboardCharts.tsx` client island (Recharts) for admin/customer metrics — [Design: Islands]
- [x] 2.11 Create `web/src/layouts/Dashboard.astro` and `web/src/layouts/Admin.astro` layouts — [Design: File Changes]
- [x] 2.12 Unit tests: JWT sign/verify + type discrimination, bcrypt cross-cost verify, middleware routing (customer OK, admin OK, cross-type 403, unauth redirect) — [Design: Testing]
- [x] 2.13 E2E tests (Playwright): customer register → login → dashboard, admin login → admin dashboard, protected-route redirect, cross-domain 403 — [Design: Testing]

## Phase 3: Payments + Support (Stripe, PQRS, AI Chat)

- [x] 3.1 Create `web/src/lib/payments/stripe.ts`: Stripe client init, signature verification, event normalization — [Spec §6 scenario: webhook signature validation; Threat: webhook forgery]
- [x] 3.2 Create `web/src/pages/api/webhooks/stripe.ts`: verify sig (401 on invalid), handle `checkout.session.completed`, `invoice.payment_succeeded`, `invoice.payment_failed`, update subscriptions table — [Spec §6 scenarios: new subscription, trial, payment failure grace]
- [x] 3.3 Implement subscription lifecycle logic in `web/src/lib/payments/lifecycle.ts`: trial(7d) → active → grace(7d) → expired — [Spec §6 scenarios: trial expires, grace period expires]
- [x] 3.4 Create Stripe checkout action `web/src/actions/create-checkout.ts` and cancel action `web/src/actions/cancel-subscription.ts` — [Spec §5, §6]
- [x] 3.5 Create PQRS action `web/src/actions/pqrs.ts` (create ticket) and support pages `web/src/pages/dashboard/support/index.astro` (list tickets), `new.astro` (form) — [Spec §7 scenarios: customer creates ticket, views status]
- [x] 3.6 Create AI provider interface `web/src/lib/ai/types.ts`, Groq impl `groq.ts`, Cerebras impl `cerebras.ts`, rotator `rotator.ts` (round-robin + fallback) — [Spec §7 scenarios: chat message, provider failure, all unavailable]
- [x] 3.7 Create SSE endpoint `web/src/pages/api/chat.ts` using provider rotator — [Design: Data Flow]
- [x] 3.8 Create `web/src/components/AIChat.tsx` client island (React, SSE consumer, error states) — [Spec §7; Design: Islands]
- [x] 3.9 Create grace period checker (Vercel cron or scheduled endpoint) to expire subscriptions past grace — [Spec §6 scenario: grace period expires]
- [x] 3.10 Integration tests: Stripe webhook signature (valid/invalid→401), lifecycle transitions (trial→active→grace→expired), download counter + redirect allow-list — [Design: Testing]
- [x] 3.11 E2E tests: PQRS create/list, AI chat message flow (mocked provider) — [Design: Testing]
