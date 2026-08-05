# Tasks: User Dashboard Web

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 550–650 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 (foundation+navbar) → PR 2 (dashboard pages+tests) |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Auth helper + plans constant + migration + UserNavbar | PR 1 | `bunx tsc --noEmit && bun run test -- auth plans` | `bun run build` (SSR renders navbar) | Revert auth.ts addition + delete plans.ts/migration/navbar changes |
| 2 | Dashboard metrics + subscription + payments pages + integration tests | PR 2 | `bun run test -- dashboard subscription payments` | `bun run build && bun run dev` (visit /dashboard) | Delete new page files + revert index.astro |

## Phase 1: Foundation (data layer + auth)

- [x] 1.1 Create `web/migrations/001_subscriptions.sql` — idempotent CREATE TABLE IF NOT EXISTS for `subscriptions` (id, user_id, plan_id, status, trial_end, current_period_end, created_at, updated_at) + `subscription_payments` (id, subscription_id, amount, currency, method, status, paid_at)
- [x] 1.2 Add `getFullTokenPayload(cookies)` to `web/src/lib/auth.ts` — re-verify JWT cookie, return full `CustomerJwtPayload` or null
- [x] 1.3 Create `web/src/data/plans.ts` — export `Plan` interface + `PLANS` constant (Básico 49900/Pro 89900/Premium 149900 COP)

## Phase 2: Navigation (UserNavbar)

- [x] 2.1 Rewrite `web/src/components/UserNavbar.astro` — server-side cookie verify via `getFullTokenPayload`; anon → login button; auth → initials avatar + native `<details>/<summary>` dropdown (Dashboard, Mi Academia, Suscripción, Descargar App, Cerrar sesión)

## Phase 3: Dashboard Pages

- [ ] 3.1 Rewrite `web/src/pages/dashboard/index.astro` — auth guard (redirect if null); query user DB for student/course/group counts + income/expenses aggregates + recent payments; error banner on DB failure; empty state with CTA; pass income data to `<DashboardCharts client:load />`
- [ ] 3.2 Create `web/src/pages/dashboard/subscription.astro` — auth guard; query control-plane `subscriptions` by user_id; render plan name/status/period end/limits from PLANS; upgrade CTA for non-Premium; trial state; handle no-subscription
- [ ] 3.3 Create `web/src/pages/dashboard/payments.astro` — auth guard; query control-plane `subscription_payments` with LIMIT/OFFSET from `?page=N`; render table (date, amount+currency, method, status); empty state

## Phase 4: Testing

- [ ] 4.1 Unit tests for `getFullTokenPayload` — valid→payload, missing→null, expired→null, invalid→null
- [ ] 4.2 Unit tests for `PLANS` — shape validation, prices, plan IDs match expected
- [ ] 4.3 Integration tests for dashboard index — mocked getUserDb: data state (renders counts+chart), empty state (zeros+CTA), DB error (degraded banner)
- [ ] 4.4 Integration tests for subscription page — mocked db: active plan, trial, Premium no-CTA
- [ ] 4.5 Integration tests for payments page — mocked db: rows rendered, empty state, pagination offset

## Phase 5: Verification

- [ ] 5.1 Run `bunx tsc --noEmit` — zero errors
- [ ] 5.2 Run `bun run build` — SSR pages compile successfully
