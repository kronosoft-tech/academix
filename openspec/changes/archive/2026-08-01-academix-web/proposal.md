# Proposal: Academix Public Website

## Intent

Academix has no public-facing web presence. Potential customers cannot discover, evaluate, purchase, or download the product online. This blocks growth in the LATAM educational market. The website fills that gap: marketing, downloads, authentication (shared with the desktop app), customer dashboards, payments, and support.

## Scope

### In Scope
- Landing/marketing pages (hero, features, pricing, FAQ, testimonials, contact)
- Download portal with OS detection and GitHub Release links
- Tutorial content (download + usage) via Astro content collections
- Custom authentication system (JWT + bcrypt + httpOnly cookies, shared Turso DB)
- Internal admin dashboard (clients, revenue, downloads, payments)
- Customer dashboard (subscription, invoices, plan management)
- Payment integration (Stripe + MercadoPago + PayU) via webhooks
- PQRS support portal (form-based)
- AI-powered technical assistance chat
- Localized pricing by country (CO, MX, CL, AR) via server islands
- Monorepo setup: `/web` directory with Astro 7.1

### Out of Scope
- Mobile app
- Multi-language support (Spanish only for MVP)
- Real-time collaboration features
- Third-party LMS integration
- Changes to the existing Tauri desktop app
- Pro/Premium plan activation (MVP = Basic plan only)

## Capabilities

### New Capabilities
- `web-landing`: Marketing pages, features showcase, FAQ, testimonials, contact form
- `web-downloads`: OS detection, architecture selector, GitHub Release links, tutorials
- `web-auth`: Custom JWT auth shared with desktop app (registration, login, logout, password reset)
- `web-admin-dashboard`: Internal metrics (clients, revenue, downloads, payments)
- `web-customer-dashboard`: Subscription management, invoices, plan changes
- `web-payments`: Stripe + MercadoPago + PayU webhook integration, subscription lifecycle
- `web-support`: PQRS portal + AI technical chat
- `web-pricing`: Server island with country detection and localized currency display

### Modified Capabilities
- None (the desktop app remains unchanged; shared Turso DB already exists)

## Approach

- **Monorepo addition**: `/web` directory at project root, independent Astro 7.1 project
- **Rendering strategy**: Static for marketing/landing (prerendered), SSR for auth/dashboards/webhooks, Server Islands for pricing/session indicators, Client Islands for charts/chat
- **Auth**: Custom middleware validates JWT from httpOnly cookies; bcrypt hashes compatible between Rust (desktop) and TypeScript (web) — both use standard bcrypt format
- **Database**: Same Turso instance as desktop app — shared `users` table
- **Payments**: Webhook-only pattern — each provider hits a dedicated endpoint, handler normalizes events into a common subscription lifecycle
- **Deploy**: Vercel with `@astrojs/vercel` adapter for SSR support
- **Content**: Astro content collections for tutorials and FAQ

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `/web` | New | Entire Astro project (marketing, auth, dashboards, payments, support) |
| `package.json` (root) | Modified | Workspace configuration for monorepo |
| Turso DB schema | Modified | New tables: subscriptions, payments, invoices, pqrs_tickets, password_resets |
| `src-tauri/migrations/` | Modified | New migrations for shared auth and subscription tables |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Scope exceeds single delivery | High | Phase into 3 PRs: Landing+Downloads → Auth+Dashboards → Payments+Support |
| Multi-gateway payment complexity | Med | Start with Stripe only, add MercadoPago/PayU in later iterations |
| bcrypt cross-platform compatibility | Low | Both use standard $2b$ format; validate in integration tests |
| Exchange rate staleness | Med | Use a simple JSON config updated manually; automate later |
| AI chat quality/cost | Med | Start with a simple FAQ-based bot; upgrade to LLM later |

## Rollback Plan

- `/web` directory is isolated — remove it and monorepo config to fully revert
- Database migrations are additive (new tables only) — can be dropped without affecting desktop app
- Vercel deployment is independent — delete the project to remove web presence
- DNS/domain changes are the only external dependency requiring coordination

## Dependencies

- Turso account with production database access
- Vercel account for hosting
- Stripe account + MercadoPago/PayU merchant accounts
- Domain name for the website
- GitHub Releases set up for desktop app builds (already exists)

## Success Criteria

- [ ] Landing page is live and accessible from target countries
- [ ] Users can download the correct installer for their OS without confusion
- [ ] Registration/login works and the same credentials work in the desktop app
- [ ] Basic plan subscription can be purchased via at least one payment provider
- [ ] Admin dashboard shows real-time client and revenue metrics
- [ ] Customer can view their subscription status and payment history

---

## Proposal Question Round

The following questions would sharpen this proposal before proceeding to specs. Review and answer what's relevant:

1. **Business rules for subscription lifecycle**: What happens when a payment fails? Grace period before deactivation? How many days overdue before the desktop app loses access? Is there a trial period?

2. **Auth edge cases — desktop/web conflict**: If a user registers on web but already has a local-only SQLite account in the desktop app (before Turso sync), how should account merging work? Or is every user expected to register on web first?

3. **MVP scope boundary for AI chat**: Should the AI chat in MVP be a simple FAQ search/matching system, or do you want an actual LLM integration (OpenAI/Anthropic API) from day one? This significantly impacts cost and complexity.

4. **Download metrics tracking**: How should download counts be tracked? GitHub API (limited), custom redirect endpoint that counts before forwarding to GitHub, or something else?

5. **PQRS workflow**: Who handles submitted tickets? Is there an internal assignment system needed, or do tickets just go to a shared email/queue? What are the expected SLA commitments?

### Assumptions (pending confirmation)
- MVP launches with Stripe only; MercadoPago and PayU are added in a follow-up phase
- Exchange rates are manually configured (static JSON), not fetched from an API
- AI chat starts as FAQ-based keyword matching, not LLM-powered
- No trial period for MVP — payment required before desktop activation
- Desktop app already uses Turso sync (users table is already remote)
´