# Tasks: Payments and Subscriptions

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 800–1000 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 → PR 2 → PR 3 → PR 4 → PR 5 → PR 6 |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Schema + shared types + lifecycle fixes + trial at register + Stripe checkout/webhook fix | PR 1 | `bun run test -- lifecycle` | `bun run dev` → register → verify trial row in DB | Revert migration + lifecycle.ts + register.ts |
| 2 | Wompi gateway (client, checkout, webhook, charge cron) | PR 2 | `bun run test -- wompi` | N/A — requires Wompi sandbox credentials | Remove wompi.ts + checkout/wompi.ts + webhooks/wompi.ts + cron/charge-wompi.ts |
| 3 | Mercado Pago gateway (client, checkout, webhook) | PR 3 | `bun run test -- mercadopago` | N/A — requires MP sandbox credentials | Remove mercadopago.ts + checkout/mercadopago.ts + webhooks/mercadopago.ts |
| 4 | Email reminders + expire-subscriptions fix | PR 4 | `bun run test -- email reminders` | N/A — requires GMAIL_APP_PASSWORD env | Remove email.ts + cron/send-reminders.ts |
| 5 | Desktop subscription guard (Rust) | PR 5 | `cargo test -p academix-desktop -- subscription` | `bun run tauri dev` → login → verify status check | Revert control_plane.rs + subscription_cache.rs + auth.rs changes |
| 6 | Pricing page UX (CheckoutPlans island) | PR 6 | `bun run dev` → visual check pricing page | `bun run dev` → navigate /pricing → select gateway | Remove CheckoutPlans.tsx + revert PricingByCountry.astro |

## Phase 1: Schema & Shared Types (PR 1 foundation)

- [ ] 1.1 Create `web/migrations/002_multi_gateway.sql` — ADD COLUMN provider, provider_sub_id, provider_payment_id, payment_source_token to subscriptions; rebuild subscription_payments for multi-gateway
- [ ] 1.2 Create `web/src/lib/payments/gateway.ts` — shared types (Gateway, NormalizedWebhookResult, CheckoutResult) + geoToGateway map

## Phase 2: Lifecycle & Stripe Fixes (PR 1 core)

- [ ] 2.1 Modify `web/src/lib/payments/lifecycle.ts` — add createTrialSubscription(15d, provider=null), getExpiredTrials, findByProviderSubId; add generic provider fields support
- [ ] 2.2 Modify `web/src/actions/register.ts` — call createTrialSubscription on signup
- [ ] 2.3 Modify `web/src/pages/api/webhooks/stripe.ts` — fix hardcoded plan/currency, add idempotency on provider_payment_id
- [ ] 2.4 Create `web/src/pages/api/checkout/stripe.ts` — Stripe checkout session with trial_period_days=15
- [ ] 2.5 Modify `web/src/pages/api/cron/expire-subscriptions.ts` — also expire trials past 15d

## Phase 3: Wompi Integration (PR 2)

- [ ] 3.1 Create `web/src/lib/payments/wompi.ts` — client (tokenize, verifySignature, createTransaction)
- [ ] 3.2 Create `web/src/pages/api/checkout/wompi.ts` — tokenize card + store payment_source_token
- [ ] 3.3 Create `web/src/pages/api/webhooks/wompi.ts` — verify sig → normalize → lifecycle call
- [ ] 3.4 Create `web/src/pages/api/cron/charge-wompi.ts` — monthly POST /transactions with stored token; success→activate, fail→grace

## Phase 4: Mercado Pago Integration (PR 3)

- [ ] 4.1 Create `web/src/lib/payments/mercadopago.ts` — REST client (createPreapproval, verifyIPN, getPaymentStatus)
- [ ] 4.2 Create `web/src/pages/api/checkout/mercadopago.ts` — create preapproval → redirect URL
- [ ] 4.3 Create `web/src/pages/api/webhooks/mercadopago.ts` — verify IPN → normalize → lifecycle call

## Phase 5: Reminders & Email (PR 4)

- [ ] 5.1 Create `web/src/lib/payments/email.ts` — nodemailer transport (GMAIL_USER/GMAIL_APP_PASSWORD) + templates (trial-countdown, grace-warning, payment-success)
- [ ] 5.2 Create `web/src/pages/api/cron/send-reminders.ts` — query trials/grace subs, send appropriate email; require CRON_SECRET Bearer auth
- [ ] 5.3 Modify `web/vercel.json` — add cron entries for send-reminders + charge-wompi

## Phase 6: Desktop Subscription Guard (PR 5)

- [ ] 6.1 Create `src-tauri/src/infrastructure/subscription_cache.rs` — read/write JSON {status, checked_at} to app_data_dir; 24h grace logic
- [ ] 6.2 Modify `src-tauri/src/infrastructure/turso/control_plane.rs` — add get_subscription_status(user_id) → (status, plan)
- [ ] 6.3 Modify `src-tauri/src/commands/auth.rs` — post-login: call control_plane → cache → block if expired/cancelled
- [ ] 6.4 Modify `src-tauri/src/lib.rs` — register subscription_cache module

## Phase 7: Pricing Page UX (PR 6)

- [ ] 7.1 Create `web/src/components/CheckoutPlans.tsx` — React island: plan cards, gateway selector (auto-detect + override), checkout POST to /api/checkout/{gw}
- [ ] 7.2 Modify `web/src/components/PricingByCountry.astro` — mount CheckoutPlans island with client:load
