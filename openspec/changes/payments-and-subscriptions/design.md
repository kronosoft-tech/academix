# Design: Payments and Subscriptions

## Technical Approach

`lifecycle.ts` remains the single gateway-agnostic core writing subscription state to the **control plane DB** (`TURSO_URL` web / `CONTROL_PLANE_DB_URL` desktop — same DB). Each gateway (Stripe/Wompi/MP) is an isolated module that translates its own events into `lifecycle` calls. No forced common interface — only a shared type contract. Trials are created at registration (no card); checkout converts to paid; a cron drives expiry, reminders, and Wompi recurring charges; desktop login reads status from the control plane with a 24h offline cache.

## Architecture Decisions

| # | Decision | Choice | Rejected | Rationale |
|---|----------|--------|----------|-----------|
| 1 | Gateway abstraction | Separate modules + shared types; `lifecycle.ts` is the core | Forced `PaymentGateway` interface | Stripe (native subs), Wompi (manual charge), MP (preapproval) differ too much; a common interface leaks. Core stays single source of truth. |
| 2 | Webhooks | Separate `webhooks/{gateway}.ts` | Dynamic `[gateway].ts` | Matches existing `webhooks/stripe.ts`; per-gateway signature + raw-body handling; independently deployable. |
| 3 | Checkout | Separate `checkout/{gateway}.ts` endpoints | Dynamic route / actions | Consistency with webhooks; distinct request/response per gateway. |
| 4 | Cron | 3 single-purpose endpoints (expire, reminders, charge-wompi) | One mega-cron | Clear responsibility, independent failure. Risk: Vercel plan cron limits (see below). |
| 5 | Desktop cache | JSON file in Tauri `app_data_dir` (`{status, checked_at}`) | Local SQLite / in-memory | One tiny record; no migration; survives restart; 24h grace = `now - checked_at < 24h`. |
| 6 | Wompi recurring | Store `payment_source_token`; monthly `charge-wompi` cron POSTs `/transactions` | Re-auth each month | Wompi has no native subs; tokenized card enables auto-charge. Success→activate, fail→grace. |
| 7 | Trial creation | In `register` action, 15d, `provider=NULL` | Post-register webhook | Spec: no-card trial starts at registration. |
| 8 | Pricing UX | Astro SSR default (geo header) + `CheckoutPlans.tsx` React island | Pure Astro | Gateway override + checkout POST need client interactivity; React/MUI available. |
| 9 | MP SDK | REST via `fetch` | `mercadopago` npm SDK | SDK not installed, heavy for serverless; preapproval is a few REST calls. |
| 10 | Schema | Additive migration for generic provider fields | Stripe-only columns | Wompi/MP have no `stripe_subscription_id`; need generic linkage. |

## Data Flow

