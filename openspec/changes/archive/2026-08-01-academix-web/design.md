# Design: Academix Web

## Technical Approach

Standalone Astro 7.1 project in `/web`, deployed to Vercel via `@astrojs/vercel` (hybrid rendering). Marketing/content pages prerender; auth, dashboards, webhooks, and APIs run SSR. Data access uses `@libsql/client` against the SAME Turso instance the desktop app already syncs to — no ORM, parameterized SQL only. Auth is custom JWT (`jose`) + `bcryptjs` in httpOnly cookies, validated in `src/middleware.ts`. Two independent identity domains: academy staff (existing `users` table = the web CUSTOMERS) and the internal Academix team (new `web_admins` table). Realizes proposal capabilities `web-*` and the spec's 8 requirement groups.

## Architecture Decisions

| Decision | Choice | Alternatives rejected | Rationale |
|----------|--------|-----------------------|-----------|
| Role model | Keep `users` AS-IS; new `web_admins` for internal team | Rebuild users; single table | Academy staff (Admin/Gerente/Empleado/Profesor) ARE the customers; internal team has different roles/access. Zero desktop migration risk |
| Identity discrimination | JWT `type` field: `customer` vs `admin` | Separate cookie names; role prefixing | One verify path; middleware routes by `type`. Cross-domain access is a hard 403 |
| Subscription payments table | Name it `subscription_payments` | Reuse `payments` | A `payments` table ALREADY exists for student/group fees — reuse would collide. Distinct table avoids breaking desktop |
| Session store | Stateless JWT only | Reuse existing `sessions` table | Desktop `sessions` is opaque-token based; web is serverless/edge — stateless JWT avoids per-request DB hits |
| Password hashing | `bcryptjs`, cost 12 | argon2; match desktop cost 10 | bcrypt embeds cost in `$2b$` hash → verify is cost-agnostic, so web(12)↔desktop(10) interop works. 12 is stronger for web-exposed surface |
| DB layer | `@libsql/client` direct | Drizzle/Prisma | Schema owned by Rust migrations; TS ORM would fight ownership. Keep it thin |
| AI providers | Interface + round-robin + fallback, SSE stream | Single provider; queue | Spec requires extensible rotation + graceful degradation |
| Pricing | Server island + static `pricing.json` + geo header | Client fetch; FX API | Spec requires SSR country detection, manual rates |

## Data Flow

