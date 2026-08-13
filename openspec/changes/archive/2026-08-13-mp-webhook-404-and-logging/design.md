# Design: MP Webhook 404 & Resilience Fix

## Technical Approach

`getPayment` throws a plain `Error` with MP's HTTP status buried in the message string only (mercadopago.ts L185-188); the webhook handler catch (POST L84-93, GET L107-115) returns 500 for every failure. MP treats a 500 as a broken endpoint and abandons retries, so affected payments never persist. Fix: enrich `getPayment`'s thrown error with `.status` (mirroring the existing `createPreference` pattern, L154-159), then classify by `err.status` at the handler-level catch. Remove `processPayment`'s inner try/catch (L126-134) so the enriched error bubbles with a single log line. Maps to proposal Approach A; realizes the `mp-webhook-resilience` spec classification matrix.

## Architecture Decisions

| Decision | Option (Tradeoff) | Chosen |
|---|---|---|
| Error shape | Property-on-Error (tiny diff, reuses `createPreference` L156) **vs** `MpApiError extends Error` class (more code) | Property-on-Error + named `MpApiError` interface |
| Classification locus | In `processPayment` (two catch sites: POST+GET) **vs** at handler catch (one site, single log) | Handler catch |
| Retry-After source (429) | Hardcode `'30'` **vs** forward MP `retry-after` header (correct) | Forward MP header, fallback `'30'` |
| Double-log fix | Remove inner try/catch (changes call site; handler owns log) **vs** keep inner catch silent | Remove inner try/catch |

**D1 — Error shape**: `getPayment` casts to `MpApiError` (interface, not class) and sets `err.status = response.status`, `err.detail = errorText`, `err.retryAfter = response.headers.get('retry-after') ?? undefined`. Mirrors `createPreference` L154-159; the handler reads `.status` only.

**D2 — Classification matrix** (from spec `mp-webhook-resilience`, authoritative):
| `err.status` | HTTP response | Log | Rationale |
|---|---|---|---|
| 404 | 200 `{received:true,warning:"payment not found"}` | warn | transient/unknown; MP stops retrying |
| 401/403 | 200 `{received:true}` | error, tag `MP_API_AUTH_ERROR` | operator-actionable token config failure |
| 429 | 503 + `Retry-After` | warn | we're rate-limited; MP backs off |
| 5xx | 503 | error | MP server error; transient, MP retries |
| none (DB/TURSO) | 500 | error | truly critical |

**D3 — Single log point**: inner try/catch (L126-134) removed; `processPayment` no longer catches/rethrows. Handler catch owns logging, so only one log line fires and `err.status` is preserved.

**D4 — GET (IPN legacy)**: applies the same matrix. Adding signature verification is explicitly out of scope (security gap noted as follow-up, not changed here).

## Data Flow

```
MP API response ──getPayment()──> Error{.status,.detail,.retryAfter}
                              │  (no inner catch — bubbles)
                              v
                  handler catch (POST & GET share classifier)
                              │ classify by err.status
                              v
 404→200{w,warn} | 401/403→200{err,MP_API_AUTH_ERROR}
 429→503+Retry-After | 5xx→503 | none→500
```
`activateApprovedPayment` DB errors carry no `.status` → naturally classify to 500.

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `web/src/lib/payments/mercadopago.ts` | Modify | `getPayment` L185-188: enrich thrown Error with `.status`/`.detail`/`.retryAfter`; add exported `MpApiError` interface |
| `web/src/pages/api/webhooks/mercadopago.ts` | Modify | POST+GET catch classify by `err.status` per matrix; remove `processPayment` inner try/catch (L126-134); single log at handler |
| `web/src/test/payments/mercadopago.test.ts` | Modify | Mock `getPayment` rejection with `Error`+`status` for 404/401/403/429/5xx; assert handler 200/503/500 + `Retry-After` + single log |

## Interfaces / Contracts

```ts
// mercadopago.ts — mirrors createPreference L156 pattern; not a class
export interface MpApiError extends Error {
  status?: number;      // MP API HTTP status (404/401/403/429/5xx)
  detail?: string;      // parsed MP error detail
  retryAfter?: string;  // MP 'retry-after' header (429 only)
}
```
`getPayment` signature is unchanged (still throws on non-2xx). Handler catch extracts status: `const mpErr = err as MpApiError; const status = mpErr.status;`.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `getPayment` attaches `.status` | Mock `global.fetch` → 404/429; assert thrown `MpApiError.status` |
| Integration | Handler classification matrix | Mock `getPayment` to reject with `Error`+`status`; assert POST returns 200 (404/401/403), 503 (429/5xx), plus `Retry-After` header on 429 + `warning` body on 404 |
| Integration | DB→500 path | Mock `activateApprovedPayment` to throw plain `Error` (no `.status`); assert 500 |
| Integration | Single log / no double | Assert `console.error`/`console.warn` called once per error (inner catch removed) |
| E2E | — | N/A: a real webhook needs MP secret + live delivery; covered by unit/integration |

Command: `bun run test -- web/src/test/payments/mercadopago.test.ts` then `bunx tsc --noEmit`.

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary changes. HMAC signature logic is untouched; only HTTP response classification changes.

## Migration / Rollout

No migration required. No DB/schema changes. Deploy to Vercel; webhook URL unchanged. `MP_WEBHOOK_SECRET` gate and signature verification logic are untouched, so the deployment-time 500 gate is preserved. Behavior change is runtime-only.

## Rollback

Revert `getPayment` `.status`/`.detail`/`.retryAfter` additions and restore `processPayment` inner try/catch (L126-134). With `.status` absent, every error falls through to the 500 catch — the pre-hardening behavior. No data impact.

## Open Questions

- None blocking. Noted follow-up (separate change): GET/IPN legacy handler lacks signature verification — an acknowledged security gap, intentionally NOT addressed here (D4).
