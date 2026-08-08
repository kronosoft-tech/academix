# Proposal: Payments and Subscriptions

## Intent

Enable Academix to monetize via multi-gateway subscriptions (Stripe, Wompi, Mercado Pago) with a 15-day no-card trial, 7-day grace period, daily email reminders, and login enforcement on the desktop app. The existing Stripe webhook and lifecycle code provides the foundation — this change completes the payment flow end-to-end.

## Scope

### In Scope
- Fix trial duration from 7 → 15 days in lifecycle.ts and Stripe Checkout config
- Wompi Widget Checkout integration for Colombia-native payments (PSE/Nequi/Bancolombia)
- Mercado Pago Subscriptions API integration for LatAm expansion
- Webhook endpoints for Wompi (`/api/webhooks/wompi`) and Mercado Pago (`/api/webhooks/mercadopago`)
- Daily cron for trial/grace email reminders (Vercel cron)
- Desktop app subscription check in `commands/auth.rs` post-login (query control plane)
- Pricing page update: link to gateway-specific checkout based on user country
- `subscription_status` field in login response for desktop UI consumption
- Expired trial cron: also handle expired trials (currently only handles grace)
- Web dashboard plan status display (days remaining, next billing)

### Out of Scope
- Admin panel for managing subscriptions manually
- Proration/plan upgrades mid-cycle (future iteration)
- Refund processing automation
- Multi-currency display on pricing page (COP-only for now, USD via Stripe later)
- Custom invoice/receipt PDF generation

## Capabilities

### New Capabilities
- `wompi-payments`: Wompi Widget Checkout integration with recurring via payment_source_id
- `mercadopago-payments`: Mercado Pago Subscriptions API for LatAm recurring billing
- `subscription-reminders`: Daily email reminders during trial countdown and grace period
- `desktop-subscription-guard`: Desktop login blocks expired/cancelled subscriptions

### Modified Capabilities
- `stripe-payments`: Update trial_period_days to 15; handle expired trials in cron
- `pricing-checkout`: Pricing page routes to correct gateway based on geo/country

## Approach

**Implementation order**: Stripe fixes first (trial 15d + expired trial handling) → Wompi widget → Mercado Pago → reminders cron → desktop enforcement.

- **Stripe**: Fix `createTrialSubscription` to use 15-day trial; add `getExpiredTrialSubscriptions` to cron
- **Wompi**: Widget Checkout Web for one-time + `payment_source_id` tokenization for recurring; webhook verifies signature via `integrity` hash
- **Mercado Pago**: `preapproval` endpoint for recurring subscriptions; webhook receives IPN notifications
- **Cron**: Expand `expire-subscriptions.ts` to also expire trials; new `send-reminders.ts` endpoint
- **Desktop**: After password verification in `login()`, query control plane `subscriptions` table; reject if status is `expired` or `cancelled`
- **Pricing**: `PricingByCountry.astro` detects country (Vercel geo headers) and renders checkout button pointing to correct gateway

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `web/src/lib/payments/lifecycle.ts` | Modified | 15-day trial, add expired trial query |
| `web/src/lib/payments/wompi.ts` | New | Wompi client, widget config, signature verification |
| `web/src/lib/payments/mercadopago.ts` | New | MP client, preapproval creation, IPN handling |
| `web/src/pages/api/webhooks/wompi.ts` | New | Wompi webhook endpoint |
| `web/src/pages/api/webhooks/mercadopago.ts` | New | Mercado Pago webhook endpoint |
| `web/src/pages/api/cron/send-reminders.ts` | New | Daily reminder emails cron |
| `web/src/pages/api/cron/expire-subscriptions.ts` | Modified | Also expire trials past 15 days |
| `web/src/components/PricingByCountry.astro` | Modified | Gateway-specific checkout buttons |
| `src-tauri/src/commands/auth.rs` | Modified | Subscription status check post-login |
| `web/src/pages/api/checkout/[gateway].ts` | New | Checkout session creation per gateway |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Wompi recurring (payment_source_id) lacks native subscription lifecycle | Med | Implement manual charge scheduling via cron; fallback to one-time charges |
| Webhook race conditions (duplicate events) | Med | Idempotency keys in `subscription_payments`; upsert on provider_payment_id |
| Desktop offline can't check subscription | Low | Cache last-known status with TTL; allow offline grace of 24h |
| Mercado Pago IPN delivery unreliable | Med | Implement polling fallback; reconciliation cron |
| Country detection inaccurate | Low | Allow user to override country/gateway at checkout |

## Rollback Plan

1. **Wompi/MP disabled**: Remove gateway endpoints; pricing page falls back to Stripe-only
2. **Trial 15→7 revert**: Single constant change in `lifecycle.ts`
3. **Desktop guard disabled**: Remove subscription check from `auth.rs` login; all logins pass
4. **Reminders disabled**: Remove cron endpoint; no emails sent (silent degradation)
5. Each gateway is independently deployable — no cross-gateway dependency

## Dependencies

- Stripe Dashboard: Products/Prices configured with 15-day trial
- Wompi merchant account and API credentials (sandbox available)
- Mercado Pago application credentials (test mode available)
- Email service (Resend/SendGrid) for reminder emails
- Vercel cron configuration for daily jobs

## Success Criteria

- [ ] Stripe checkout creates subscription with 15-day trial (no card upfront)
- [ ] Wompi widget completes payment for Colombian users
- [ ] Mercado Pago creates recurring subscription for LatAm users
- [ ] Webhook handlers update subscription status correctly for all 3 gateways
- [ ] Daily reminders sent during trial (days remaining) and grace (payment failed)
- [ ] Desktop login blocks access when subscription is expired/cancelled
- [ ] Expired trials transition to `expired` status after 15 days
- [ ] Pricing page shows correct checkout flow based on detected country

## Proposal Question Round

The following questions would improve this proposal before finalizing specs. Since this is automatic mode, these assumptions need user validation:

1. **Email service choice**: Which email provider for reminders? Resend is lightweight and developer-friendly; SendGrid has more templates. Assumption: Resend (already common in Vercel stacks).

2. **Wompi recurring model**: Wompi doesn't have native subscription management like Stripe. Should we (a) charge manually via cron using tokenized payment_source_id, or (b) require Colombian users to re-authorize monthly? Assumption: (a) automatic monthly charge via stored token.

3. **Desktop offline grace**: When the desktop app can't reach the control plane to verify subscription, should login be blocked or allowed with a 24h cached grace? Assumption: 24h offline grace with cached status.

4. **Trial start trigger**: Does the 15-day trial start at (a) user registration on the web, or (b) first desktop app login? Assumption: (a) registration creates the subscription record immediately.

5. **Country gateway routing**: Should the pricing page auto-detect country via Vercel geo headers and pre-select the gateway, or show all options and let the user choose? Assumption: auto-detect with option to switch manually.