Two login flows, one middleware:

    /auth/login  ─> users table       ─> JWT{type:'customer'} ─> /dashboard/*
    /admin/login ─> web_admins table  ─> JWT{type:'admin'}    ─> /admin/*

    Browser ─cookie(JWT)─> middleware.ts ─verify(jose)─> check `type`
        type='customer' ─> locals.user  ─> allow /dashboard/*
        type='admin'    ─> locals.admin ─> allow /admin/*
        cross-access ─> 403   |   unauth on protected ─> redirect login

Payments:

    /pricing ─> action ─> Stripe Checkout ─> Stripe ─webhook─> /api/webhooks/stripe
                                                  verify sig ─> normalize ─> subscriptions + subscription_payments

AI chat: client island ─POST─> /api/chat ─> ProviderRotator.chat() ─SSE─> client (fallback to next provider on error).

## Rendering Strategy

| Route | Rendering | Reason |
|-------|-----------|--------|
| `/`, `/features`, `/faq`, `/contact` | Static | Marketing, rarely changes |
| `/pricing` | Static shell + Server Island | PricingByCountry per geo |
| `/downloads`, `/tutorials/*` | Static | Content collections |
| `/auth/*` | SSR | Server processing |
| `/dashboard/*` | SSR | Protected, customer data |
| `/admin/*` | SSR | Protected, internal metrics |
| `/api/*` | SSR | Webhooks, AI, downloads |

## Islands

| Component | Type | Used In |
|-----------|------|---------|
| PricingByCountry | Server Island | /pricing |
| UserNavbar | Server Island | All pages |
| DashboardCharts | Client Island (React/Recharts) | /dashboard/*, /admin/* |
| AIChat | Client Island (React) | /dashboard/support |
| DownloadSelector | Client Island (React) | /downloads |

## File Changes

| Path | Action | Description |
|------|--------|-------------|
| `web/package.json`, `astro.config.mjs`, `tsconfig.json` | Create | Astro 7.1 + Vercel adapter, Tailwind v4 vite plugin, Zod |
| `web/src/pages/` | Create | Routes: `index`, `features`, `faq`, `pricing`, `contact`, `downloads`, `tutorials/[...]`, `auth/*`, `dashboard/*`, `admin/*`, `api/*` |
| `web/src/middleware.ts` | Create | JWT validation, `type` gate, populates `locals.user` / `locals.admin` |
| `web/src/layouts/` | Create | Base, Dashboard, Admin layouts |
| `web/src/components/` | Create | Astro components + React islands (DashboardCharts, AIChat, DownloadSelector, PricingByCountry, UserNavbar) |
| `web/src/content/` | Create | Collections: `tutorials`, `faq` |
| `web/src/actions/` | Create | Astro actions: contact, register, login, admin-login, logout, password-reset, pqrs, cancel-subscription |
| `web/src/lib/` | Create | `auth.ts` (jwt/bcrypt), `db.ts` (libsql client), `payments/stripe.ts`, `ai/` (rotator + providers), `email.ts` (Resend), `rate-limit.ts` |
| `web/src/data/` | Create | `pricing.json`, `countries.json` |
| `web/src/pages/api/webhooks/stripe.ts` | Create | Signature verify + lifecycle normalize |
| `web/src/pages/api/download.ts` | Create | Increment `downloads` counter, redirect to allow-listed GitHub asset |
| `web/src/pages/api/chat.ts` | Create | SSE endpoint over provider rotator |
| `src-tauri/migrations/020_web_schema.sql` | Create | New tables: web_admins, subscriptions, subscription_payments, invoices, pqrs_tickets, pqrs_responses, password_resets, downloads, ai_conversations |

## Interfaces / Contracts

```sql
-- 020_web_schema.sql — additive only, NO changes to existing tables
CREATE TABLE IF NOT EXISTS web_admins (id TEXT PRIMARY KEY, email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL, name TEXT NOT NULL,
  role TEXT NOT NULL CHECK(role IN ('superadmin', 'employee', 'manager')),
  is_active INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS idx_web_admins_email ON web_admins(email);
CREATE TABLE IF NOT EXISTS subscriptions (id TEXT PRIMARY KEY, user_id TEXT NOT NULL,
  plan TEXT NOT NULL, status TEXT NOT NULL, trial_start TEXT, trial_end TEXT,
  grace_start TEXT, grace_end TEXT, stripe_subscription_id TEXT,
  current_period_start TEXT, current_period_end TEXT,
  FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE);
CREATE TABLE IF NOT EXISTS subscription_payments (id TEXT PRIMARY KEY, user_id TEXT NOT NULL,
  subscription_id TEXT, amount REAL NOT NULL, currency TEXT NOT NULL, status TEXT NOT NULL,
  provider TEXT NOT NULL, provider_payment_id TEXT, created_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS invoices (id TEXT PRIMARY KEY, user_id TEXT NOT NULL,
  payment_id TEXT, number TEXT NOT NULL, pdf_url TEXT, created_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS pqrs_tickets (id TEXT PRIMARY KEY, user_id TEXT NOT NULL,
  type TEXT NOT NULL, subject TEXT NOT NULL, description TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'open', assigned_to TEXT,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL, resolved_at TEXT);
CREATE TABLE IF NOT EXISTS pqrs_responses (id TEXT PRIMARY KEY, ticket_id TEXT NOT NULL,
  author_id TEXT NOT NULL, message TEXT NOT NULL, created_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS password_resets (id TEXT PRIMARY KEY, user_id TEXT NOT NULL,
  token TEXT NOT NULL UNIQUE, expires_at TEXT NOT NULL, used_at TEXT);
CREATE TABLE IF NOT EXISTS downloads (id TEXT PRIMARY KEY, os TEXT, arch TEXT,
  version TEXT, ip TEXT, country TEXT, created_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS ai_conversations (id TEXT PRIMARY KEY, user_id TEXT NOT NULL,
  provider TEXT, model TEXT, messages_json TEXT, created_at TEXT NOT NULL);
```

```ts
interface AIProvider { name: string; chat(messages: ChatMessage[]): AsyncGenerator<string> }
interface CustomerJwtPayload { sub: string; email: string; role: string; // Admin|Gerente|Empleado|Profesor
  type: 'customer'; iat: number; exp: number } // exp = 7d, sub = users.id
interface AdminJwtPayload { sub: string; email: string; role: string; // superadmin|employee|manager
  type: 'admin'; iat: number; exp: number } // exp = 7d, sub = web_admins.id
// cookie: httpOnly, secure, sameSite=lax, path=/
```

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit | JWT sign/verify + `type` discrimination, bcrypt cross-cost verify, round-robin + fallback, Zod schemas, pricing lookup+USD fallback | Vitest |
| Integration | Stripe webhook signature (valid/invalid→401), lifecycle transitions (trial/grace/expire), download counter+redirect allow-list, migration 020 idempotency | Vitest + libsql in-memory / test DB |
| E2E | customer register→login→dashboard, admin login→admin dashboard, protected-route redirect, cross-domain 403, PQRS create/list | Playwright |

## Threat Matrix

No git/shell/subprocess/VCS/PR/executable-classification boundary → matrix rows **N/A**. Three HTTP-routing security boundaries handled explicitly: (1) **Open-redirect** on `/api/download` — target MUST be validated against a GitHub-release allow-list, never reflected from user input; (2) **Webhook forgery** on `/api/webhooks/stripe` — reject non-verifying signatures with 401 and log; (3) **Privilege crossing** — a `customer` JWT hitting `/admin/*` (or vice versa) MUST 403, never silently render. All three carry RED tests into tasks.

## Migration / Rollout

Single additive migration `020_web_schema.sql`. NO changes to existing tables. All `CREATE TABLE IF NOT EXISTS` for idempotency (project convention). Rollback: drop new tables; desktop unaffected. Deliver in 3 PRs per proposal: Phase 1 (Landing+Downloads) → Phase 2 (Auth+Dashboards) → Phase 3 (Payments+Support).

## Open Questions

- [ ] Rate-limit store on serverless: in-memory is per-instance; may need Vercel KV/edge for effectiveness.
- [ ] `web_admins` seeding: how is the first `superadmin` created (env-seeded like desktop admin, or manual insert)?
