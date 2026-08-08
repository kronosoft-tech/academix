# Design: User Dashboard Web

## Technical Approach

Deliver the dashboard as server-rendered Astro pages (`export const prerender = false`) that read data at request time, following the existing pattern in `dashboard/index.astro`. Metrics come from the user's individual Turso DB via `getUserDb(payload)`; subscription/payment data comes from the control-plane DB via the existing `db` proxy. Mutations stay out of scope — every page is read-only. Interactivity (navbar dropdown, charts) uses the established React-island / native-HTML approach.

## Architecture Decisions

| # | Decision | Chosen | Alternative rejected | Rationale |
|---|----------|--------|----------------------|-----------|
| 1 | Access full JWT in pages | Add `getFullTokenPayload(cookies)` in `auth.ts` that re-verifies the cookie and returns `CustomerJwtPayload` | Extend `Astro.locals` with `dbUrl`/`dbToken` | `locals.user` deliberately omits credentials; spreading `dbToken` into every route widens exposure. Helper keeps creds scoped to pages that query the user DB. |
| 2 | UserNavbar auth detection | Server-side cookie verification inside the `.astro` component; native `<details>`/`<summary>` for the dropdown | Client React island; rely on `locals.user` | Middleware skips verification on public routes, so `locals.user` is undefined there. Component must verify the cookie itself. `<details>` needs zero JS and is accessible. Credentials never reach the client (SSR-only). |
| 3 | Control-plane migration | Versioned idempotent SQL file `web/migrations/001_subscriptions.sql`, applied via Turso CLI before deploy | Inline DDL in request/lifecycle code; startup runner | App is serverless (Vercel) — no natural startup hook; DDL in the request path is unsafe on a production control-plane DB. A file matches the desktop app's migration convention and keeps DDL out of hot paths. |
| 4 | Charts data shape | `index.astro` aggregates user-DB payments into `{ label, value }[]` (last 6 months income) and passes to `<DashboardCharts client:load data={...} />` | Fetch inside the React component | Component already accepts `{ label, value }[]`; keeping the query server-side avoids exposing `dbToken` to the client. `DashboardCharts.tsx` is reused unchanged. |
| 5 | User-DB error handling | Single `try/catch` around client + queries; on throw set `dbError` → degraded banner; success with zero rows → empty/zero state + desktop-download CTA | Let errors bubble | User DBs can be transiently unavailable; the page must never 500. Distinguishing error vs empty gives correct UX. |

## Data Flow

    Browser ──cookie──▶ Astro page (prerender:false)
                          │
        getFullTokenPayload(cookies) ─▶ CustomerJwtPayload
                          │
          ┌───────────────┴────────────────┐
          ▼                                 ▼
    getUserDb(payload)                 db (control-plane)
    student/course/group counts,       subscriptions +
    income/expense aggregates          subscription_payments
          │                                 │
          └──────▶ page HTML + <DashboardCharts client:load/>

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `web/src/lib/auth.ts` | Modify | Add `getFullTokenPayload(cookies): Promise<CustomerJwtPayload \| null>` — verifies cookie, returns full payload or `null`. |
| `web/src/components/UserNavbar.astro` | Modify | Server-verify cookie; anonymous → login button; authenticated → initials avatar + `<details>` dropdown (Dashboard, Mi Academia, Suscripción, Descargar App, Cerrar sesión). |
| `web/src/pages/dashboard/index.astro` | Modify | Replace body with user-DB metrics (students/courses/groups/income/expenses/recent payments) + charts; error/empty states. |
| `web/src/pages/dashboard/subscription.astro` | Create | Current plan, status, period end, plan limits; upgrade CTA for non-Premium. |
| `web/src/pages/dashboard/payments.astro` | Create | Paginated (`?page=N`, LIMIT/OFFSET) control-plane payment history. |
| `web/src/data/plans.ts` | Create | `PLANS` constant: Básico $49,900 / Pro $89,900 / Premium $149,900 COP with display limits. |
| `web/migrations/001_subscriptions.sql` | Create | Idempotent `CREATE TABLE IF NOT EXISTS subscriptions` + `subscription_payments`. |

## Interfaces / Contracts

```typescript
// auth.ts
export async function getFullTokenPayload(
  cookies: AstroCookies
): Promise<CustomerJwtPayload | null>;

// data/plans.ts
export interface Plan {
  id: 'basico' | 'pro' | 'premium';
  name: string;
  priceCOP: number;      // monthly, e.g. 49900
  maxStudents: number | null;   // null = unlimited
  maxUsers: number | null;
  features: string[];
}
export const PLANS: Plan[];
```

Logout reuses the existing `logout` Astro action (mutation-via-action convention).

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `getFullTokenPayload` valid → payload; missing/expired/invalid → `null` | Vitest, sign tokens with test secret |
| Unit | `PLANS` shape/values match spec | Vitest assertion |
| Integration | Dashboard metrics: with data, empty (zeros+CTA), DB error (degraded banner) | Mock `getUserDb`; assert render branches |
| Integration | Subscription page: active / trial / Premium (no CTA) | Mock control-plane `db` |
| Integration | Payments: rows render, empty state, pagination | Mock `db`, assert LIMIT/OFFSET |

## Threat Matrix

N/A — no routing changes, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary. Change is SSR pages + read-only DB queries + one idempotent SQL file.

## Migration / Rollout

Apply `web/migrations/001_subscriptions.sql` to the control-plane DB via `turso db shell` before deploying the pages. Idempotent (`IF NOT EXISTS`) — safe to re-run. Rollback: `DROP TABLE IF EXISTS` both tables (no production data at risk); revert page files and `UserNavbar`.

## Open Questions

- [ ] User-DB table/column names for student/course/group/payment aggregates must match the desktop app schema — verify exact names during apply; queries are individually guarded to degrade gracefully if a table is absent.
- [ ] Pre-existing `<form action="/api/logout">` in `Dashboard.astro`/`Admin.astro` targets a non-existent endpoint; navbar uses the `logout` action instead. Reconciling the layouts is out of scope for this change.