```
Register ──→ createTrialSubscription(15d, no card) ──→ control plane
Pricing island ──→ /api/checkout/{gw} ──→ gateway ──→ redirect/widget
Gateway event ──→ /api/webhooks/{gw} ──(verify sig)──→ lifecycle ──→ control plane
Vercel cron ──→ expire | send-reminders | charge-wompi ──→ lifecycle + email
Desktop login ──→ control_plane.get_subscription_status ──→ cache.json (24h grace)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `web/src/lib/payments/gateway.ts` | Create | Shared types (`NormalizedWebhookResult`, `CheckoutResult`), geo→gateway map |
| `web/src/lib/payments/lifecycle.ts` | Modify | 15d trial, optional/generic provider fields, `getExpiredTrials`, `findByProviderSubId` |
| `web/src/lib/payments/wompi.ts` | Create | Client, tokenize/charge, SHA256 signature verify |
| `web/src/lib/payments/mercadopago.ts` | Create | Preapproval REST client + IPN verify |
| `web/src/lib/payments/email.ts` | Create | nodemailer (Gmail) transport + reminder/grace templates |
| `web/src/pages/api/checkout/{stripe,wompi,mercadopago}.ts` | Create | Per-gateway checkout session creation |
| `web/src/pages/api/webhooks/stripe.ts` | Modify | Fix hardcoded plan/currency; idempotency on `provider_payment_id` |
| `web/src/pages/api/webhooks/{wompi,mercadopago}.ts` | Create | Signature-verified event handlers |
| `web/src/pages/api/cron/expire-subscriptions.ts` | Modify | Also expire ended trials |
| `web/src/pages/api/cron/{send-reminders,charge-wompi}.ts` | Create | Daily reminders; monthly Wompi charge |
| `web/src/actions/register.ts` | Modify | Create trial subscription on signup |
| `web/src/actions/create-checkout.ts` | Modify | Remove `trial_period_days` (trial pre-consumed); route via `checkout/stripe.ts` |
| `web/src/components/PricingByCountry.astro` | Modify | Mount `CheckoutPlans` island |
| `web/src/components/CheckoutPlans.tsx` | Create | React island: gateway override + checkout POST |
| `web/migrations/002_multi_gateway.sql` | Create | Add `provider`, `provider_subscription_id`, `provider_customer_id`, `payment_source_token`; rebuild `subscription_payments` to allow `wompi` |
| `web/vercel.json` | Modify | Add `send-reminders` + `charge-wompi` crons |
| `src-tauri/src/infrastructure/turso/control_plane.rs` | Modify | Add `get_subscription_status(user_id)` |
| `src-tauri/src/infrastructure/subscription_cache.rs` | Create | JSON file cache read/write |
| `src-tauri/src/commands/auth.rs` | Modify | Post-login status check + 24h offline cache |
| `src-tauri/src/lib.rs` | Modify | Register `subscription_cache` module |

## Interfaces / Contracts

```typescript
// gateway.ts
export type Gateway = 'stripe' | 'wompi' | 'mercadopago';
export interface NormalizedWebhookResult {
  action: 'activate' | 'grace' | 'cancel' | 'ignore';
  providerSubId: string;
  userId?: string;
  amount?: number; currency?: string; providerPaymentId?: string;
}
export interface CheckoutResult { url?: string; widgetToken?: string; }
```

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit | Signature verify (Wompi SHA256, Stripe, MP), trial 15d math, geo→gateway map | Vitest, mocked env |
| Unit | Rust cache 24h grace logic, `get_subscription_status` mapping | `cargo test` |
| Integration | Webhook→lifecycle status transitions, idempotency on duplicate `provider_payment_id` | Vitest + in-memory libsql |
| E2E | Register→trial visible; pricing gateway switch renders correct checkout | Playwright |

## Security

No shell/subprocess/VCS/executable-classification boundary. Security controls that are requirements (must reach tasks + RED tests): (a) every webhook verifies provider signature before mutating state; (b) all cron endpoints require `Bearer CRON_SECRET`; (c) idempotency keyed on `provider_payment_id` to survive webhook retries/races; (d) never log payment tokens or `db_token`.

## Threat Matrix

N/A — no routing dispatch on untrusted input, shell, subprocess, VCS/PR automation, executable-file classification, or process integration. Webhook/cron security captured in Security section above.

## Migration / Rollout

Additive migration only (`ALTER TABLE ADD COLUMN`, idempotent per existing "duplicate column" handling). `subscription_payments` provider set widened by table rebuild (`payu`→`wompi`). Each gateway independently removable; trial length is a one-line constant; desktop guard removable with no side effects.

## Open Questions

- [ ] Vercel plan cron limits: Hobby caps crons (may force consolidating the 3 endpoints). Confirm plan tier.
- [ ] `subscription_payments` rebuild: drop `payu` entirely or keep for history?
- [ ] Two divergent schemas exist (`web/migrations/001` uses `plan_id`; `src-tauri/020` uses `plan`). Confirm `020` is authoritative for the control plane before migrating.
