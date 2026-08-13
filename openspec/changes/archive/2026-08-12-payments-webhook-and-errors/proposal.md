# Proposal: Secure Payment Activation and Webhook Error Handling

> Retrospective — implemented in commit `42c2d86` (review-approved).

## Intent

Production-blocking bugs in `web/`:

- Wompi "paid but nothing saved": fetches lacked `Authorization` (401 → silent no-op); webhook checksum ignored `signature.properties`/`amount_in_cents` → every event 401.
- MP 502 checkout: preference failure hid MP `detail`; dashboard handled dead `preapproval_id` not Checkout Pro `payment_id`.
- Security flaws: no Wompi ownership proof; MP replayable; MP webhook 200 on failure (no retry); desktop-first users had no trial row → silent no-op.

## Scope

### In Scope (all `web/`)
- Wompi ownership guard (verify-wompi + dashboard SSR, 400 on mismatch); Bearer-auth fetches; widget calls verify-wompi.
- Wompi checksum per `event.signature.properties`; webhook sets `plan_id`.
- MP webhook `x-signature` HMAC (timing-safe, `MP_WEBHOOK_SECRET`); 401/400/500 semantics (MP retries on 500).
- `activateApprovedPayment`: replay guard, activation via `external_reference`, idempotent payment record; dashboard MP redirect (`payment_id`/`collection_id`).
- `getOrCreateTrialSubscription` (lazy trial row) in all activation paths.
- `lifecycle.ts` schema alignment; checkout surfaces `detail`; tests + AGENTS.md.

### Out of Scope
- Desktop app; Stripe; MP preapproval API (unsupported in CO).
- `charge-wompi.ts` cron (`s.plan` + stale prices) — follow-up.
- `.env.example`, webhook URL registration, tunnel docs.

## Approach

Fix-in-place with safety hardening: centralize activation, verify both webhook signatures, guard ownership/replay, log errors with correct statuses instead of silent no-ops.

## Affected Areas

| Area | Impact | Change |
|------|--------|--------|
| `verify-wompi.ts`, `dashboard/index.astro` | Modified | Wompi auth + ownership; MP Checkout Pro path; logging |
| `wompi.ts`, `webhooks/wompi.ts` | Modified | Checksum per `signature.properties`; `plan_id`; lazy row |
| `mercadopago.ts`, `webhooks/mercadopago.ts` | Modified | Signature verify; replay guard; 500 retry |
| `checkout/mercadopago.ts`, `CheckoutPlans.tsx` | Modified | Lazy row; `detail` surfaced; verify-wompi call |
| `lifecycle.ts` | Modified | Schema alignment; lazy trial row |
| `test/payments*.test.ts`, `AGENTS.md` | Modified | Schema columns; gateway docs |

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `wompi-payments`: ownership guard, Bearer auth, checksum per `signature.properties`, `plan_id`.
- `mercadopago-payments`: Checkout Pro redirect, `x-signature` verification, replay guard, 500-retry.
- `subscription-lifecycle`: `plan_id` alignment, lazy `getOrCreateTrialSubscription`.

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `MP_WEBHOOK_SECRET` unset → webhook 500s | Med | Explicit 500 + logs; key documented |
| Retrospective drift vs committed code | Low | Grounded in `42c2d86` diff |
| `charge-wompi` cron still broken | High | Out of scope; follow-up |

## Rollback Plan

Revert commit `42c2d86` — no migrations changed, only queries/handlers; legacy DBs unaffected.

## Dependencies

- New env: `MP_WEBHOOK_SECRET` (MP dashboard secret).
- Existing: `WOMPI_PUBLIC_KEY`, `WOMPI_EVENTS_SECRET`, `SITE_URL`; webhook URLs set in dashboards.

## Success Criteria

- [ ] Wompi payment activates immediately; ownership mismatch → 400.
- [ ] Wompi webhook accepts real events; sets `plan_id`.
- [ ] MP webhook rejects unverified POSTs (401); 500 when processing fails.
- [ ] Replayed MP `payment_id` never double-records.
- [ ] Desktop-first user completes both gateway flows.
- [ ] `tsc --noEmit` + `bun run test` pass in `web/`.
