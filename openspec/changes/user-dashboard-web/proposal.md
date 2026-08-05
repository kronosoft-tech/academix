# Proposal: User Dashboard Web

## Intent

The web app has auth, layout, and DB plumbing in place but no functional user-facing pages beyond a subscription summary. Users cannot see academy metrics, manage billing, or access an authenticated navigation menu. This change delivers the dashboard experience end-to-end.

## Scope

### In Scope
- Academy metrics page (read-only): student/course/group counts, income vs expenses, recent payments — sourced from user's individual Turso DB
- Control-plane migration: `subscriptions` + `subscription_payments` tables
- Subscription management page (`/dashboard/subscription`) with plan details + upgrade CTA
- Payments history page (`/dashboard/payments`)
- Auth-aware `UserNavbar`: login CTA when anonymous, avatar + dropdown menu when authenticated (Dashboard, Mi Academia, Suscripción, Descargar App, Cerrar sesión)
- `getFullTokenPayload(cookies)` helper to access JWT claims (dbUrl/dbToken) from pages
- Pricing plans definition: Básico $49,900 / Pro $89,900 / Premium $149,900 COP/month

### Out of Scope
- Payment gateway integration (Stripe/PayU/Wompi) — deferred
- Plan upgrade/downgrade mutation flow
- Admin-side subscription management
- AI assistant features referenced in plan tiers
- Mobile-responsive sidebar navigation overhaul

## Capabilities

### New Capabilities
- `user-dashboard-metrics`: Read-only academy metrics from user's individual Turso DB
- `user-subscription-management`: Subscription and billing pages (view-only)
- `authenticated-user-menu`: Auth-aware navbar with dropdown navigation

### Modified Capabilities
- None (existing dashboard page gets replaced/enhanced — no pre-existing spec)

## Approach

1. **Auth helper**: Create `getFullTokenPayload(cookies)` that verifies token and returns full `CustomerJwtPayload` including `dbUrl`/`dbToken`.
2. **Metrics page**: Replace current `/dashboard/index.astro` — call `getUserDb(payload)` to query student/course/group counts and payment aggregates.
3. **Control-plane migration**: Add SQL migration for `subscriptions` and `subscription_payments` tables on the `academix` (control-plane) DB.
4. **Subscription page**: New `/dashboard/subscription.astro` — shows current plan, status, period end, upgrade CTA.
5. **Payments page**: New `/dashboard/payments.astro` — paginated payment history table from control-plane.
6. **UserNavbar**: Convert from static Astro to a server island (or hybrid component) that checks auth state and renders login button or avatar+dropdown.
7. **Pricing constants**: Shared `src/data/plans.ts` defining the 3 tiers with limits and prices.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/lib/auth.ts` | Modified | Add `getFullTokenPayload` helper |
| `src/components/UserNavbar.astro` | Modified | Auth-aware with dropdown |
| `src/pages/dashboard/index.astro` | Modified | Academy metrics from user DB |
| `src/pages/dashboard/subscription.astro` | New | Subscription details page |
| `src/pages/dashboard/payments.astro` | New | Payment history page |
| `src/data/plans.ts` | New | Pricing plan definitions |
| Control-plane DB migration | New | `subscriptions` + `subscription_payments` tables |
| `src/components/DashboardCharts.tsx` | Modified | Wire up with real data |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| JWT contains DB credentials — XSS could leak them | Med | httpOnly cookie already in place; CSP headers; never expose in client JS |
| Control-plane migration on production DB | Low | Idempotent migration with IF NOT EXISTS; test in staging first |
| User DB queries on empty/new academies | Med | Handle zero-state gracefully with empty-state UI |
| Pricing hardcoded — no admin override | Low | Acceptable for MVP; plan management deferred to admin panel |

## Rollback Plan

1. Revert migration (DROP TABLE IF EXISTS for new tables — no existing data at risk)
2. Revert `UserNavbar` to static placeholder
3. Dashboard pages are new files — delete to rollback
4. `getFullTokenPayload` is additive — removal has no side effects

## Dependencies

- Control-plane DB migration must run before subscription/payments pages work
- `getUserDb` helper already exists — no new external deps

## Success Criteria

- [ ] `/dashboard` shows live academy metrics (students, courses, groups, income)
- [ ] `/dashboard/subscription` renders current plan and status
- [ ] `/dashboard/payments` shows payment history from control-plane
- [ ] `UserNavbar` shows dropdown menu when authenticated, login button when not
- [ ] Zero runtime errors on empty/new academy databases
- [ ] TypeScript passes (`bunx tsc --noEmit`)

## Proposal Question Round

These questions would improve the proposal by uncovering edge cases and business rules. In automatic mode, I'm documenting assumptions that the orchestrator should validate:

1. **Empty academy state**: When a user just registered and has no students/courses yet — should the metrics page show zeros with a CTA to "Download the desktop app to get started", or just zeros?
   - *Assumption*: Show zeros + onboarding CTA pointing to desktop download.

2. **Subscription source of truth**: The dashboard page already queries `subscriptions` from the control-plane DB. Is the subscription created at registration time (with trial status), or only after first payment?
   - *Assumption*: Created at registration with `status: 'trial'` and `trial_end` 14 days out.

3. **UserNavbar on public pages**: Should the dropdown appear on landing/pricing/FAQ pages too (replacing "Iniciar Sesión"), or only inside `/dashboard`?
   - *Assumption*: Auth-aware globally — appears on all pages including landing.

4. **Plan limits enforcement**: Are student/user limits enforced server-side now, or is this display-only for the web dashboard (enforcement lives in the desktop app)?
   - *Assumption*: Display-only on web; enforcement deferred to desktop app sync layer.

5. **Currency display**: Payments are stored with `currency` and `amount`. Should we always display in COP, or respect the stored currency?
   - *Assumption*: Display the stored currency (future-proof for international expansion).
